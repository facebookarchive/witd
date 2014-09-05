extern crate libc;

use std::{c_vec, ptr, mem, io, sync, time};
use self::libc::{c_void, c_int, c_long, c_double, size_t, malloc};

type PaDeviceIndex = c_int;
type PaError = c_int;
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
pub type PaStreamCallbackFlags = u64;
type PaStreamCallback =
    extern "C" fn(*const c_void, *mut c_void, u32,
                  *const PaStreamCallbackTimeInfo,
                  PaStreamCallbackFlags, *mut c_void) -> PaStreamCallbackResult;

#[link(name = "portaudio")]
extern {
    fn Pa_Initialize() -> c_void;
    fn Pa_GetDefaultInputDevice() -> PaDeviceIndex; // PaDeviceIndex
    fn Pa_OpenDefaultStream(
        stream: *mut *mut PaStream, numInputChannels: c_int, numOutputChannels: c_int,
        sampleFormat: PaSampleFormat, sampleRate: c_double, framesPerBuffer: u32,
        streamCallBack: Option<PaStreamCallback>, userData: *mut c_void)
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
         println!("rx {} frames", frame_count);

         let c_bytes: c_vec::CVec<u8> = unsafe {
             c_vec::CVec::new(input as *mut u8, frame_count as uint)
         };
         let bytes: Vec<u8> = c_bytes.as_slice().to_vec();

         let tx: &mut Sender<Vec<u8>> = unsafe {
             &mut *(data as *mut Sender<Vec<u8>>)
         };

         println!("tx addr: {:p}, bytes len: {}", tx, bytes.len())
         let result = tx.send_opt(bytes);
         if result.is_err() {
             println!("error while sending: {}", result.err());
         }

         PaContinue
     }

fn job(wave_tx: Sender<Vec<u8>>, mic_rx: Receiver<bool>) {
    let mut stream: *mut PaStream = ptr::mut_null();
    let frames_per_buffer: u32 = 32;
    
    let mut tx = wave_tx.clone();
    let tx_ptr: *mut c_void = &mut tx as *mut _ as *mut c_void;
    
    unsafe {
        let err = Pa_OpenDefaultStream(
            &mut stream, 1, 1, paUInt8, 16000. as c_double, frames_per_buffer,
            Some(stream_callback), tx_ptr);
        if err != paNoError {
            println!("error while opening stream: {}", err);
        }
    }
    
    loop {
        println!("mic agent: ready to recv");
        let r: Result<bool, ()> = mic_rx.recv_opt();
        
        if r.is_err() {
            println!("mic agent: done");
            break;
        }
        
        let x = r.unwrap();
        println!("mic agent: recv'd {}", x);
        
        if x == true {
            unsafe {
                let err = Pa_StartStream(stream);
                    if err != paNoError {
                        println!("error while starting stream: {}", err);
                    }
            };
        } else if x == false {
            unsafe {
                let err = Pa_StopStream(stream);
                if err != paNoError {
                    println!("error while stopping stream: {}", err);
                }
            }
        }
    }
}

pub fn init (wave_tx: Sender<Vec<u8>>) -> (Sender<bool>) {
    unsafe { Pa_Initialize() };

    let (mic_tx, mic_rx) = channel();

    // spawn actor
    spawn(proc() {
        job(wave_tx, mic_rx);
    });

    return mic_tx;
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