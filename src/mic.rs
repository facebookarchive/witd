extern crate libc;

use std::{c_str, c_vec, fmt, io, mem, ptr, sync, time};
use self::libc::{c_void, c_char, c_int, c_long, c_double, size_t, malloc};

type PaDeviceIndex = c_int;
type PaError = c_int;
type PaHostApiIndex = i32;
static paNoError: c_int = 0;

/*
#define paFloat32        ((PaSampleFormat) 0x00000001)
#define paInt32          ((PaSampleFormat) 0x00000002)
#define paInt24          ((PaSampleFormat) 0x00000004)
#define paInt16          ((PaSampleFormat) 0x00000008)
#define paInt8           ((PaSampleFormat) 0x00000010)
#define paUInt8          ((PaSampleFormat) 0x00000020)
#define paCustomFormat   ((PaSampleFormat) 0x00010000)
#define paNonInterleaved ((PaSampleFormat) 0x80000000)
*/
static paInt16: u32 = 0x00000008;
static paUInt8: u32 = 0x00000020;

type PaSampleFormat = u32;

type PaStream = c_void;
pub type PaTime = c_double;

// callback stuff
pub struct PaStreamCallbackTimeInfo {
    pub input_buffer_adc_time : PaTime,
    pub current_time : PaTime,
    pub output_buffer_dac_time : PaTime
}
pub enum PaStreamCallbackResult {
    PaContinue = 0,
    PaComplete = 1,
    PaAbort = 2
}

// Stream flags
pub type PaStreamFlags = u64;
pub static PaNoFlag: PaStreamFlags = 0;
pub static PaClipOff: PaStreamFlags = 0x00000001;
pub static PaDitherOff: PaStreamFlags = 0x00000002;
pub static PaNeverDropInput: PaStreamFlags = 0x00000004;
pub static PaPrimeOutputBuffersUsingStreamCallback: PaStreamFlags = 0x00000008;
pub static PaPlatformSpecificFlags: PaStreamFlags = 0xFFFF0000;

pub type PaHostApiTypeId = i32;
pub static PaInDevelopment: PaHostApiTypeId = 0;
pub static PaDirectSound: PaHostApiTypeId = 1;
pub static PaMME: PaHostApiTypeId = 2;
pub static PaASIO: PaHostApiTypeId = 3;
pub static PaSoundManager: PaHostApiTypeId = 4;
pub static PaCoreAudio: PaHostApiTypeId = 5;
pub static PaOSS: PaHostApiTypeId = 7;
pub static PaALSA: PaHostApiTypeId = 8;
pub static PaAL: PaHostApiTypeId = 9;
pub static PaBeOS: PaHostApiTypeId = 10;
pub static PaWDMKS: PaHostApiTypeId = 11;
pub static PaJACK: PaHostApiTypeId = 12;
pub static PaWASAPI: PaHostApiTypeId = 13;
pub static PaAudioScienceHPI: PaHostApiTypeId = 14;

/// A structure containing information about a particular host API.
#[repr(C)]
pub struct PaHostApiInfo {
    pub struct_version: i32,
    pub host_type: i32,
    pub name: *const c_char,
    pub device_count: i32,
    pub default_input_device: i32,
    pub default_output_device: i32
}
#[repr(C)]
#[deriving(Clone, PartialEq, PartialOrd)]
pub struct PaDeviceInfo {
    pub struct_version: i32,
    pub name: *const c_char,
    pub host_api: PaHostApiIndex,
    pub max_input_channels: i32,
    pub max_output_channels: i32,
    pub default_low_input_latency: PaTime,
    pub default_low_output_latency: PaTime,
    pub default_high_input_latency: PaTime,
    pub default_high_output_latency: PaTime,
    pub default_sample_rate: c_double
}
#[repr(C)]
pub struct PaStreamParameters {
    pub device : PaDeviceIndex,
    pub channel_count : i32,
    pub sample_format : PaSampleFormat,
    pub suggested_latency : PaTime,
    pub host_api_specific_stream_info : *mut c_void
}

pub type PaStreamCallbackFlags = u64;
type PaStreamCallback =
    extern "C" fn(*const c_void, *mut c_void, u32,
                  *const PaStreamCallbackTimeInfo,
                  PaStreamCallbackFlags, *mut c_void) -> PaStreamCallbackResult;

#[link(name = "portaudio")]
extern {
    fn Pa_Initialize() -> c_void;
    fn Pa_GetErrorText(e: PaError) -> *const c_char;
    fn Pa_GetDeviceCount() -> PaDeviceIndex;
    fn Pa_GetDeviceInfo(i: c_int) -> *const PaDeviceInfo;
    fn Pa_GetDefaultInputDevice() -> PaDeviceIndex; // PaDeviceIndex
    fn Pa_GetHostApiInfo(i: c_int) -> *const PaHostApiInfo;
    fn Pa_OpenStream(
        stream: *mut *mut PaStream,
        inputParams: *const PaStreamParameters,
        outputParams: *const PaStreamParameters,
        sampleRate: c_double,
        framesPerBuffer: u32,
        streamFlags: PaStreamFlags,
        streamCallBack: Option<PaStreamCallback>,
        userData: *mut c_void)
        -> PaError;
    fn Pa_OpenDefaultStream(
        stream: *mut *mut PaStream,
        numInputChannels: c_int,
        numOutputChannels: c_int,
        sampleFormat: PaSampleFormat,
        sampleRate: c_double,
        framesPerBuffer: u32,
        streamCallBack: Option<PaStreamCallback>,
        userData: *mut c_void)
        -> PaError;
    fn Pa_StartStream(stream: *mut PaStream) -> PaError;
    fn Pa_StopStream(stream: *mut PaStream) -> PaError;
    fn Pa_ReadStream(stream: *mut PaStream, buffer: *mut c_void, frames: u32) -> PaError;
    fn Pa_Sleep(msec: c_long) -> c_void;
}

