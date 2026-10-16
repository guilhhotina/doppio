use std::ffi::{c_char, c_int, c_uint, c_void};

pub const API_VERSION: c_uint = 1;
pub const REGION_NTSC: c_uint = 0;

pub const DEVICE_JOYPAD: c_uint = 1;
pub const JOYPAD_B: c_uint = 0;
pub const JOYPAD_Y: c_uint = 1;
pub const JOYPAD_SELECT: c_uint = 2;
pub const JOYPAD_START: c_uint = 3;
pub const JOYPAD_UP: c_uint = 4;
pub const JOYPAD_DOWN: c_uint = 5;
pub const JOYPAD_LEFT: c_uint = 6;
pub const JOYPAD_RIGHT: c_uint = 7;
pub const JOYPAD_A: c_uint = 8;
pub const JOYPAD_X: c_uint = 9;
pub const JOYPAD_BUTTONS: c_uint = 10;

pub const ENV_EXPERIMENTAL: c_uint = 0x10000;
pub const ENV_PRIVATE: c_uint = 0x20000;
pub const ENV_SET_PIXEL_FORMAT: c_uint = 10;
pub const ENV_SET_HW_RENDER: c_uint = 14;
pub const ENV_GET_VARIABLE: c_uint = 15;
pub const ENV_SET_VARIABLES: c_uint = 16;
pub const ENV_SET_SUPPORT_NO_GAME: c_uint = 18;
pub const ENV_GET_LOG_INTERFACE: c_uint = 27;
pub const ENV_GET_USERNAME: c_uint = 38;
pub const ENV_SET_NETPACKET_INTERFACE: c_uint = 78;

pub const PIXEL_FORMAT_XRGB8888: c_int = 1;
pub const HW_CONTEXT_OPENGLES2: c_int = 2;

pub const LOG_DEBUG: c_int = 0;
pub const LOG_INFO: c_int = 1;
pub const LOG_WARN: c_int = 2;
pub const LOG_ERROR: c_int = 3;

pub const fn hw_frame_buffer_valid() -> *const c_void {
    std::ptr::without_provenance(usize::MAX)
}

pub type EnvironmentFn = unsafe extern "C" fn(cmd: c_uint, data: *mut c_void) -> bool;
pub type VideoRefreshFn = unsafe extern "C" fn(data: *const c_void, width: c_uint, height: c_uint, pitch: usize);
pub type AudioSampleFn = unsafe extern "C" fn(left: i16, right: i16);
pub type AudioSampleBatchFn = unsafe extern "C" fn(data: *const i16, frames: usize) -> usize;
pub type InputPollFn = unsafe extern "C" fn();
pub type InputStateFn = unsafe extern "C" fn(port: c_uint, device: c_uint, index: c_uint, id: c_uint) -> i16;
pub type LogPrintfFn = unsafe extern "C" fn(level: c_int, fmt: *const c_char, ...);
pub type HwContextResetFn = unsafe extern "C" fn();
pub type HwGetCurrentFramebufferFn = unsafe extern "C" fn() -> usize;
pub type ProcAddressFn = unsafe extern "C" fn();
pub type HwGetProcAddressFn = unsafe extern "C" fn(sym: *const c_char) -> Option<ProcAddressFn>;

