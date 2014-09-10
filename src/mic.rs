extern crate libc;

use std::ptr::null;
use std::io;
use std::c_str::CString;
use self::libc::{c_char, c_uchar, c_int, c_uint, c_double, c_void, size_t, free};
use std::comm::{Empty, Disconnected};
use std::vec::Vec;

static SOX_MAX_NLOOPS: uint = 8;
static BUF_SIZE: uint = 100;

#[repr(C)]
#[deriving(Show)]
pub enum SoxErrorT {
  SOX_SUCCESS = 0,     /**< Function succeeded = 0 */
  SOX_EOF = -1,        /**< End Of File or other error = -1 */
  SOX_EHDR = 2000,     /**< Invalid Audio Header = 2000 */
  SOX_EFMT = 2001,     /**< Unsupported data format = 2001 */
  SOX_ENOMEM = 2002,   /**< Can't alloc memory = 2002 */
  SOX_EPERM = 2003,    /**< Operation not permitted = 2003 */
  SOX_ENOTSUP = 2004,  /**< Operation not supported = 2004 */
  SOX_EINVAL = 2005    /*< Invalid argument = 2005 */
}

#[repr(C)]
#[deriving(Show)]
pub enum SoxEncodingT {
    SOX_ENCODING_UNKNOWN    = 0,  /**< encoding has not yet been determined */

    SOX_ENCODING_SIGN2      = 1,  /**< signed linear 2's comp: Mac */
    SOX_ENCODING_UNSIGNED   = 2,  /**< unsigned linear: Sound Blaster */
    SOX_ENCODING_FLOAT      = 3,  /**< floating point (binary format) */
    SOX_ENCODING_FLOAT_TEXT = 4,  /**< floating point (text format) */
    SOX_ENCODING_FLAC       = 5,  /**< FLAC compression */
    SOX_ENCODING_HCOM       = 6,  /**< Mac FSSD files with Huffman compression */
    SOX_ENCODING_WAVPACK    = 7,  /**< WavPack with integer samples */
    SOX_ENCODING_WAVPACKF   = 8,  /**< WavPack with float samples */
    SOX_ENCODING_ULAW       = 9,  /**< u-law signed logs: US telephony, SPARC */
    SOX_ENCODING_ALAW       = 10, /**< A-law signed logs: non-US telephony, Psion */
    SOX_ENCODING_G721       = 11, /**< G.721 4-bit ADPCM */
    SOX_ENCODING_G723       = 12, /**< G.723 3 or 5 bit ADPCM */
    SOX_ENCODING_CL_ADPCM   = 13, /**< Creative Labs 8 --> 2,3,4 bit Compressed PCM */
    SOX_ENCODING_CL_ADPCM16 = 14, /**< Creative Labs 16 --> 4 bit Compressed PCM */
    SOX_ENCODING_MS_ADPCM   = 15, /**< Microsoft Compressed PCM */
    SOX_ENCODING_IMA_ADPCM  = 16, /**< IMA Compressed PCM */
    SOX_ENCODING_OKI_ADPCM  = 17, /**< Dialogic/OKI Compressed PCM */
    SOX_ENCODING_DPCM       = 18, /**< Differential PCM: Fasttracker 2 (xi) */
    SOX_ENCODING_DWVW       = 19, /**< Delta Width Variable Word */
    SOX_ENCODING_DWVWN      = 20, /**< Delta Width Variable Word N-bit */
    SOX_ENCODING_GSM        = 21, /**< GSM 6.10 33byte frame lossy compression */
    SOX_ENCODING_MP3        = 22, /**< MP3 compression */
    SOX_ENCODING_VORBIS     = 23, /**< Vorbis compression */
    SOX_ENCODING_AMR_WB     = 24, /**< AMR-WB compression */
    SOX_ENCODING_AMR_NB     = 25, /**< AMR-NB compression */
    SOX_ENCODING_CVSD       = 26, /**< Continuously Variable Slope Delta modulation */
    SOX_ENCODING_LPC10      = 27, /**< Linear Predictive Coding */
    SOX_ENCODING_OPUS       = 28, /**< Opus compression */