struct MicState {
    stream: *mut PaStream,
    tx: Sender<Vec<u8>>
}

extern "C" fn stream_callback
    (input: *const c_void, output: *mut c_void,
     frame_count: u32, info: *const PaStreamCallbackTimeInfo,
     flags: PaStreamCallbackFlags, data: *mut c_void)
     -> PaStreamCallbackResult {
         // println!("rx {} frames", frame_count);

         let c_bytes: c_vec::CVec<u8> = unsafe {
             c_vec::CVec::new(input as *mut u8, frame_count as uint)
         };
         let bytes: Vec<u8> = c_bytes.as_slice().to_vec();

         let tx: &mut Sender<Vec<u8>> = unsafe {
             &mut *(data as *mut Sender<Vec<u8>>)
         };

         // println!("tx addr: {:p}, bytes len: {}", tx, bytes.len())
         let result = tx.send_opt(bytes);
         if result.is_err() {
             println!("error while sending: {}", result.err());
         }

         PaContinue
     }

fn print_err(tag: &str, err: PaError) {
    unsafe {
        let msg = c_str::CString::new(Pa_GetErrorText(err), false);
        println!("[mic] {}: {}", tag, msg.as_str().unwrap_or("unk"));
    }
}

pub fn start(input_device: Option<int>) -> (Box<io::ChanReader>, Sender<bool>) {
    let (mut tx, rx) = channel();
    let reader = io::ChanReader::new(rx);

    let (ctl_tx, ctl_rx) = channel();

    spawn(proc() {
        let mut stream: *mut PaStream = ptr::mut_null();
        let frames_per_buffer: u32 = 32;

        // let (mut tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();
        let mut tx = tx.clone();
        let tx_ptr: *mut c_void = &mut tx as *mut _ as *mut c_void;

        unsafe {
            if input_device.is_none() {
                println!("[mic] using default device");
                let err = Pa_OpenDefaultStream(
                    &mut stream, 1, 1, paUInt8, 16000. as c_double, frames_per_buffer,
                    Some(stream_callback), tx_ptr);
                if err != paNoError {
                    print_err("error while opening stream", err);
                }
            } else {
                println!("[mic] using device #{}", input_device.unwrap());
                let in_params = PaStreamParameters {
                    device: input_device.unwrap() as i32,
                    sample_format: paUInt8,
                    channel_count: 1 as i32,
                    suggested_latency: 5. as f64,
                    host_api_specific_stream_info: ptr::mut_null()
                };

                let err = Pa_OpenStream(
                    &mut stream, &in_params, ptr::null(),
                    16000. as c_double, frames_per_buffer, 0,
                    Some(stream_callback), tx_ptr);
                if err != paNoError {
                    print_err("error while opening stream", err);
                }
            }
        }

        loop {
            println!("[mic] ready to recv");
            let r: Result<bool, ()> = ctl_rx.recv_opt();

            if r.is_err() {
                println!("[mic] done");
                break;
            }

            let x = r.unwrap();
            println!("[mic] recv'd {}", x);

            if x == true {
                unsafe {
                    let err = Pa_StartStream(stream);
                    if err != paNoError {
                        print_err("error while starting stream", err);
                    }
                };
            } else if x == false {
                unsafe {
                    let err = Pa_StopStream(stream);
                    if err != paNoError {
                        print_err("error while stopping stream", err);
                    }
                }
                break;
            }
        }
    });

    ctl_tx.send(true);

    (box reader, ctl_tx)
}

pub fn stop(tx: &Sender<bool>) {
    tx.send(false);
}

unsafe fn device_to_string(info: *const PaDeviceInfo) -> String {
    // device name
    let c_str_name = c_str::CString::new((*info).name, false);
    let name_opt = c_str_name.as_str();
    let name = name_opt.unwrap_or("none");

    // api name
    let api = Pa_GetHostApiInfo((*info).host_api);
    let c_str_api = c_str::CString::new((*api).name, false);
    let api_name_opt = c_str_api.as_str();
    let api_name = api_name_opt.unwrap_or("none");

    format!("\"{}\", host_api: \"{}\"", name, api_name)
}

pub fn list_devices() {
    let n_devices = unsafe { Pa_GetDeviceCount() };

    println!("[mic] detected {} devices", n_devices);

    for i in range(0, n_devices) {
        unsafe {
            let info = Pa_GetDeviceInfo(i);
            println!("[mic] device #{}: {}", i, device_to_string(info));
        }
    }

    unsafe {
        let def = Pa_GetDefaultInputDevice();
        let info = Pa_GetDeviceInfo(def);
        println!("[mic] using default device (#{}: {})", def, device_to_string(info));
    }
}

pub fn init (/*args: &[String]*/) {
    unsafe { Pa_Initialize() };
}

/*
fn test() {
    let (box reader, ctl) = mic_init();

    spawn(proc() {
        mic_start(&ctl);
        io::timer::sleep(time::duration::Duration::milliseconds(1500));
        mic_stop(&ctl);
    });

    let mut r = reader;
    let mut w = io::File::open_mode(&Path::new("/tmp/foo.raw"), io::Open, io::ReadWrite);
    io::util::copy(&mut r, &mut w);
}
*/
