extern crate time;
extern crate curl;
extern crate http;
extern crate url;
extern crate serialize;
use std::io::net::ip::{SocketAddr, IpAddr, Ipv4Addr};
use http::server::{Config, Server, ResponseWriter};
use http::server::request::{AbsolutePath, Request, RequestUri};
use http::headers::content_type::MediaType;
use std::os;
use std::io;
use self::curl::ErrCode;
use serialize::json;
use serialize::json::Json;
mod wit_client;
mod mic;

#[deriving(Clone)]
struct HttpServer {
    host: IpAddr,
    port: u16,
    wit_tx: Sender<wit_client::WitCommand>
}

    // let params = parse_query_params(uri);
    // println!("{}", params);
    // let token = params.find(&"access_token");
    // match token {
    //     None => response_tx.send(Ok(json::from_str("err").unwrap())),
        // Some(token) => {
        //     println!("Starting to listen");
        //     cmd_tx.send(wit_client::WitRequest{sender: response_tx,
        //                                        token: token.to_string(),
        //                                        spec: wit_client::Speech("audio/raw;encoding=unsigned-integer;bits=16;rate=8000;endian=big".to_string())});
        //     mic_req_chan_in.send(true);
        // }

impl Server for HttpServer {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: self.host, port: self.port } }
    }

    fn handle_request(&self, r: http::server::request::Request, w: &mut ResponseWriter) {
        w.headers.date = Some(time::now_utc());
        w.headers.content_type = Some(MediaType {
            type_: format!("application"),
            subtype: format!("json"),
            parameters: vec!((format!("charset"), format!("UTF-8")))
        });

        w.headers.server = Some(format!("witd 0.0.1"));


        println!("http request: {}", r.request_uri);
        match r.request_uri {
            AbsolutePath(ref uri) => {
                let uri_vec:Vec<&str> = uri.as_slice().split('?').collect();
                match uri_vec.as_slice() {
                    ["/text", ..args] =>
                        println!("[http] text request"),
                        // handle_text(cmd_tx, response_tx, args[0]),
                    ["/start", ..args] => {
                        println!("[http] start request");
                        // async Wit start
                        let token = format!("ASAYW7NKIBW63T5LRWT2MDWLYDHGZQG7");
                        let content_type = format!("audio/raw;encoding=unsigned-integer;bits=16;rate=8000;endian=big");
                        wit_client::wit_speech_start(&self.wit_tx, token, content_type);
                    },
                    ["/stop", ..args] => {
                        println!("[http] stop request");
                        let wit_rx = wit_client::wit_speech_stop(&self.wit_tx);
                        let json = wit_rx.recv();
                        println!("[http] recv from wit: {}", json);
                        w.write(format!("{}", json.unwrap()).as_bytes()).unwrap();
                    },
                    _ => println!("unk uri: {}", uri)
                }
            }
            _ => println!("not absolute uri")
        };
    }
}

fn main() {
    let host: IpAddr =
        from_str(os::getenv("HOST")
                 .unwrap_or("0.0.0.0".to_string())
                 .as_slice())
        .unwrap_or(Ipv4Addr(0,0,0,0));

    let port: u16 =
        from_str(os::getenv("PORT")
                 .unwrap_or("9877".to_string())
                 .as_slice())
        .unwrap_or(9877);

    let wit_tx = wit_client::init();

    let server = HttpServer {
        host: host,
        port: port,
        wit_tx: wit_tx
    };

    println!("[witd] listening on {}:{}", host.to_string(), port);
    server.serve_forever();
}