    SOX_ENCODINGS           = 29  /*< End of list marker */
}

#[repr(C)]
pub enum LsxIoType {
  LsxIoFile = 0,
  LsxIoPipe = 1,
  LsxIoUrl = 2
}

#[repr(C)]
#[deriving(Show)]
pub enum SoxBool {
    SoxFalse = 0,
    SoxTrue = 1
}

#[repr(C)]
pub enum SoxOptionT {
    SoxOptionNo = 0,
    SoxOptionYes = 1,
    SoxOptionDefault = 2
}

#[repr(C)]
pub enum SoxPlotT {
    SoxPlotOff = 0,
    SoxPlotOctave = 1,
    SoxPlotGnuplot = 2,
    SoxPlotData = 3
}

#[repr(C)]
pub struct SoxInstrInfoT {
    pub midi_note: c_char,
    pub midi_low: c_char,
    pub midi_hi: c_char,
    pub nloops: c_uchar,
    pub loops: c_uint
}

#[repr(C)]
pub struct SoxLoopInfoT {
    pub start: u64,
    pub length: u64,
    pub count: c_uint,
    pub _type: c_uchar
}

#[repr(C)]
pub struct SoxOobT {
    pub comments: *mut *const c_char,
    pub instr: SoxInstrInfoT,
    pub loops: [SoxLoopInfoT, ..SOX_MAX_NLOOPS]
}

#[repr(C)]
#[deriving(Clone)]
#[allow(raw_pointer_deriving)]
pub struct SoxSignalInfoT {
    pub rate: c_double,
    pub channels: c_uint,
    pub precision: c_uint,
    pub length: u64,
    pub mult: *mut c_double
}

#[repr(C)]
pub struct SoxEncodingInfoT {
    pub encoding: SoxEncodingT,
    pub bits_per_sample: c_uint,
    pub compression: c_double,
    pub reverse_bytes: SoxOptionT,
    pub reverse_nibbles: SoxOptionT,
    pub reverse_bits: SoxOptionT,
    pub opposite_endian: SoxBool
}

#[repr(C)]
pub struct SoxFormatHandlerT {
    pub sox_lib_version_code: c_uint,
    pub description: *const c_char,
    pub names: *const *const c_char,
    pub flags: c_uint,
    pub startread: *const c_void,
    pub read: *const c_void,
    pub stopread: *const c_void,
    pub startwrite: *const c_void,
    pub write: *const c_void,
    pub stopwrite: *const c_void,
    pub seek: *const c_void,
    pub write_formats: *const c_uint,
    pub write_rates: *const c_double,
    pub priv_size: size_t
}

#[repr(C)]
pub struct SoxFormatT {
    pub filename: *const c_char,
    pub signal: SoxSignalInfoT,
    pub encoding: SoxEncodingInfoT,
    pub filetype: *const c_char,
    pub oob: SoxOobT,
    pub seekable: SoxBool,
    pub mode: c_char,
    pub olength: u64,
    pub clips: u64,
    pub sox_errno: c_int,
    pub sox_errstr: [c_char, ..256],
    pub fp: *mut c_void,
    pub io_type: LsxIoType,
    pub tell_off: u64,
    pub data_start: u64,
    pub handler: SoxFormatHandlerT,
    pub _priv: *mut c_void
}

#[repr(C)]
pub struct SoxEffectHandlerT {
    pub name: *const c_char,
    pub usage: *const c_char,
    pub flags: c_uint,
    pub getopts: *mut c_void,
    pub start: *mut c_void,
    pub flow: *mut c_void,
    pub drain: *mut c_void,
    pub stop: *mut c_void,
    pub kill: *mut c_void,
    pub priv_size: size_t
}

