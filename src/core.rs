use crate::app::App;
use crate::ffi::*;
use crate::gl::Gl;
use std::cell::{Cell, RefCell};
use std::ffi::{CStr, CString, c_char, c_int, c_uint, c_void};
use std::ptr::null;

pub const WIDTH: u32 = 1152;
pub const HEIGHT: u32 = 648;
pub const FRAME_RATE: u64 = 60;
pub const FPS: f64 = FRAME_RATE as f64;
pub const SAMPLE_RATE: u32 = 32000;
const AUDIO_MAX_FRAMES: usize = SAMPLE_RATE as usize / FRAME_RATE as usize + 1;

#[derive(Clone, Copy)]
pub struct Frontend {
    env: Option<EnvironmentFn>,
    video: Option<VideoRefreshFn>,
    audio: Option<AudioSampleBatchFn>,
    input_poll: Option<InputPollFn>,
    input_state: Option<InputStateFn>,
    log: Option<LogPrintfFn>,
}

impl Frontend {
    const NONE: Frontend =
        Frontend { env: None, video: None, audio: None, input_poll: None, input_state: None, log: None };

    pub fn env<T>(&self, cmd: c_uint, data: &mut T) -> bool {
        self.env.is_some_and(|f| unsafe { f(cmd, (data as *mut T).cast()) })
    }

    pub fn probe(&self, cmd: c_uint) -> bool {
        self.env.is_some_and(|f| unsafe { f(cmd, std::ptr::null_mut()) })
    }

    pub fn log(&self, level: c_int, msg: &str) {
        match (self.log, CString::new(msg.replace('\0', " "))) {
            (Some(f), Ok(c)) => unsafe { f(level, c"doppio: %s\n".as_ptr(), c.as_ptr()) },
            _ => eprintln!("doppio: {msg}"),
        }
    }

    pub fn variable(&self, key: &CStr) -> Option<String> {
        let mut v = Variable { key: key.as_ptr(), value: null() };
        (self.env(ENV_GET_VARIABLE, &mut v) && !v.value.is_null())
            .then(|| unsafe { CStr::from_ptr(v.value) }.to_string_lossy().into_owned())
    }

    pub fn username(&self) -> Option<String> {
        let mut name: *const c_char = null();
        (self.env(ENV_GET_USERNAME, &mut name) && !name.is_null())
            .then(|| unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned())
    }

    pub fn buttons(&self, port: c_uint) -> u16 {
        self.input_state.map_or(0, |f| {
            (0..JOYPAD_BUTTONS)
                .filter(|&id| unsafe { f(port, DEVICE_JOYPAD, 0, id) } != 0)
                .fold(0, |mask, id| mask | 1 << id)
        })
    }

    fn poll_input(&self) {
        self.input_poll.iter().for_each(|f| unsafe { f() });
    }

    fn present(&self, drawn: bool) {
        let frame = if drawn { hw_frame_buffer_valid() } else { null() };
        self.video.iter().for_each(|f| unsafe { f(frame, WIDTH, HEIGHT, 0) });
    }

    fn push_audio(&self, stereo: &[i16]) {
        self.audio.filter(|_| !stereo.is_empty()).iter().for_each(|f| unsafe {
            f(stereo.as_ptr(), stereo.len() / 2);
        });
    }
}

struct Audio {
    frames: u64,
    sent: u64,
    buf: [i16; AUDIO_MAX_FRAMES * 2],
}

impl Audio {
    const NEW: Audio = Audio { frames: 0, sent: 0, buf: [0; AUDIO_MAX_FRAMES * 2] };

    fn due(&mut self) -> usize {
        self.frames += 1;
        let target = self.frames * SAMPLE_RATE as u64 / FRAME_RATE;
        let n = target - self.sent;
        self.sent = target;
        n as usize
    }
}

struct Core {
    fe: Cell<Frontend>,
    hw: Cell<Option<(HwGetProcAddressFn, HwGetCurrentFramebufferFn)>>,
    app: RefCell<Option<App>>,
    audio: RefCell<Audio>,
}

unsafe impl Sync for Core {}

static CORE: Core = Core {
    fe: Cell::new(Frontend::NONE),
    hw: Cell::new(None),
    app: RefCell::new(None),
    audio: RefCell::new(Audio::NEW),
};

pub fn frontend() -> Frontend {
    CORE.fe.get()
}

