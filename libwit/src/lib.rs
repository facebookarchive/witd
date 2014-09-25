#![crate_name="libwit"]
#![crate_type="dylib"]

extern crate time;
extern crate curl;
extern crate serialize;
use std::io::MemWriter;
use serialize::json;
use std::io;
use std::string::String;

mod wit;
mod mic;

#[deriving(Clone)]
pub struct WitHandle {
    tx: Sender<wit::WitCommand>
}

fn receive_json(receiver: Receiver<Result<json::Json,wit::RequestError>>) -> Option<String> {
    let result = receiver.recv();
    println!("[wit] received from wit: {}", result);
    match result {
        Ok(json) => {
            let mut s = MemWriter::new();
            json.to_pretty_writer(&mut s as &mut io::Writer).unwrap();
            String::from_utf8(s.unwrap()).ok()
        }
        Err(e) => {
            println!("[wit] an error occurred: {}", e)
            None
        }
    }
}

pub fn init(device_opt: Option<String>) -> WitHandle {
    let handle = WitHandle {
        tx: wit::init(wit::Options{input_device: device_opt.clone()})
    };
    println!("[wit] initialized with device {}", device_opt.unwrap_or("default".to_string()));
    handle
}

pub fn start_recording(handle: &WitHandle, access_token: String) {
    wit::start_recording(&handle.tx, access_token);
}

pub fn stop_recording(handle: &WitHandle) -> Option<String> {
    let receiver = wit::stop_recording(&handle.tx);
    receive_json(receiver)
}

pub fn text_query(handle: &WitHandle, text: String, access_token: String) -> Option<String> {
    let receiver = wit::interpret_string(&handle.tx, access_token, text);
    receive_json(receiver)
}