#[repr(C)]
pub struct SoxGlobalsT {
    pub verbosity: c_uint,
    pub output_message_handler: *mut c_void,
    pub repeatable: SoxBool,
    pub bufsize: size_t,
    pub input_bufsiz: size_t,
    pub ranqd1: i32,
    pub stdin_in_use_by: *const c_char,
    pub stdout_in_use_by: *const c_char,
    pub subsystem: *const c_char,
    pub tmp_path: *const c_char,
    pub use_magic: SoxBool,
    pub use_threads: SoxBool
}

#[repr(C)]
pub struct SoxEffectsGlobalsT {
    pub plot: SoxPlotT,
    pub global_info: *mut SoxGlobalsT
}

#[repr(C)]
pub struct SoxEffectT {
    pub global_info: *mut SoxEffectsGlobalsT,
    pub in_signal: SoxSignalInfoT,
    pub out_signal: SoxSignalInfoT,
    pub in_encoding: *const SoxEncodingInfoT,
    pub out_encoding: *const SoxEncodingInfoT,
    pub handler: SoxEffectHandlerT,
    pub obuf: i32,
    pub obeg: size_t,
    pub oend: size_t,
    pub imin: size_t,
    pub clips: u64,
    pub flows: size_t,
    pub flow: size_t,
    pub _priv: *const c_void
}

#[repr(C)]
pub struct SoxEffectsChainT {
    pub effects: *mut *mut SoxEffectT,
    pub table_size: c_uint,
    pub length: c_uint,
    pub ibufc: *mut *mut i32,
    pub obufc: *mut *mut i32,
    pub global_info: SoxEffectsGlobalsT,
    pub in_enc: SoxEncodingInfoT,
    pub out_enc: SoxEncodingInfoT
}

#[link(name = "sox")]
extern {
    fn sox_version() -> *const c_char;
    fn sox_format_init() -> SoxErrorT;
    fn sox_open_read(
        path: *const c_char,
        signal: *const SoxSignalInfoT,
        encoding: *const SoxEncodingInfoT,
        filetype: *const c_char) -> *const SoxFormatT;
    fn sox_read(ft: *const SoxFormatT, buf: *const i32, len: size_t) -> size_t;
    fn sox_open_write(
        path: *const c_char,
        signal: *const SoxSignalInfoT,
        encoding: *const SoxEncodingInfoT,
        filetype: *const c_char,
        oob: *const SoxOobT,
        overwrite_permitted: *const c_void) -> *const SoxFormatT;
    fn sox_close(ft: *const SoxFormatT) -> SoxErrorT;
    fn sox_create_effects_chain(
        in_enc: *const SoxEncodingInfoT,
        out_enc: *const SoxEncodingInfoT) -> *mut SoxEffectsChainT;
    fn sox_find_effect(name: *const c_char) -> *const SoxEffectHandlerT;
    fn sox_create_effect(eh: *const SoxEffectHandlerT) -> *mut SoxEffectT;
    fn sox_effect_options(effp: *mut SoxEffectT, argc: c_int, argv: *const *const c_char) -> SoxErrorT;
    fn sox_add_effect(
        chain: *mut SoxEffectsChainT,
        effp: *mut SoxEffectT,
        _in: *mut SoxSignalInfoT,
        out: *const SoxSignalInfoT) -> SoxErrorT;
    fn sox_flow_effects(chain: *mut SoxEffectsChainT, callback: *const c_void, client_data: *const c_void) -> SoxErrorT;
    fn sox_quit() -> SoxErrorT;
}

pub struct MicContext {
    pub reader: Box<io::ChanReader>,
    pub sender: Sender<bool>,
    pub rate: u32,
    pub encoding: String,
    pub bits_per_sample: u32,
    pub is_big_endian: bool
}

pub fn is_big_endian() -> bool {
    return 1u16.to_be() == 1u16;
}

