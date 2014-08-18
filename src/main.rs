extern crate time;
extern crate http;
extern crate curl;
extern crate url;
extern crate serialize;

use std::collections::HashMap;
use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::Writer;
use serialize::{json, Encodable};
use http::server::{Config, Server, Request, ResponseWriter};
use http::server::request::{AbsolutePath, RequestUri};
use http::headers::content_type::MediaType;
use curl::http::Request;
use curl::ErrCode;
use serialize::json;
use serialize::json::Json;

mod wit_client;

#[deriving(Encodable)]
struct HttpError {
    status: uint,
    code: int,
    error: String
}



fn parse_query_params<'s>(uri: &[&'s str]) -> HashMap<&'s str, &'s str> {
    let mut args = HashMap::<&'s str, &'s str>::new();
    for param in uri.iter() {
        let v_params:Vec<&str> = param.split('=').collect();
        let inserted = match v_params.as_slice() {
            [k] => args.insert(k, "true"),
            [k, v] => args.insert(k, v),
            [k, v, ..] => args.insert(k, v),
            _ => false
        };
        println!("param {} inserted : {}", v_params, inserted);
    }
    return args;
}


fn handle_err(w: &mut ResponseWriter, uri: Option<RequestUri>, err: String) -> () {
    println!("An error occured for uri {} : {}", uri, err);
    let error = HttpError {status: 400, code: 1, error: err};
    w.write(json::encode(&error).as_bytes()).unwrap();
}

fn handle_text(req_chan_in: &Sender<wit_client::WitRequest>, 
               w: &mut ResponseWriter, 
               uri: &[&str]) -> () {   
    let params = parse_query_params(uri);
    println!("{}", params);
    let query = params.find(&"q");
    println!("Requesting Wit.AI for : {}", query);
    match query {
        None => handle_err(w, None, "Query is empty".to_string()),
        Some(q) => {
            println!("Sending request to Wit.AI: {}", q);
            let (result_chan_in, result_chan_out) = channel();
            req_chan_in.send(wit_client::WitRequest{sender: result_chan_in, 
                                                    spec: wit_client::Message(q.to_string())});
            match result_chan_out.recv() {
                Ok(json) => {
                    w.write(format!("{}", json).as_bytes()).unwrap();        
                },
                Err(failure) => {
                    handle_err(w, None, failure.to_string())
                }
            }
        }
    }
}

fn handle_req(req_chan_in: &Sender<wit_client::WitRequest>, 
              w: &mut ResponseWriter, 
              uri: &String) -> () {
    println!("uri_parsed {}", uri);
    let uri_vec:Vec<&str> = uri.as_slice().split('?').collect();
    match uri_vec.as_slice() {
        ["/text", ..args] => handle_text(req_chan_in, w, args),
        _ => println!("Another request : {}", uri)
    }
}

#[deriving(Clone)]
struct WitDServer {
    req_chan_in: Sender<wit_client::WitRequest>,
}

impl Server for WitDServer {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(0, 0, 0, 0), port: 9877 } }
    }

    fn handle_request(&self, _r: http::server::request::Request, w: &mut ResponseWriter) {
        w.headers.date = Some(time::now_utc());
        w.headers.content_type = Some(MediaType {
            type_: String::from_str("application"),
            subtype: String::from_str("json"),
            parameters: vec!((String::from_str("charset"), String::from_str("UTF-8")))
        });
        w.headers.server = Some(String::from_str("WitD rust powered"));

        match _r.request_uri {
            AbsolutePath(ref uri) => handle_req(&self.req_chan_in, w, uri),
            _ => handle_err(w, Some(_r.request_uri), "Invalid URI".to_string())
        } 
    }
}

fn start_wit_client() -> Sender<wit_client::WitRequest> {
    let (req_chan_in, req_chan_out): (Sender<wit_client::WitRequest>, Receiver<wit_client::WitRequest>) = channel();
    spawn(proc() {
        wit_client::result_fetcher(req_chan_out);
    });
    return req_chan_in;
}

fn main() {        
    let req_chan_in = start_wit_client();
    let server = WitDServer {
        req_chan_in: req_chan_in
    };
    server.serve_forever();
}
