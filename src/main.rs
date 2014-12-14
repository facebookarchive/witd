extern crate time;
extern crate hyper;
extern crate url;
extern crate getopts;
extern crate wit;
extern crate serialize;

use std::collections::HashMap;
use std::io::net::ip::{IpAddr, Ipv4Addr};
use std::{os, io};
use getopts::{optopt, optflag, getopts, usage};
use serialize::json;
use serialize::json::Json;
use std::io::MemWriter;
use std::collections::TreeMap;
use hyper::{status, server, uri};
use hyper::header::common;
use hyper::server::response::Response;
use hyper::server::request::Request;
use std::sync::Mutex;

struct HttpHandler {
    wit_handle: Mutex<wit::cmd::WitHandle>,
    default_autoend: bool
}

fn parse_query_params<'s>(uri: &'s str) -> HashMap<&'s str, &'s str> {
    let mut args = HashMap::<&'s str, &'s str>::new();
    let all_params: Vec<&str> = uri.split('&').collect();
    for param in all_params.iter() {
        let v_params:Vec<&str> = param.split('=').collect();
        match v_params.as_slice() {
            [k] => args.insert(k, "true"),
            [k, v] => args.insert(k, v),
            [k, v, ..] => args.insert(k, v),
            _ => None
        };
    }
    return args;
}

fn opt_string_from_result(json_result: Result<json::Json, wit::cmd::RequestError>) -> Option<String> {
    json_result.ok().and_then(|json| {
        println!("[wit] received response: {}", json);
        let mut s = MemWriter::new();
        json.to_writer(&mut s as &mut io::Writer).unwrap();
        String::from_utf8(s.into_inner()).ok()
    })
}

fn write_resp(wit_res: Result<json::Json, wit::cmd::RequestError>, mut res: server::Response) {

    fn handle_io_result<T>(result: io::IoResult<T>) {
        match result {
            Err(e) => println!("[wit] error writing response: {}", e),
            _ => ()
        };
    }

    match opt_string_from_result(wit_res) {
        Some(s) => {
            let mut res = res.start().ok().expect("unable to start writing response.");
            handle_io_result(res.write(format!("{}", s).as_bytes()));
            res.end().unwrap();
        },
        None => {
            *res.status_mut() = status::StatusCode::InternalServerError;
            let mut res = res.start().ok().expect("unable to start writing response.");
            handle_io_result(res.write(b"something went wrong, sowwy!"));
            res.end().unwrap();
        }
    }
}

fn json_status_response<T>(status: &str) -> Result<Json, T> {
    let mut map = TreeMap::new();
    map.insert("status".to_string(), Json::String(status.to_string()));
    Ok(Json::Object(map))
}

impl server::Handler for HttpHandler {
    fn handle(&self, req: Request, mut res: Response) {
        let handle = {
            let handle_lock = self.wit_handle.lock();
            handle_lock.clone()
        };
        res.headers_mut().set(common::Date(time::now_utc()));
        res.headers_mut().set(common::ContentType(from_str("application/json; charset=utf-8").unwrap()));
        res.headers_mut().set(common::Server("witd 0.0.1".to_string()));

        match req.uri {
            uri::RequestUri::AbsolutePath(ref uri) => {
                let uri_vec:Vec<&str> = uri.as_slice().split('?').collect();

                match uri_vec.as_slice() {
                    ["/text", args..] => {
                        if args.len() == 0 {
                            let mut res = res.start().ok().expect("unable to start writing response.");
                            res.write("params not found (token or q)".as_bytes())
                                .unwrap_or_else(|e| println!("could not write resp: {}", e));
                            return;
                        }

                        let params = parse_query_params(uri_vec[1]);
                        let token = params.get(&"access_token");
                        let text = params.get(&"q");

                        if token.is_none() || text.is_none() {
                            let mut res = res.start().ok().expect("unable to start writing response.");
                            res.write("params not found (token or q)".as_bytes())
                                .unwrap_or_else(|e| println!("could not write resp: {}", e));
                            return;
                        }

                        let wit_res = wit::cmd::text_query(
                            &handle,
                            text.unwrap().to_string(),
                            token.unwrap().to_string()
                        );
                        write_resp(wit_res, res);
                    },
                    ["/start", args..] => {
                        // async Wit start
                        if args.len() == 0 {
                            let mut res = res.start().ok().expect("unable to start writing response.");
                            res.write("params not found (token)".as_bytes())
                                .unwrap_or_else(|e| println!("could not write resp: {}", e));
                            return;
                        }

                        let params = parse_query_params(uri_vec[1]);
                        let token = params.get(&"access_token");

                        if token.is_none() {
                            let mut res = res.start().ok().expect("unable to start writing response.");
                            res.write("params not found (token)".as_bytes())
                                .unwrap_or_else(|e| println!("could not write resp: {}", e));
                            return;
                        }
                        let token = token.unwrap().to_string();

                        let autoend_enabled = params
                            .get(&"autoend")
                            .and_then(|x| {from_str(*x)})
                            .unwrap_or(self.default_autoend);

                        if autoend_enabled {
                            let wit_res = wit::cmd::voice_query_auto(
                                &handle,
                                token
                            );
                            write_resp(wit_res, res);
                        } else {
                            wit::cmd::voice_query_start(
                                &handle,
                                token
                            );
                            write_resp(json_status_response("ok"), res);
                        }
                    },
                    ["/stop", ..] => {
                        let wit_res = wit::cmd::voice_query_stop(&handle);
                        write_resp(wit_res, res);
                    },
                    _ => {
                        println!("unk uri: {}", uri);
                        write_resp(json_status_response("error"), res);
                    }
                }
            },
            _ => println!("not absolute uri")
        }
    }
}

fn main() {
    let args = os::args();

    let opts = [
        optflag("h", "help", "display this help message"),
        optopt("i", "input", "select input device", "default"),
        optopt("a", "host", "IP address to listen on", "0.0.0.0"),
        optopt("p", "port", "TCP port to listen on", "9877"),
        optopt("e", "autoend", "Enable end of speech detection", "false"),
        optopt("v", "verbosity", "Verbosity level", "3")
    ];

    let matches = match getopts(args.tail(), &opts) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string())
    };

    let host: IpAddr =
        from_str(os::getenv("WITD_HOST")
                 .or(matches.opt_str("host"))
                 .unwrap_or("0.0.0.0".to_string())
                 .as_slice())
        .unwrap_or(Ipv4Addr(0,0,0,0));

    let port: u16 =
        from_str(os::getenv("WITD_PORT")
                 .or(matches.opt_str("port"))
                 .unwrap_or("9877".to_string())
                 .as_slice())
        .unwrap_or(9877);

    let default_autoend: bool = matches
        .opt_str("autoend")
        .and_then(|x| {
            from_str(x.as_slice())
        })
        .unwrap_or(false);

    // before Wit is initialized
    if matches.opt_present("help") {
        println!("{}", usage("witd (https://github.com/wit-ai/witd)", opts.as_slice()));
        return;
    }

    let device_opt = matches.opt_str("input");
    let verbosity = matches.opt_str("verbosity")
                        .and_then(|s| { from_str(s.as_slice()) })
                        .unwrap_or(3);
    let handle = wit::cmd::init(device_opt, verbosity);

    let server = hyper::server::Server::http(host, port);

    match server.listen(HttpHandler { wit_handle: Mutex::new(handle), default_autoend: default_autoend }) {
        Ok(_) => {
            if verbosity > 0 {
                println!("[witd] listening on {}:{}", host.to_string(), port);
            }
        },

        Err(e) => {
            println!("Couldn't listen: {}", e);
        }
    }
}