pub fn start(input_device: Option<String>) -> Option<MicContext> {

    let (mut tx, rx) = channel();
    let reader = io::ChanReader::new(rx);

    let (ctl_tx, ctl_rx) = channel();

    let path = input_device.unwrap_or("default".to_string()).to_c_str();
    let alsa = "alsa".to_c_str();
    let coreaudio = "coreaudio".to_c_str();

    let mut inputPtr = unsafe {sox_open_read(path.as_ptr(), null(), null(), alsa.as_ptr())};
    if inputPtr.is_null() {
        println!("Couldn't open input device using alsa. Trying with coreaudio...");
        inputPtr = unsafe {sox_open_read(path.as_ptr(), null(), null(), coreaudio.as_ptr())};
    }
    if inputPtr.is_null() {
        println!("Failed to open input device");
        return None;
    }

    let input = unsafe {*inputPtr};
    println!("Initialized recording device");
    println!("rate: {}, channels: {}, encoding: {}, bits_per_sample: {}, opposite_endian: {}",
        input.signal.rate,
        input.signal.channels,
        input.encoding.encoding,
        input.encoding.bits_per_sample,
        input.encoding.opposite_endian);


    spawn(proc() {
        loop {
            match ctl_rx.try_recv() {
                Ok(x) => {
                    println!("[mic] recv'd {}", x);
                    match x {
                        true => {
                            println!("[mic] Mmh?");
                        }
                        false => {
                            println!("[mic] stopping");
                            unsafe {sox_close(inputPtr)};
                            break;
                        }
                    }
                }
                Err(Empty) => {
                    let numChannels = input.signal.channels as uint;
                    let totalBytes = 4 * (BUF_SIZE - BUF_SIZE % numChannels);
                    let buf = Vec::from_elem(totalBytes, 0u8);
                    unsafe {sox_read(inputPtr, (&buf).as_ptr() as *const i32, BUF_SIZE as u64)};
                    //println!("Read: {}", buf);
                    let totalMonoBytes = totalBytes / numChannels;
                    let monobuf = Vec::from_fn(totalMonoBytes, |idx| {
                        buf[(idx / 4) * 4 * numChannels + (idx % 4)]
                    });
                    let result = tx.send_opt(monobuf);
                    if result.is_err() {
                        println!("[mic] error while sending: {}", result.err());
                    }
                }
                Err(Disconnected) => {
                    println!("[mic] done");
                    break;
                }
            }
        }
    });

    ctl_tx.send(true);

    let soxEncoding = input.encoding.encoding;
    let encodingOpt = match soxEncoding {
        SOX_ENCODING_SIGN2 => Some("signed-integer"),
        SOX_ENCODING_UNSIGNED => Some("unsigned-integer"),
        SOX_ENCODING_FLOAT => Some("floating-point"),
        SOX_ENCODING_ULAW => Some("ulaw"),
        SOX_ENCODING_ALAW => Some("alaw"),
        _ => None
    };
    if encodingOpt.is_none() {
        println!("[mic] unsupported encoding: {}", soxEncoding);
        return None
    }
    let is_big_endian = match input.encoding.opposite_endian {
        SoxFalse => is_big_endian(),
        SoxTrue => !is_big_endian()
    };
    Some(MicContext {
        reader: box reader,
        sender: ctl_tx,
        rate: input.signal.rate as u32,
        encoding: encodingOpt.unwrap().to_string(),
        bits_per_sample: input.encoding.bits_per_sample,
        is_big_endian: is_big_endian
    })
}

pub fn stop(tx: &Sender<bool>) {
    tx.send(false);
}

pub fn init (/*args: &[String]*/) {
    match unsafe {sox_format_init()} {
        SOX_SUCCESS => println!("[mic] initialized sox: {}", unsafe {CString::new(sox_version(), false)}),
        err => {
            println!("[mic] failed to initialize sox: {}", err);
            return;
        }
    };
}
