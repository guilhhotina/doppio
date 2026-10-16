use crate::ffi::{HwGetCurrentFramebufferFn, HwGetProcAddressFn, ProcAddressFn};
use std::ffi::{CStr, c_char, c_int, c_void};

const RTLD_NOW: c_int = 2;
const RTLD_GLOBAL: c_int = 0x100;

unsafe extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

struct Resolver {
    get: HwGetProcAddressFn,
    lib: Option<*mut c_void>,
}

impl Resolver {
    fn library() -> *mut c_void {
        [c"libGLESv2.so.2", c"libGLESv2.so"]
            .iter()
            .map(|name| unsafe { dlopen(name.as_ptr(), RTLD_NOW | RTLD_GLOBAL) })
            .find(|h| !h.is_null())
            .unwrap_or(std::ptr::null_mut())
    }

    fn find(&mut self, name: *const c_char) -> Option<ProcAddressFn> {
        unsafe { (self.get)(name) }.or_else(|| {
            let lib = *self.lib.get_or_insert_with(Resolver::library);
            let sym = (!lib.is_null()).then(|| unsafe { dlsym(lib, name) }).filter(|p| !p.is_null())?;
            Some(unsafe { std::mem::transmute::<*mut c_void, ProcAddressFn>(sym) })
        })
    }
}

pub type GLenum = u32;
pub type GLuint = u32;
pub type GLint = i32;
pub type GLsizei = i32;
pub type GLbitfield = u32;
pub type GLboolean = u8;
pub type GLfloat = f32;
pub type GLsizeiptr = isize;
pub type GLintptr = isize;

pub const NO_ERROR: GLenum = 0;
pub const ZERO: GLenum = 0;
pub const ONE: GLenum = 1;
pub const POINTS: GLenum = 0x0000;
pub const LINES: GLenum = 0x0001;
pub const TRIANGLES: GLenum = 0x0004;
pub const TRIANGLE_STRIP: GLenum = 0x0005;
pub const DEPTH_BUFFER_BIT: GLbitfield = 0x0100;
pub const COLOR_BUFFER_BIT: GLbitfield = 0x4000;
pub const NEVER: GLenum = 0x0200;
pub const LESS: GLenum = 0x0201;
pub const EQUAL: GLenum = 0x0202;
pub const LEQUAL: GLenum = 0x0203;
pub const ALWAYS: GLenum = 0x0207;
pub const SRC_ALPHA: GLenum = 0x0302;
pub const ONE_MINUS_SRC_ALPHA: GLenum = 0x0303;
pub const DST_COLOR: GLenum = 0x0306;
pub const FRONT: GLenum = 0x0404;
pub const BACK: GLenum = 0x0405;
pub const CW: GLenum = 0x0900;
pub const CCW: GLenum = 0x0901;
pub const CULL_FACE: GLenum = 0x0B44;
pub const DEPTH_TEST: GLenum = 0x0B71;
pub const BLEND: GLenum = 0x0BE2;
pub const SCISSOR_TEST: GLenum = 0x0C11;
pub const UNPACK_ALIGNMENT: GLenum = 0x0CF5;
pub const TEXTURE_2D: GLenum = 0x0DE1;
pub const BYTE: GLenum = 0x1400;
pub const UNSIGNED_BYTE: GLenum = 0x1401;
pub const SHORT: GLenum = 0x1402;
pub const UNSIGNED_SHORT: GLenum = 0x1403;
pub const FLOAT: GLenum = 0x1406;
pub const ALPHA: GLenum = 0x1906;
pub const RGB: GLenum = 0x1907;
pub const RGBA: GLenum = 0x1908;
pub const LUMINANCE: GLenum = 0x1909;
pub const VENDOR: GLenum = 0x1F00;
pub const RENDERER: GLenum = 0x1F01;
pub const VERSION: GLenum = 0x1F02;
pub const NEAREST: GLenum = 0x2600;
pub const LINEAR: GLenum = 0x2601;
pub const TEXTURE_MAG_FILTER: GLenum = 0x2800;
pub const TEXTURE_MIN_FILTER: GLenum = 0x2801;
pub const TEXTURE_WRAP_S: GLenum = 0x2802;
pub const TEXTURE_WRAP_T: GLenum = 0x2803;
pub const REPEAT: GLenum = 0x2901;
pub const UNSIGNED_SHORT_4_4_4_4: GLenum = 0x8033;
pub const UNSIGNED_SHORT_5_5_5_1: GLenum = 0x8034;
pub const CLAMP_TO_EDGE: GLenum = 0x812F;
pub const DEPTH_COMPONENT16: GLenum = 0x81A5;
pub const UNSIGNED_SHORT_5_6_5: GLenum = 0x8363;
pub const TEXTURE0: GLenum = 0x84C0;
pub const ARRAY_BUFFER: GLenum = 0x8892;
pub const ELEMENT_ARRAY_BUFFER: GLenum = 0x8893;
pub const STREAM_DRAW: GLenum = 0x88E0;
pub const STATIC_DRAW: GLenum = 0x88E4;
pub const DYNAMIC_DRAW: GLenum = 0x88E8;
pub const FRAGMENT_SHADER: GLenum = 0x8B30;
pub const VERTEX_SHADER: GLenum = 0x8B31;
pub const COMPILE_STATUS: GLenum = 0x8B81;
pub const LINK_STATUS: GLenum = 0x8B82;
pub const INFO_LOG_LENGTH: GLenum = 0x8B84;
pub const FRAMEBUFFER_COMPLETE: GLenum = 0x8CD5;
pub const COLOR_ATTACHMENT0: GLenum = 0x8CE0;
pub const DEPTH_ATTACHMENT: GLenum = 0x8D00;
pub const FRAMEBUFFER: GLenum = 0x8D40;
pub const RENDERBUFFER: GLenum = 0x8D41;

