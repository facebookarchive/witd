#![allow(non_camel_case_types)]

use std::c_str::CString;
use libc::c_char;
use cmd;
use cmd::WitHandle;
use native;
use std::rt::task::Task;
use std;
use std::mem;

struct WitContext {
    task: Box<Task>,
    handle: WitHandle
}

pub type wit_context_ptr = *const ();

#[no_mangle]
pub unsafe extern "C" fn wit_init(device_opt: *const c_char) -> wit_context_ptr {
    let task = native::task::new((0, std::uint::MAX));
    let mut handle: Option<WitHandle> = None;

    let task = task.run(|| {
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
    });

    let boxed = box WitContext {
        task: task,
        handle: handle.unwrap()
    };
    let res: wit_context_ptr = mem::transmute(boxed);
    res
}

#[no_mangle]
pub unsafe extern "C" fn wit_start_recording(context: wit_context_ptr, access_token: *const c_char) {
    let context: Box<WitContext> = mem::transmute(context);
    let handle = context.handle.clone();
    context.task.run(|| {
        let access_token_str = CString::new(access_token, false);
        match access_token_str.as_str() {
            Some(s) => {
                let access_token = s.to_string();
                    cmd::start_recording(&handle, access_token)
            }
            None => {
                println!("[wit] error: failed to read access token");
            }
        }
    }).destroy();
}


