#![allow(non_camel_case_types)]

use std::c_str::CString;
use libc::c_char;
use cmd;
use cmd::WitHandle;
use native;
use std;
use std::mem;
use std::ptr;
use std::rt;

struct WitContext {
    handle: WitHandle
}

pub type wit_context_ptr = *const ();

#[no_mangle]
pub unsafe extern "C" fn wit_init(device_opt: *const c_char) -> wit_context_ptr {
    // Manually initializing the runtime
    rt::init(0, ptr::null());

    let task = native::task::new((0, std::uint::MAX));
    let mut handle: Option<WitHandle> = None;

    task.run(|| {
        let device = if device_opt.is_null() {
            None
        } else {
            let device_str = CString::new(device_opt, false);
            match device_str.as_str() {
                Some(s) => Some(s.to_string()),
                None => {
                    println!("[wit] warning: failed to read device name. Using default instead");
                    None
                }
            }
        };
        handle = Some(cmd::init(device));
    }).destroy();

    let boxed = box WitContext {
        handle: handle.unwrap()
    };
    let res: wit_context_ptr = mem::transmute(boxed);
    res
}

#[no_mangle]
pub unsafe extern "C" fn wit_start_recording(context: wit_context_ptr, access_token: *const c_char) {
    let task = native::task::new((0, std::uint::MAX));
    let context: &WitContext = mem::transmute(context);
    task.run(|| {
        let access_token_opt = CString::new(access_token, false);
        match access_token_opt.as_str() {
            Some(access_token_str) => {
                let access_token = access_token_str.to_string();
                cmd::start_recording(&context.handle, access_token)
            }
            None => {
                println!("[wit] error: failed to read access token");
            }
        }
    }).destroy();
}

#[no_mangle]
pub unsafe extern "C" fn wit_stop_recording(context: wit_context_ptr) -> *const c_char {
    let task = native::task::new((0, std::uint::MAX));
    let context: &WitContext = mem::transmute(context);
    let mut result: Option<String> = None;
    task.run(|| {
        result = cmd::stop_recording(&context.handle);
    }).destroy();
    match result {
        Some(r) => r.to_c_str().as_ptr(),
        None => ptr::null()
    }
}

#[no_mangle]
pub unsafe extern "C" fn wit_text_query(context: wit_context_ptr, text: *const c_char, access_token: *const c_char) -> *const c_char {
    let task = native::task::new((0, std::uint::MAX));
    let context: &WitContext = mem::transmute(context);
    let mut result: Option<String> = None;
    task.run(|| {
        let access_token_opt = CString::new(access_token, false);
        match access_token_opt.as_str() {
            Some(access_token_str) => {
                let access_token = access_token_str.to_string();
                let text_opt = CString::new(text, false);
                match text_opt.as_str() {
                    Some(text_str) => {
                        let text = text_str.to_string();
                        result = cmd::text_query(&context.handle, text, access_token)
                    },
                    None => {
                        println!("[wit] error: failed to read query text");
                    }
                }
            }
            None => {
                println!("[wit] error: failed to read access token");
            }
        }
    }).destroy();
    match result {
        Some(r) => r.to_c_str().as_ptr(),
        None => ptr::null()
    }
}