#[repr(C)]
pub struct SystemInfo {
    pub library_name: *const c_char,
    pub library_version: *const c_char,
    pub valid_extensions: *const c_char,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

#[repr(C)]
pub struct GameGeometry {
    pub base_width: c_uint,
    pub base_height: c_uint,
    pub max_width: c_uint,
    pub max_height: c_uint,
    pub aspect_ratio: f32,
}

#[repr(C)]
pub struct SystemTiming {
    pub fps: f64,
    pub sample_rate: f64,
}

#[repr(C)]
pub struct SystemAvInfo {
    pub geometry: GameGeometry,
    pub timing: SystemTiming,
}

#[repr(C)]
pub struct GameInfo {
    pub path: *const c_char,
    pub data: *const c_void,
    pub size: usize,
    pub meta: *const c_char,
}

#[repr(C)]
pub struct Variable {
    pub key: *const c_char,
    pub value: *const c_char,
}

#[repr(C)]
pub struct LogCallback {
    pub log: Option<LogPrintfFn>,
}

#[repr(C)]
pub struct HwRenderCallback {
    pub context_type: c_int,
    pub context_reset: Option<HwContextResetFn>,
    pub get_current_framebuffer: Option<HwGetCurrentFramebufferFn>,
    pub get_proc_address: Option<HwGetProcAddressFn>,
    pub depth: bool,
    pub stencil: bool,
    pub bottom_left_origin: bool,
    pub version_major: c_uint,
    pub version_minor: c_uint,
    pub cache_context: bool,
    pub context_destroy: Option<HwContextResetFn>,
    pub debug_context: bool,
}

pub const NETPACKET_UNRELIABLE: c_int = 0;
pub const NETPACKET_RELIABLE: c_int = 1 << 0;
pub const NETPACKET_UNSEQUENCED: c_int = 1 << 1;
pub const NETPACKET_FLUSH_HINT: c_int = 1 << 2;
pub const NETPACKET_BROADCAST: u16 = 0xFFFF;

pub type NetpacketSendFn = unsafe extern "C" fn(flags: c_int, buf: *const c_void, len: usize, client_id: u16);
pub type NetpacketPollReceiveFn = unsafe extern "C" fn();
pub type NetpacketStartFn =
    unsafe extern "C" fn(client_id: u16, send: Option<NetpacketSendFn>, poll_receive: Option<NetpacketPollReceiveFn>);
pub type NetpacketReceiveFn = unsafe extern "C" fn(buf: *const c_void, len: usize, client_id: u16);
pub type NetpacketStopFn = unsafe extern "C" fn();
pub type NetpacketPollFn = unsafe extern "C" fn();
pub type NetpacketConnectedFn = unsafe extern "C" fn(client_id: u16) -> bool;
pub type NetpacketDisconnectedFn = unsafe extern "C" fn(client_id: u16);

#[repr(C)]
pub struct NetpacketCallback {
    pub start: Option<NetpacketStartFn>,
    pub receive: Option<NetpacketReceiveFn>,
    pub stop: Option<NetpacketStopFn>,
    pub poll: Option<NetpacketPollFn>,
    pub connected: Option<NetpacketConnectedFn>,
    pub disconnected: Option<NetpacketDisconnectedFn>,
    pub protocol_version: *const c_char,
}

pub const GECND_NETPLAY_GET_LOBBY: c_uint = ENV_PRIVATE | 0x4701;
pub const GECND_NETPLAY_POST_LOBBY: c_uint = ENV_PRIVATE | 0x4702;
pub const GECND_NETPLAY_CONNECT: c_uint = ENV_PRIVATE | 0x4703;
pub const GECND_NETPLAY_DISCONNECT: c_uint = ENV_PRIVATE | 0x4704;

pub const GECND_LOBBY_NONE: c_int = 0;
pub const GECND_LOBBY_LOADING: c_int = 1;
pub const GECND_LOBBY_READY: c_int = 2;
pub const GECND_LOBBY_FAILED: c_int = 3;

#[repr(C)]
pub struct GecndLobby {
    pub refresh: bool,
    pub state: c_int,
    pub json: *const c_char,
    pub size: usize,
}

#[repr(C)]
pub struct GecndRoom {
    pub game_name: *const c_char,
    pub port: u16,
    pub listed: bool,
    pub relay: *const c_char,
}

#[repr(C)]
pub struct GecndJoin {
    pub host: *const c_char,
    pub port: u16,
    pub mitm_session: *const c_char,
}

const P: usize = size_of::<usize>();
const fn by_width(w64: usize, w32: usize) -> usize {
    if P == 8 { w64 } else { w32 }
}

const _: () = {
    use std::mem::offset_of;
    assert!(size_of::<SystemInfo>() == by_width(32, 16));
    assert!(offset_of!(SystemInfo, need_fullpath) == 3 * P);
    assert!(size_of::<GameGeometry>() == 20);
    assert!(offset_of!(SystemAvInfo, timing) == 24);
    assert!(size_of::<SystemAvInfo>() == 40);
    assert!(size_of::<GameInfo>() == 4 * P);
    assert!(size_of::<Variable>() == 2 * P);
    assert!(size_of::<LogCallback>() == P);
    assert!(offset_of!(HwRenderCallback, context_reset) == P);
    assert!(offset_of!(HwRenderCallback, get_proc_address) == 3 * P);
    assert!(offset_of!(HwRenderCallback, depth) == 4 * P);
    assert!(offset_of!(HwRenderCallback, bottom_left_origin) == 4 * P + 2);
    assert!(offset_of!(HwRenderCallback, version_major) == 4 * P + 4);
    assert!(offset_of!(HwRenderCallback, cache_context) == 4 * P + 12);
    assert!(offset_of!(HwRenderCallback, context_destroy) == by_width(48, 32));
    assert!(offset_of!(HwRenderCallback, debug_context) == by_width(56, 36));
    assert!(size_of::<HwRenderCallback>() == by_width(64, 40));
    assert!(size_of::<NetpacketCallback>() == 7 * P);
    assert!(offset_of!(NetpacketCallback, protocol_version) == 6 * P);
    assert!(offset_of!(GecndLobby, state) == 4);
    assert!(offset_of!(GecndLobby, json) == by_width(8, 8));
    assert!(size_of::<GecndLobby>() == by_width(24, 16));
    assert!(offset_of!(GecndRoom, listed) == P + 2);
    assert!(offset_of!(GecndRoom, relay) == by_width(16, 8));
    assert!(size_of::<GecndRoom>() == by_width(24, 12));
    assert!(offset_of!(GecndJoin, mitm_session) == by_width(16, 8));
    assert!(size_of::<GecndJoin>() == by_width(24, 12));
};