fn update(f: impl FnOnce(&mut Frontend)) {
    let mut fe = CORE.fe.get();
    f(&mut fe);
    CORE.fe.set(fe);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_environment(cb: Option<EnvironmentFn>) {
    update(|fe| fe.env = cb);
    let fe = CORE.fe.get();
    let mut log = LogCallback { log: None };
    if fe.env(ENV_GET_LOG_INTERFACE, &mut log) {
        update(|fe| fe.log = log.log);
    }
    let mut no_game = true;
    fe.env(ENV_SET_SUPPORT_NO_GAME, &mut no_game);
    let mut vars = [
        Variable { key: c"doppio_night_seconds".as_ptr(), value: c"Night length; 360|120|180|240|480|600".as_ptr() },
        Variable { key: null(), value: null() },
    ];
    fe.env(ENV_SET_VARIABLES, &mut vars);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_video_refresh(cb: Option<VideoRefreshFn>) {
    update(|fe| fe.video = cb);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_audio_sample(_cb: Option<AudioSampleFn>) {}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_audio_sample_batch(cb: Option<AudioSampleBatchFn>) {
    update(|fe| fe.audio = cb);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_input_poll(cb: Option<InputPollFn>) {
    update(|fe| fe.input_poll = cb);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_input_state(cb: Option<InputStateFn>) {
    update(|fe| fe.input_state = cb);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_init() {}

#[unsafe(no_mangle)]
pub extern "C" fn retro_deinit() {
    CORE.app.replace(None);
    CORE.hw.set(None);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_api_version() -> c_uint {
    API_VERSION
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_get_system_info(info: *mut SystemInfo) {
    let version = concat!(env!("CARGO_PKG_VERSION"), "\0");
    unsafe {
        info.write(SystemInfo {
            library_name: c"doppio".as_ptr(),
            library_version: version.as_ptr().cast(),
            valid_extensions: c"".as_ptr(),
            need_fullpath: false,
            block_extract: false,
        })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_get_system_av_info(info: *mut SystemAvInfo) {
    unsafe {
        info.write(SystemAvInfo {
            geometry: GameGeometry {
                base_width: WIDTH,
                base_height: HEIGHT,
                max_width: WIDTH,
                max_height: HEIGHT,
                aspect_ratio: WIDTH as f32 / HEIGHT as f32,
            },
            timing: SystemTiming { fps: FPS, sample_rate: SAMPLE_RATE as f64 },
        })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_controller_port_device(_port: c_uint, _device: c_uint) {}

#[unsafe(no_mangle)]
pub extern "C" fn retro_reset() {
    CORE.app.borrow_mut().iter_mut().for_each(App::reset);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_run() {
    let fe = CORE.fe.get();
    fe.poll_input();
    let input = [0, 1, 2, 3].map(|port| fe.buttons(port));
    let drawn = CORE.app.borrow_mut().as_mut().is_some_and(|app| app.frame(&fe, input));
    fe.present(drawn);
    let mut audio = CORE.audio.borrow_mut();
    let n = audio.due();
    let out = &mut audio.buf[..n * 2];
    match CORE.app.borrow_mut().as_mut() {
        Some(app) => app.audio(out),
        None => out.fill(0),
    }
    fe.push_audio(out);
}

unsafe extern "C" fn context_reset() {
    let fe = CORE.fe.get();
    let gl = CORE
        .hw
        .get()
        .ok_or_else(|| "context_reset before SET_HW_RENDER".to_string())
        .and_then(|(proc, fbo)| {
            Gl::load(proc, fbo).map_err(|name| format!("GLES2 entry point {name} not found via get_proc_address nor libGLESv2"))
        });
    match (gl, CORE.app.borrow_mut().as_mut()) {
        (Ok(gl), Some(app)) => app.context_reset(&fe, gl),
        (Err(e), _) => fe.log(LOG_ERROR, &e),
        (Ok(_), None) => fe.log(LOG_WARN, "context_reset without a loaded game"),
    }
}

unsafe extern "C" fn context_destroy() {
    CORE.app.borrow_mut().iter_mut().for_each(App::context_destroy);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_load_game(_game: *const GameInfo) -> bool {
    let fe = CORE.fe.get();
    let mut format = PIXEL_FORMAT_XRGB8888;
    if !fe.env(ENV_SET_PIXEL_FORMAT, &mut format) {
        fe.log(LOG_WARN, "frontend refused XRGB8888 pixel format");
    }
    let mut hw = HwRenderCallback {
        context_type: HW_CONTEXT_OPENGLES2,
        context_reset: Some(context_reset),
        get_current_framebuffer: None,
        get_proc_address: None,
        depth: true,
        stencil: false,
        bottom_left_origin: true,
        version_major: 2,
        version_minor: 0,
        cache_context: false,
        context_destroy: Some(context_destroy),
        debug_context: false,
    };
    if !fe.env(ENV_SET_HW_RENDER, &mut hw) {
        fe.log(LOG_ERROR, "frontend refused SET_HW_RENDER (OpenGL ES 2.0 required); cannot start");
        return false;
    }
    match hw.get_proc_address.zip(hw.get_current_framebuffer) {
        Some(callbacks) => CORE.hw.set(Some(callbacks)),
        None => {
            fe.log(LOG_ERROR, "SET_HW_RENDER accepted but get_proc_address/get_current_framebuffer missing");
            return false;
        }
    }
    *CORE.audio.borrow_mut() = Audio::NEW;
    CORE.app.replace(Some(App::new(&fe)));
    true
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_load_game_special(_type: c_uint, _info: *const GameInfo, _num: usize) -> bool {
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_unload_game() {
    CORE.app.replace(None);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_region() -> c_uint {
    REGION_NTSC
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_serialize_size() -> usize {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_serialize(_data: *mut c_void, _size: usize) -> bool {
    false
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_unserialize(_data: *const c_void, _size: usize) -> bool {
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_cheat_reset() {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_cheat_set(_index: c_uint, _enabled: bool, _code: *const c_char) {}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_memory_data(_id: c_uint) -> *mut c_void {
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_memory_size(_id: c_uint) -> usize {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_paces_exact_rate_per_frame() {
        let mut a = Audio::NEW;
        let counts: Vec<usize> = (0..FRAME_RATE * 3).map(|_| a.due()).collect();
        assert_eq!(counts.iter().sum::<usize>(), SAMPLE_RATE as usize * 3);
        assert!(counts.iter().all(|&n| (AUDIO_MAX_FRAMES - 2..=AUDIO_MAX_FRAMES).contains(&n)));
    }
}