macro_rules! gl_fns {
    ($($name:ident: fn($($arg:ty),*) $(-> $ret:ty)?;)*) => {
        #[allow(non_snake_case)]
        #[derive(Clone, Copy)]
        struct Fns {
            $($name: unsafe extern "C" fn($($arg),*) $(-> $ret)?,)*
        }

        impl Fns {
            fn load(r: &mut Resolver) -> Result<Fns, &'static str> {
                Ok(Fns {
                    $($name: {
                        let f = r.find(concat!(stringify!($name), "\0").as_ptr().cast()).ok_or(stringify!($name))?;
                        unsafe {
                            std::mem::transmute::<ProcAddressFn, unsafe extern "C" fn($($arg),*) $(-> $ret)?>(f)
                        }
                    },)*
                })
            }
        }
    };
}

gl_fns! {
    glGetError: fn() -> GLenum;
    glGetString: fn(GLenum) -> *const u8;
    glEnable: fn(GLenum);
    glDisable: fn(GLenum);
    glViewport: fn(GLint, GLint, GLsizei, GLsizei);
    glScissor: fn(GLint, GLint, GLsizei, GLsizei);
    glClearColor: fn(GLfloat, GLfloat, GLfloat, GLfloat);
    glClearDepthf: fn(GLfloat);
    glClear: fn(GLbitfield);
    glDepthFunc: fn(GLenum);
    glDepthMask: fn(GLboolean);
    glColorMask: fn(GLboolean, GLboolean, GLboolean, GLboolean);
    glBlendFunc: fn(GLenum, GLenum);
    glCullFace: fn(GLenum);
    glFrontFace: fn(GLenum);
    glPixelStorei: fn(GLenum, GLint);
    glCreateShader: fn(GLenum) -> GLuint;
    glShaderSource: fn(GLuint, GLsizei, *const *const c_char, *const GLint);
    glCompileShader: fn(GLuint);
    glGetShaderiv: fn(GLuint, GLenum, *mut GLint);
    glGetShaderInfoLog: fn(GLuint, GLsizei, *mut GLsizei, *mut c_char);
    glDeleteShader: fn(GLuint);
    glCreateProgram: fn() -> GLuint;
    glAttachShader: fn(GLuint, GLuint);
    glBindAttribLocation: fn(GLuint, GLuint, *const c_char);
    glLinkProgram: fn(GLuint);
    glGetProgramiv: fn(GLuint, GLenum, *mut GLint);
    glGetProgramInfoLog: fn(GLuint, GLsizei, *mut GLsizei, *mut c_char);
    glUseProgram: fn(GLuint);
    glDeleteProgram: fn(GLuint);
    glGetUniformLocation: fn(GLuint, *const c_char) -> GLint;
    glUniform1i: fn(GLint, GLint);
    glUniform1f: fn(GLint, GLfloat);
    glUniform2f: fn(GLint, GLfloat, GLfloat);
    glUniform3f: fn(GLint, GLfloat, GLfloat, GLfloat);
    glUniform4f: fn(GLint, GLfloat, GLfloat, GLfloat, GLfloat);
    glUniformMatrix4fv: fn(GLint, GLsizei, GLboolean, *const GLfloat);
    glGenBuffers: fn(GLsizei, *mut GLuint);
    glBindBuffer: fn(GLenum, GLuint);
    glBufferData: fn(GLenum, GLsizeiptr, *const c_void, GLenum);
    glBufferSubData: fn(GLenum, GLintptr, GLsizeiptr, *const c_void);
    glDeleteBuffers: fn(GLsizei, *const GLuint);
    glEnableVertexAttribArray: fn(GLuint);
    glDisableVertexAttribArray: fn(GLuint);
    glVertexAttribPointer: fn(GLuint, GLint, GLenum, GLboolean, GLsizei, *const c_void);
    glActiveTexture: fn(GLenum);
    glGenTextures: fn(GLsizei, *mut GLuint);
    glBindTexture: fn(GLenum, GLuint);
    glTexImage2D: fn(GLenum, GLint, GLint, GLsizei, GLsizei, GLint, GLenum, GLenum, *const c_void);
    glTexSubImage2D: fn(GLenum, GLint, GLint, GLint, GLsizei, GLsizei, GLenum, GLenum, *const c_void);
    glTexParameteri: fn(GLenum, GLenum, GLint);
    glDeleteTextures: fn(GLsizei, *const GLuint);
    glGenFramebuffers: fn(GLsizei, *mut GLuint);
    glBindFramebuffer: fn(GLenum, GLuint);
    glFramebufferTexture2D: fn(GLenum, GLenum, GLenum, GLuint, GLint);
    glCheckFramebufferStatus: fn(GLenum) -> GLenum;
    glDeleteFramebuffers: fn(GLsizei, *const GLuint);
    glGenRenderbuffers: fn(GLsizei, *mut GLuint);
    glBindRenderbuffer: fn(GLenum, GLuint);
    glRenderbufferStorage: fn(GLenum, GLenum, GLsizei, GLsizei);
    glFramebufferRenderbuffer: fn(GLenum, GLenum, GLenum, GLuint);
    glDeleteRenderbuffers: fn(GLsizei, *const GLuint);
    glDrawArrays: fn(GLenum, GLint, GLsizei);
    glDrawElements: fn(GLenum, GLsizei, GLenum, *const c_void);
}

