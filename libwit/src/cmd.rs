use std::io::MemWriter;
use serialize::json;
use std::io;

use client;

pub type WitHandle = Sender<client::WitCommand>;

fn receive_json(receiver: Receiver<Result<json::Json,client::RequestError>>) -> Option<String> {
    receiver.recv().ok().and_then(|json| {
        println!("[wit] received response: {}", json);
        let mut s = MemWriter::new();
        json.to_writer(&mut s as &mut io::Writer).unwrap();
        String::from_utf8(s.unwrap()).ok()
    })
}

pub fn init(device_opt: Option<String>) -> WitHandle {
    let handle = client::init(client::Options{input_device: device_opt.clone()});
    println!("[wit] initialized with device: {}", device_opt.unwrap_or("default".to_string()));
    handle
}

pub fn start_recording(handle: &WitHandle, access_token: String) {
    client::start_recording(handle, access_token);
}

pub fn stop_recording(handle: &WitHandle) -> Option<String> {
    let receiver = client::stop_recording(handle);
    receive_json(receiver)
}

pub fn text_query(handle: &WitHandle, text: String, access_token: String) -> Option<String> {
    let receiver = client::interpret_string(handle, access_token, text);
    receive_json(receiver)
}
