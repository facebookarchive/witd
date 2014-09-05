extern crate curl;

use std::collections::HashMap;
use std::io;
use std::io::Writer;
use serialize::{Encodable};
use self::curl::ErrCode;
use serialize::json;
use serialize::json::Json;

use wit_client;
use mic;

pub struct Req {
    pub response_tx: Sender<String>,
    pub uri: Option<String>
}

#[deriving(Encodable)]
pub struct Error {
    pub status: uint,
    pub code: int,
    pub error: String
}

fn parse_query_params<'s>(uri: &'s str) -> HashMap<&'s str, &'s str> {
    let mut args = HashMap::<&'s str, &'s str>::new();
    let all_params: Vec<&str> = uri.split('&').collect();
    for param in all_params.iter() {
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

fn handle_text(cmd_tx: Sender<wit_client::WitRequest>, 
               response_tx: Sender<String>, 
               uri: &str) -> () {   
    let params = parse_query_params(uri);
    println!("{}", params);
    let token = params.find(&"access_token");
    match token {
        None => response_tx.send("No access token provided".to_string()),
        Some(token) => {
            let query = params.find(&"q");
            println!("Requesting Wit.AI for : {}", query);
            match query {
                None => response_tx.send("Query is empty".to_string()),
                Some(q) => {
                    println!("Sending request to Wit.AI: {}", q);
                    let (result_tx, result_rx) = channel();
                    cmd_tx.send(wit_client::WitRequest{sender: result_tx,
                                                       token: token.to_string(),
                                                       spec: wit_client::Message(q.to_string())});
                    match result_rx.recv() {
                        Ok(json) => {
                            response_tx.send(format!("{}", json));
                        },
                        Err(failure) => {
                            let err = Error {code: 2, status: 500, error: failure.to_string()};
                            response_tx.send(format!("{}", json::encode(&err)));
                        }
                    }
                }
            }
        }
    }
}

fn handle_speech_start(cmd_tx: Sender<wit_client::WitRequest>, 
                       response_tx: Sender<Result<Json,ErrCode>>, 
                       mic_req_chan_in: Sender<bool>,
                       mic_reader_out: Box<io::ChanReader>,
                       uri: &str) -> () {   
    let params = parse_query_params(uri);
    println!("{}", params);
    let token = params.find(&"access_token");
    match token {
        None => response_tx.send(Ok(json::from_str("err").unwrap())),
        Some(token) => {
            println!("Starting to listen");
            cmd_tx.send(wit_client::WitRequest{sender: response_tx,
                                               token: token.to_string(),
                                               spec: wit_client::Speech("audio/raw;encoding=unsigned-integer;bits=16;rate=8000;endian=big".to_string())});
            mic_req_chan_in.send(true);
        }
    }
}


fn handle_speech_stop(rx: &Receiver<Result<Json,ErrCode>>, 
                      response_chan_in: Sender<String>, 
                      mic_req_chan_in: Sender<bool>) -> &Receiver<Result<Json,ErrCode>> {   
    mic_req_chan_in.send(false);
    match rx.recv() {
        Ok(json) => {
            response_chan_in.send(format!("{}", json));
        },
        Err(failure) => {
            let err = Error {code: 2, status: 500, error: failure.to_string()};
            response_chan_in.send(format!("{}", json::encode(&err)));
        }
    }
    return rx;
}

fn handle_req(rx: &Receiver<Result<Json,ErrCode>>,
              tx: Sender<Result<Json,ErrCode>>, 
              cmd_tx: Sender<wit_client::WitRequest>, 
              response_tx: Sender<String>,
              wave_tx: Sender<Vec<u8>>,
              uri: String) -> &Receiver<Result<Json,ErrCode>> {
    println!("uri_parsed {}", uri);
    let uri_vec:Vec<&str> = uri.as_slice().split('?').collect();
    match uri_vec.as_slice() {
        ["/text", ..args] => handle_text(cmd_tx, response_tx, args[0]),
        ["/start", ..args] => {
            // async response
            response_tx.send("ok".to_string());
            println!("Starting mic");
            //let mic_tx:Sender<bool> = mic::init(wave_tx);
            
            //handle_speech_start(cmd_tx, tx, mic_req_chan_in, mic_reader_out, args[0]);
        },
        ["/stop", ..args] => {
            //rx = handle_speech_stop(rx, response_chan_in, mic_req_chan_in);
        },
        _ => println!("Another request : {}", uri)
    }
    return rx;
}

fn job(connector_rx: Receiver<Req>, cmd_tx: Sender<wit_client::WitRequest>, wave_tx: Sender<Vec<u8>>) {
    let (state_tx, state_rx): (Sender<Result<Json,ErrCode>>, Receiver<Result<Json,ErrCode>>) = channel();
    loop {
        //Read message from connector_chan
        let Req{response_tx: response_tx, uri: uri} = connector_rx.recv();
        println!("Receiving message in connector, will dispatch...");
        match uri {
            Some(req) => {
                println!("Request coming from http: {}", req.to_string());
                handle_req(&state_rx, state_tx.clone(), cmd_tx.clone(), response_tx, wave_tx.clone(), req);
            },
            None => {
                let err = Error{code: 4, status: 400, error: "Invalid Uri".to_string()}; 
                response_tx.send(format!("{}", json::encode(&err)));
            }
        }
    }
}

pub fn init() -> Sender<Req> {
    let (connector_tx, connector_rx): (Sender<Req>, Receiver<Req>) = channel();
    println!("Spawning Connector");
    println!("Starting a wit client");
    let (cmd_tx, wave_tx): (Sender<wit_client::WitRequest>, Sender<Vec<u8>>) = wit_client::init();
    println!("spawing connector to handle http request");
    spawn(proc() {
        job(connector_rx, cmd_tx, wave_tx);
    });
    return connector_tx;
}