#[derive(Clone, Copy)]
pub struct Gl {
    f: Fns,
    target: HwGetCurrentFramebufferFn,
}

fn bytes<T: Copy>(data: &[T]) -> (GLsizeiptr, *const c_void) {
    (size_of_val(data) as GLsizeiptr, data.as_ptr().cast())
}

impl Gl {
    pub fn load(get: HwGetProcAddressFn, target: HwGetCurrentFramebufferFn) -> Result<Gl, &'static str> {
        Fns::load(&mut Resolver { get, lib: None }).map(|f| Gl { f, target })
    }

    pub fn target_framebuffer(&self) -> GLuint {
        unsafe { (self.target)() as GLuint }
    }

    pub fn bind_target(&self) {
        self.bind_framebuffer(self.target_framebuffer());
    }

    pub fn error(&self) -> GLenum {
        unsafe { (self.f.glGetError)() }
    }

    pub fn string(&self, name: GLenum) -> Option<&'static str> {
        let p = unsafe { (self.f.glGetString)(name) };
        (!p.is_null()).then(|| unsafe { CStr::from_ptr(p.cast()) }.to_str().ok()).flatten()
    }

    pub fn enable(&self, cap: GLenum) {
        unsafe { (self.f.glEnable)(cap) }
    }

    pub fn disable(&self, cap: GLenum) {
        unsafe { (self.f.glDisable)(cap) }
    }

    pub fn viewport(&self, x: i32, y: i32, w: i32, h: i32) {
        unsafe { (self.f.glViewport)(x, y, w, h) }
    }

    pub fn scissor(&self, x: i32, y: i32, w: i32, h: i32) {
        unsafe { (self.f.glScissor)(x, y, w, h) }
    }

    pub fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe { (self.f.glClearColor)(r, g, b, a) }
    }

    pub fn clear_depth(&self, d: f32) {
        unsafe { (self.f.glClearDepthf)(d) }
    }

    pub fn clear(&self, mask: GLbitfield) {
        unsafe { (self.f.glClear)(mask) }
    }

    pub fn depth_func(&self, func: GLenum) {
        unsafe { (self.f.glDepthFunc)(func) }
    }

    pub fn depth_mask(&self, write: bool) {
        unsafe { (self.f.glDepthMask)(write as GLboolean) }
    }

    pub fn color_mask(&self, r: bool, g: bool, b: bool, a: bool) {
        unsafe { (self.f.glColorMask)(r as GLboolean, g as GLboolean, b as GLboolean, a as GLboolean) }
    }

    pub fn blend_func(&self, src: GLenum, dst: GLenum) {
        unsafe { (self.f.glBlendFunc)(src, dst) }
    }

    pub fn cull_face(&self, mode: GLenum) {
        unsafe { (self.f.glCullFace)(mode) }
    }

    pub fn front_face(&self, mode: GLenum) {
        unsafe { (self.f.glFrontFace)(mode) }
    }

    pub fn pixel_store(&self, pname: GLenum, param: i32) {
        unsafe { (self.f.glPixelStorei)(pname, param) }
    }

    fn info_log(&self, id: GLuint, get_iv: unsafe extern "C" fn(GLuint, GLenum, *mut GLint), get_log: unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut c_char)) -> String {
        let mut len = 0;
        unsafe { get_iv(id, INFO_LOG_LENGTH, &mut len) };
        let mut buf = vec![0u8; len.max(1) as usize];
        let mut written = 0;
        unsafe { get_log(id, buf.len() as GLsizei, &mut written, buf.as_mut_ptr().cast()) };
        buf.truncate(written.max(0) as usize);
        String::from_utf8_lossy(&buf).into_owned()
    }

    fn shader(&self, kind: GLenum, src: &str) -> Result<GLuint, String> {
        let s = unsafe { (self.f.glCreateShader)(kind) };
        let ptr = src.as_ptr().cast::<c_char>();
        let len = src.len() as GLint;
        let mut ok = 0;
        unsafe {
            (self.f.glShaderSource)(s, 1, &ptr, &len);
            (self.f.glCompileShader)(s);
            (self.f.glGetShaderiv)(s, COMPILE_STATUS, &mut ok);
        }
        match ok {
            0 => {
                let log = self.info_log(s, self.f.glGetShaderiv, self.f.glGetShaderInfoLog);
                unsafe { (self.f.glDeleteShader)(s) };
                Err(log)
            }
            _ => Ok(s),
        }
    }

    pub fn program(&self, vs: &str, fs: &str, attribs: &[(GLuint, &CStr)]) -> Result<GLuint, String> {
        let v = self.shader(VERTEX_SHADER, vs).map_err(|e| format!("vertex shader: {e}"))?;
        let f = self
            .shader(FRAGMENT_SHADER, fs)
            .inspect_err(|_| unsafe { (self.f.glDeleteShader)(v) })
            .map_err(|e| format!("fragment shader: {e}"))?;
        let p = unsafe { (self.f.glCreateProgram)() };
        let mut ok = 0;
        unsafe {
            (self.f.glAttachShader)(p, v);
            (self.f.glAttachShader)(p, f);
            attribs.iter().for_each(|(i, name)| (self.f.glBindAttribLocation)(p, *i, name.as_ptr()));
            (self.f.glLinkProgram)(p);
            (self.f.glGetProgramiv)(p, LINK_STATUS, &mut ok);
            (self.f.glDeleteShader)(v);
            (self.f.glDeleteShader)(f);
        }
        match ok {
            0 => {
                let log = self.info_log(p, self.f.glGetProgramiv, self.f.glGetProgramInfoLog);
                unsafe { (self.f.glDeleteProgram)(p) };
                Err(format!("link: {log}"))
            }
            _ => Ok(p),
        }
    }

    pub fn use_program(&self, p: GLuint) {
        unsafe { (self.f.glUseProgram)(p) }
    }

    pub fn delete_program(&self, p: GLuint) {
        unsafe { (self.f.glDeleteProgram)(p) }
    }

    pub fn uniform_location(&self, p: GLuint, name: &CStr) -> GLint {
        unsafe { (self.f.glGetUniformLocation)(p, name.as_ptr()) }
    }

    pub fn uniform1i(&self, loc: GLint, v: i32) {
        unsafe { (self.f.glUniform1i)(loc, v) }
    }

    pub fn uniform1f(&self, loc: GLint, v: f32) {
        unsafe { (self.f.glUniform1f)(loc, v) }
    }

    pub fn uniform2f(&self, loc: GLint, x: f32, y: f32) {
        unsafe { (self.f.glUniform2f)(loc, x, y) }
    }

    pub fn uniform3f(&self, loc: GLint, x: f32, y: f32, z: f32) {
        unsafe { (self.f.glUniform3f)(loc, x, y, z) }
    }

    pub fn uniform4f(&self, loc: GLint, x: f32, y: f32, z: f32, w: f32) {
        unsafe { (self.f.glUniform4f)(loc, x, y, z, w) }
    }

    pub fn uniform_mat4(&self, loc: GLint, m: &[f32; 16]) {
        unsafe { (self.f.glUniformMatrix4fv)(loc, 1, 0, m.as_ptr()) }
    }

    pub fn gen_buffer(&self) -> GLuint {
        let mut b = 0;
        unsafe { (self.f.glGenBuffers)(1, &mut b) };
        b
    }

    pub fn bind_buffer(&self, target: GLenum, b: GLuint) {
        unsafe { (self.f.glBindBuffer)(target, b) }
    }

    pub fn buffer_data<T: Copy>(&self, target: GLenum, data: &[T], usage: GLenum) {
        let (len, ptr) = bytes(data);
        unsafe { (self.f.glBufferData)(target, len, ptr, usage) }
    }

    pub fn buffer_reserve(&self, target: GLenum, len: usize, usage: GLenum) {
        unsafe { (self.f.glBufferData)(target, len as GLsizeiptr, std::ptr::null(), usage) }
    }

    pub fn buffer_sub_data<T: Copy>(&self, target: GLenum, offset: usize, data: &[T]) {
        let (len, ptr) = bytes(data);
        unsafe { (self.f.glBufferSubData)(target, offset as GLintptr, len, ptr) }
    }

    pub fn delete_buffer(&self, b: GLuint) {
        unsafe { (self.f.glDeleteBuffers)(1, &b) }
    }

    pub fn enable_attrib(&self, index: GLuint) {
        unsafe { (self.f.glEnableVertexAttribArray)(index) }
    }

    pub fn disable_attrib(&self, index: GLuint) {
        unsafe { (self.f.glDisableVertexAttribArray)(index) }
    }

    pub fn attrib_pointer(&self, index: GLuint, size: i32, ty: GLenum, normalized: bool, stride: usize, offset: usize) {
        unsafe {
            (self.f.glVertexAttribPointer)(
                index,
                size,
                ty,
                normalized as GLboolean,
                stride as GLsizei,
                std::ptr::without_provenance(offset),
            )
        }
    }

    pub fn gen_texture(&self) -> GLuint {
        let mut t = 0;
        unsafe { (self.f.glGenTextures)(1, &mut t) };
        t
    }

    pub fn bind_texture(&self, unit: u32, t: GLuint) {
        unsafe {
            (self.f.glActiveTexture)(TEXTURE0 + unit);
            (self.f.glBindTexture)(TEXTURE_2D, t);
        }
    }

    pub fn tex_image<T: Copy>(&self, format: GLenum, w: i32, h: i32, ty: GLenum, data: Option<&[T]>) {
        let ptr = data.map_or(std::ptr::null(), |d| d.as_ptr().cast());
        unsafe { (self.f.glTexImage2D)(TEXTURE_2D, 0, format as GLint, w, h, 0, format, ty, ptr) }
    }

    pub fn tex_sub_image<T: Copy>(&self, x: i32, y: i32, w: i32, h: i32, format: GLenum, ty: GLenum, data: &[T]) {
        unsafe { (self.f.glTexSubImage2D)(TEXTURE_2D, 0, x, y, w, h, format, ty, data.as_ptr().cast()) }
    }

    pub fn tex_parameter(&self, pname: GLenum, param: GLenum) {
        unsafe { (self.f.glTexParameteri)(TEXTURE_2D, pname, param as GLint) }
    }

    pub fn delete_texture(&self, t: GLuint) {
        unsafe { (self.f.glDeleteTextures)(1, &t) }
    }

    pub fn gen_framebuffer(&self) -> GLuint {
        let mut f = 0;
        unsafe { (self.f.glGenFramebuffers)(1, &mut f) };
        f
    }

    pub fn bind_framebuffer(&self, f: GLuint) {
        unsafe { (self.f.glBindFramebuffer)(FRAMEBUFFER, f) }
    }

    pub fn framebuffer_texture(&self, attachment: GLenum, t: GLuint) {
        unsafe { (self.f.glFramebufferTexture2D)(FRAMEBUFFER, attachment, TEXTURE_2D, t, 0) }
    }

    pub fn framebuffer_status(&self) -> GLenum {
        unsafe { (self.f.glCheckFramebufferStatus)(FRAMEBUFFER) }
    }

    pub fn delete_framebuffer(&self, f: GLuint) {
        unsafe { (self.f.glDeleteFramebuffers)(1, &f) }
    }

    pub fn gen_renderbuffer(&self, format: GLenum, w: i32, h: i32) -> GLuint {
        let mut r = 0;
        unsafe {
            (self.f.glGenRenderbuffers)(1, &mut r);
            (self.f.glBindRenderbuffer)(RENDERBUFFER, r);
            (self.f.glRenderbufferStorage)(RENDERBUFFER, format, w, h);
            (self.f.glBindRenderbuffer)(RENDERBUFFER, 0);
        }
        r
    }

    pub fn framebuffer_renderbuffer(&self, attachment: GLenum, r: GLuint) {
        unsafe { (self.f.glFramebufferRenderbuffer)(FRAMEBUFFER, attachment, RENDERBUFFER, r) }
    }

    pub fn delete_renderbuffer(&self, r: GLuint) {
        unsafe { (self.f.glDeleteRenderbuffers)(1, &r) }
    }

    pub fn draw_arrays(&self, mode: GLenum, first: i32, count: i32) {
        unsafe { (self.f.glDrawArrays)(mode, first, count) }
    }

    pub fn draw_elements_u16(&self, mode: GLenum, count: i32, offset: usize) {
        unsafe { (self.f.glDrawElements)(mode, count, UNSIGNED_SHORT, std::ptr::without_provenance(offset)) }
    }
}
