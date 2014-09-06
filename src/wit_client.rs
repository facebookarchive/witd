extern crate curl;
extern crate serialize;
use std::str;
use std::io;
use self::curl::http;
use self::curl::http::Request;
use self::curl::ErrCode;
use serialize::json;
use serialize::json::Json;
use self::curl::http::body::{Body, ToBody, ChunkedBody};
use mic;

pub enum WitRequestSpec {
    Message(String),
    Speech(String)
}

pub struct WitRequest {
    pub sender: Sender<Result<Json,ErrCode>>,
    pub spec: WitRequestSpec,
    pub token: String
}

pub enum WitCommand {
    Start(String, String),
    Stop(Sender<Result<Json, ErrCode>>)
}

fn exec_request(request: Request, token: String) -> Result<Json,ErrCode> {
    println!("[exec] start");
    request
        .header("Authorization", format!("Bearer {}", token).as_slice())
        .header("Accept", "application/vnd.wit.20140620+json")
    .exec().map(|resp| {
        println!("[exec] resp={}", resp);
        let body = resp.get_body();
        let str = str::from_utf8(body.as_slice())
            .expect("Response was not valid UTF-8");
        json::from_str(str).unwrap()
    })
}

fn message_request(msg: String, token: String) -> Result<Json,ErrCode> {
    let mut init_req = http::handle();
    let req = init_req
        .get(format!("https://api.wit.ai/message?q={}", msg));
    exec_request(req, token)
}

pub struct WrapReader<'a>(pub &'a mut Reader+'static);

impl<'a> ToBody<'a> for WrapReader<'a> {
    fn to_body(self) -> Body<'a> {
        let WrapReader(x) = self;
        ChunkedBody(x)
    }
}

fn speech_request(stream: &mut io::ChanReader, content_type:String, token: String) -> Result<Json,ErrCode> {
    let mut init_req = http::handle();
    let req = init_req.post("https://api.wit.ai/speech", WrapReader(stream))
        .content_type(content_type.as_slice())
        .chunked();
    exec_request(req, token)
}

fn job(cmd_rx: Receiver<WitRequest>, mut stream: io::ChanReader) {
    loop {
        let WitRequest { sender: sender, spec: spec, token: token } = cmd_rx.recv();
        println!("Receiving cmd in wit client");
        let result = match spec {
          Message(msg) => message_request(msg, token),
          Speech(content_type) => speech_request(&mut stream, content_type, token)
        };
        sender.send(result)
    }
}

pub fn wit_speech_start(ctl: &Sender<WitCommand>,
                        token: String,
                        content_type: String) {
    ctl.send(Start(token, content_type));
}

pub fn wit_speech_stop(ctl: &Sender<WitCommand>) -> Receiver<Result<Json,ErrCode>> {
    let (result_tx, result_rx) = channel();
    ctl.send(Stop(result_tx));
    return result_rx
}

pub fn init() -> Sender<WitCommand>{
    let (cmd_tx, cmd_rx): (Sender<WitCommand>, Receiver<WitCommand>) = channel();

    println!("[wit] init");

    spawn(proc() {
        let mut ongoing: Option<(Receiver<Result<Json,ErrCode>>, Sender<bool>)> = None;
        loop {
            let cmd = cmd_rx.recv();
            ongoing = match cmd {
                Start(token, content_type) => {
                    if ongoing.is_none() {
                        let (http_tx, http_rx) = channel();
                        let (mut reader, ctl_tx) = mic::init();
                        mic::start(&ctl_tx);

                        spawn(proc() {
                            let mut reader_ref = &mut *reader;
                            let foo = speech_request(reader_ref, content_type, token);
                            println!("{}", foo);
                            http_tx.send(foo);
                        });

                        Some((http_rx, ctl_tx))
                    } else {
                        ongoing
                    }
                },
                Stop(result_tx) => {
                    if ongoing.is_none() {
                        println!("[wit] trying to stop but no request started");
                        None
                    } else {
                        let tuple = ongoing.as_ref().unwrap();
                        let http_rx = tuple.ref0();
                        let ctl_tx = tuple.ref1();

                        mic::stop(ctl_tx);
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
