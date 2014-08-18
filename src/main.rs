extern crate time;
extern crate http;
extern crate curl;
extern crate url;
extern crate serialize;
use std::io::net::ip::{SocketAddr, IpAddr, Ipv4Addr};
use http::server::{Config, Server, ResponseWriter};
use http::server::request::{AbsolutePath, Request};
use http::headers::content_type::MediaType;
use std::os;
mod wit_server;
mod wit_client;

#[deriving(Clone)]
struct WitDServer {
    req_chan_in: Sender<wit_client::WitRequest>,
    host: IpAddr,
    port: u16
}

impl Server for WitDServer {
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

        match _r.request_uri {
            AbsolutePath(ref uri) => wit_server::handle_req(&self.req_chan_in, w, uri),
            _ => wit_server::handle_err(w, Some(_r.request_uri), "Invalid URI".to_string())
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
    let host: IpAddr = from_str(os::getenv("HOST").unwrap_or("0.0.0.0".to_string()).as_slice()).unwrap_or(Ipv4Addr(0,0,0,0));
    let port: u16 = from_str(os::getenv("PORT").unwrap_or("9877".to_string()).as_slice()).unwrap_or(9877);
    
    let req_chan_in = start_wit_client();
    let server = WitDServer {
        host: host,
        port: port,
        req_chan_in: req_chan_in
    };

    println!("Server will listen on {}:{}", host.to_string(), port);
    server.serve_forever();
}
