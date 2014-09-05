extern crate time;
extern crate http;
extern crate url;
extern crate serialize;
use std::io::net::ip::{SocketAddr, IpAddr, Ipv4Addr};
use http::server::{Config, Server, ResponseWriter};
use http::server::request::{AbsolutePath, Request, RequestUri};
use http::headers::content_type::MediaType;
use std::os;
use std::io;
use serialize::json;
mod connector;
mod wit_client;
mod mic;

#[deriving(Clone)]
struct HttpServer {
    connector_tx: Sender<connector::Req>,
    host: IpAddr,
    port: u16
}


fn return_err(w: &mut ResponseWriter, uri: Option<RequestUri>, err: String) -> () {
    println!("An error occured for uri {} : {}", uri, err);
    let error = connector::Error {status: 400, code: 1, error: err};
    w.write(json::encode(&error).as_bytes()).unwrap();
}

impl Server for HttpServer {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: self.host, port: self.port } }
    }

    fn handle_request(&self, _r: http::server::request::Request, w: &mut ResponseWriter) {
        w.headers.date = Some(time::now_utc());
        w.headers.content_type = Some(MediaType {
            type_: String::from_str("application"),
            subtype: String::from_str("json"),
            parameters: vec!((String::from_str("charset"), String::from_str("UTF-8")))
        });
        w.headers.server = Some(String::from_str("WitD rust powered"));
        
        let (response_tx, response_rx) = channel();
        match _r.request_uri {
            AbsolutePath(ref uri) => self.connector_tx.send(connector::Req{response_tx: response_tx, uri: Some(uri.clone())}),
            _ => return_err(w, Some(_r.request_uri), "Invalid URI".to_string())
        };
        let json = response_rx.recv();
        println!("Response from Wit.AI : {}", json);
        w.write(format!("{}", json).as_bytes()).unwrap();        
    }
}

fn start_connector() -> Sender<connector::Req> {
    return connector::init();
}

fn main() {        
    let host: IpAddr = from_str(os::getenv("HOST").unwrap_or("0.0.0.0".to_string()).as_slice()).unwrap_or(Ipv4Addr(0,0,0,0));
    let port: u16 = from_str(os::getenv("PORT").unwrap_or("9877".to_string()).as_slice()).unwrap_or(9877);
    
    let connector_tx = start_connector();

    let server = HttpServer {
        host: host,
        port: port,
        connector_tx: connector_tx
    };

    println!("Server will listen on {}:{}", host.to_string(), port);
    server.serve_forever();
}
