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

pub enum WitRequestSpec {
    Message(String),
    Speech(String)
}

pub struct WitRequest {
    pub sender: Sender<Result<Json,ErrCode>>,
    pub spec: WitRequestSpec,
    pub token: String
}

fn exec_request(request: Request, token: String) -> Result<Json,ErrCode> {
    request
        .header("Authorization", format!("Bearer {}", token).as_slice())
        .header("Accept", "application/vnd.wit.20140620+json")
    .exec().map(|resp| {
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

pub fn init() -> (Sender<WitRequest>, Sender<Vec<u8>>){
    let (cmd_tx, cmd_rx): (Sender<WitRequest>, Receiver<WitRequest>) = channel();
    let (wave_tx, wave_rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();
    let stream = io::ChanReader::new(wave_rx);
    println!("Spawning wit client ...");
    spawn(proc() {
        job(cmd_rx, stream);
    });
    return (cmd_tx, wave_tx);
}
