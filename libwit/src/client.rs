use std::str;
use std::io;
use curl::http;
use curl::http::Request;
use curl::ErrCode;
use serialize::json;
use serialize::json::Json;
use curl::http::body::{Body, ToBody, ChunkedBody};
use url;

use mic;

pub enum WitCommand {
    Text(String, String, Sender<Result<Json, RequestError>>),
    Start(String),
    Stop(Sender<Result<Json, RequestError>>)
}

#[deriving(Show)]
pub enum RequestError {
    InvalidResponseError,
    ParserError(json::ParserError),
    NetworkError(ErrCode),
    StatusError(uint)
}

pub struct State {
    http: Receiver<Result<Json,RequestError>>,
    mic: Sender<bool>,
}

pub struct Options {
    pub input_device: Option<String>
}

fn exec_request(request: Request, token: String) -> Result<Json,RequestError> {
    request
        .header("Authorization", format!("Bearer {}", token).as_slice())
        .header("Accept", "application/vnd.wit.20140620+json")
        .exec()
        .map_err(|e| {
            println!("[wit] network error: {}", e);
            NetworkError(e)
        })
        .and_then(|x| {
            let status = x.get_code();
            if status >= 400 {
                println!("[wit] server responded with error: {}", status);
                return Err(StatusError(status));
            }
            let body = x.get_body();
            match str::from_utf8(body.as_slice()) {
                Some(str) => {
                    let obj = json::from_str(str);
                    obj.map_err(|e| {
                        println!("[wit] could not parse response from server: {}", str);
                        ParserError(e)
                    })
                }
                None => {
                    println!("[wit] response was not valid UTF-8");
                    Err(InvalidResponseError)
                }
            }
        })
}

fn do_message_request(msg: String, token: String) -> Result<Json,RequestError> {
    let mut init_req = http::handle();
    let encoded = url::utf8_percent_encode(msg.as_slice(), url::QUERY_ENCODE_SET);
    let req = init_req
        .get(format!("https://api.wit.ai/message?q={}", encoded));
    exec_request(req, token)
}

pub struct WrapReader<'a>(pub &'a mut Reader+'static);

impl<'a> ToBody<'a> for WrapReader<'a> {
    fn to_body(self) -> Body<'a> {
        let WrapReader(x) = self;
        ChunkedBody(x)
    }
}

fn do_speech_request(stream: &mut io::ChanReader, content_type:String, token: String) -> Result<Json,RequestError> {
    let mut init_req = http::handle();
    let req = init_req.post("https://api.wit.ai/speech", WrapReader(stream))
        .content_type(content_type.as_slice())
        .chunked();
    exec_request(req, token)
}

pub fn interpret_string(ctl: &Sender<WitCommand>,
                        token: String,
                        text: String) -> Receiver<Result<Json,RequestError>> {
    let (result_tx, result_rx) = channel();
    ctl.send(Text(token, text, result_tx));
    return result_rx
}

pub fn start_recording(ctl: &Sender<WitCommand>,
                        token: String) {
    ctl.send(Start(token));
}

pub fn stop_recording(ctl: &Sender<WitCommand>) -> Receiver<Result<Json,RequestError>> {
    let (result_tx, result_rx) = channel();
    ctl.send(Stop(result_tx));
    return result_rx
}

pub fn init(opts: Options) -> Sender<WitCommand>{
    mic::init();

    let (cmd_tx, cmd_rx): (Sender<WitCommand>, Receiver<WitCommand>) = channel();

    println!("[wit] init");

    spawn(proc() {
        let mut ongoing: Option<State> = None;
        loop {
            println!("[wit] ready. state={}", if ongoing.is_none() {"no"} else {"yes"});
            let cmd = cmd_rx.recv_opt();
            if cmd.is_err() {
                break;
            }
            ongoing = match cmd.unwrap() {
                Text(token, text, result_tx) => {
                    let r = do_message_request(text, token);
                    result_tx.send(r);
                    ongoing
                }
                Start(token) => {
                    if ongoing.is_none() {
                        let mic_context_opt = mic::start(opts.input_device.clone());

                        let (http_tx, http_rx) = channel();
                        let mic::MicContext {
                            reader: mut reader,
                            sender: mic_tx,
                            rate: rate,
                            encoding: encoding
                        } = mic_context_opt.unwrap();

                        let content_type =
                            format!("audio/raw;encoding={};bits=16;rate={};endian=big", encoding, rate);
                        println!("Sending speech request with content type: {}", content_type);
                        spawn(proc() {
                            let reader_ref = &mut *reader;
                            let foo = do_speech_request(reader_ref, content_type, token);
                            http_tx.send(foo);
                        });

                        Some(State {
                            http: http_rx,
                            mic: mic_tx,
                        })
                    } else {
                        ongoing
                    }
                },
                Stop(result_tx) => {
                    if ongoing.is_none() {
                        println!("[wit] trying to stop but no request started");
                        None
                    } else {
                        let State { http: http_rx, mic: mic_tx } = ongoing.unwrap();

                        mic::stop(&mic_tx);
                        let foo = http_rx.recv();
                        result_tx.send(foo);

                        None
                    }
                }
            }
        }
    });
    return cmd_tx
}
