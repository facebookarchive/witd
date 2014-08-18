extern crate http;

use std::collections::HashMap;
use std::io::Writer;
use serialize::{Encodable};
use http::server::{ResponseWriter};
use http::server::request::{RequestUri};
use serialize::json;
use wit_client;

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
                    println!("Response from Wit.AI : {}", json);
                    w.write(format!("{}", json).as_bytes()).unwrap();        
                },
                Err(failure) => {
                    handle_err(w, None, failure.to_string())
                }
            }
        }
    }
}

pub fn handle_req(req_chan_in: &Sender<wit_client::WitRequest>, 
              w: &mut ResponseWriter, 
              uri: &String) -> () {
    println!("uri_parsed {}", uri);
    let uri_vec:Vec<&str> = uri.as_slice().split('?').collect();
    match uri_vec.as_slice() {
        ["/text", ..args] => handle_text(req_chan_in, w, args),
        _ => println!("Another request : {}", uri)
    }
}

pub fn handle_err(w: &mut ResponseWriter, uri: Option<RequestUri>, err: String) -> () {
    println!("An error occured for uri {} : {}", uri, err);
    let error = HttpError {status: 400, code: 1, error: err};
    w.write(json::encode(&error).as_bytes()).unwrap();
}
