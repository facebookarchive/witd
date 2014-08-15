extern crate time;
extern crate http;
extern crate url;

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::Writer;
use http::server::{Config, Server, Request, ResponseWriter};
use http::server::request::{AbsoluteUri, AbsolutePath};
use http::headers::content_type::MediaType;
use url::{Url, RelativeSchemeData, NonRelativeSchemeData, ParseResult};

#[deriving(Clone)]
struct WitDServer;

fn handle_req(w: &mut ResponseWriter, uri: &Url) -> () {
    println!("uri_parsed {}", uri);
}

impl Server for WitDServer {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(0, 0, 0, 0), port: 9877 } }
    }

    fn handle_request(&self, _r: Request, w: &mut ResponseWriter) {
        w.headers.date = Some(time::now_utc());
        w.headers.content_type = Some(MediaType {
            type_: String::from_str("application"),
            subtype: String::from_str("json"),
            parameters: vec!((String::from_str("charset"), String::from_str("UTF-8")))
        });
        w.headers.server = Some(String::from_str("WitD rust powered"));

        match _r.request_uri {
            AbsoluteUri(ref url) => handle_req(w, url),
            AbsolutePath(ref uri) => println!("Path : {}", uri),
            _ => println!("Invalid request")
        } 
        w.write(b"{\"status\": 200}").unwrap();
    }
}

fn main() {
    WitDServer.serve_forever();
}
