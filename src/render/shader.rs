use crate::gl::Gl;
use std::ffi::CStr;

pub const MAX_LIGHTS: usize = 4;

const WORLD_VS: &str = "attribute vec3 a_pos;
attribute vec2 a_uv;
attribute vec4 a_nrm;
uniform mat4 u_mvp;
uniform mat4 u_model;
uniform vec3 u_lamp;
uniform vec3 u_amb;
uniform vec4 u_lpos[4];
uniform vec4 u_ldir[4];
uniform vec3 u_eye;
uniform vec2 u_fog;
uniform vec2 u_snap;
varying vec3 v_uvq;
varying vec3 v_col;
varying float v_fog;
void main() {
    vec3 wp = (u_model * vec4(a_pos, 1.0)).xyz;
    vec3 n = normalize((u_model * vec4(a_nrm.xyz, 0.0)).xyz);
    vec3 c = u_amb + u_lamp * a_nrm.w;
    for (int i = 0; i < 4; i++) {
        vec3 d = u_lpos[i].xyz - wp;
        float dd = dot(d, d);
        vec3 l = d * inversesqrt(dd + 0.0001);
        float cone = smoothstep(u_ldir[i].w, u_ldir[i].w + 0.1, dot(-l, u_ldir[i].xyz));
        float wrap = max(dot(n, l), 0.0) * 0.8 + 0.2;
        c += vec3(1.0, 0.88, 0.66) * (u_lpos[i].w * cone * wrap / (1.0 + dd * 0.16));
    }
    v_col = c;
    vec4 p = u_mvp * vec4(a_pos, 1.0);
    if (p.w > 0.05) {
        p.xy = floor(p.xy / p.w * u_snap + 0.5) / u_snap * p.w;
    }
    v_fog = clamp((length(wp - u_eye) - u_fog.x) * u_fog.y, 0.0, 1.0);
    float q = p.w * 0.05;
    v_uvq = vec3(a_uv * (q / 2048.0), q);
    gl_Position = p;
}
";

const WORLD_FS: &str = "precision mediump float;
uniform sampler2D u_tex;
uniform sampler2D u_dither;
uniform vec3 u_fogc;
uniform vec4 u_fx;
varying vec3 v_uvq;
varying vec3 v_col;
varying float v_fog;
void main() {
    vec2 uv = v_uvq.xy / v_uvq.z;
#ifdef SCREEN
    float s = texture2D(u_tex, uv * vec2(0.7, 0.5) + u_fx.xy).r;
    float line = fract(gl_FragCoord.y * 0.5) * 0.25 + 0.75;
    vec3 c = vec3(0.78, 0.86, 1.0) * (s * u_fx.z * line + u_fx.w) + v_col * 0.04;
#else
    vec3 c = texture2D(u_tex, uv).rgb * v_col;
#endif
    c = mix(c, u_fogc, v_fog);
    float d = texture2D(u_dither, gl_FragCoord.xy * 0.25).r;
    gl_FragColor = vec4(floor(c * 31.0 + d) * (1.0 / 31.0), 1.0);
}
";

const GLOW_VS: &str = "attribute vec3 a_pos;
attribute vec2 a_uv;
attribute vec4 a_nrm;
uniform mat4 u_mvp;
uniform mat4 u_model;
uniform vec3 u_eye;
uniform vec2 u_snap;
varying float v_a;
void main() {
    vec4 p = u_mvp * vec4(a_pos, 1.0);
    if (p.w > 0.05) {
        p.xy = floor(p.xy / p.w * u_snap + 0.5) / u_snap * p.w;
    }
    vec3 wp = (u_model * vec4(a_pos, 1.0)).xyz;
    vec3 n = normalize((u_model * vec4(a_nrm.xyz, 0.0)).xyz);
    float f = abs(dot(n, normalize(u_eye - wp)));
    v_a = a_uv.x * (1.0 / 2048.0) * f * f;
    gl_Position = p;
}
";

const GLOW_FS: &str = "precision mediump float;
uniform sampler2D u_dither;
uniform vec3 u_color;
varying float v_a;
void main() {
    float d = texture2D(u_dither, gl_FragCoord.xy * 0.25).r;
    gl_FragColor = vec4(floor(u_color * v_a * 31.0 + d) * (1.0 / 31.0), 1.0);
}
";

const BLIT_VS: &str = "attribute vec2 a_pos;
uniform vec4 u_rect;
uniform vec2 u_canvas;
varying vec2 v_uv;
void main() {
    vec2 p = mix(u_rect.xy, u_rect.zw, a_pos) / u_canvas;
    v_uv = vec2(p.x, 1.0 - p.y);
    gl_Position = vec4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0);
}
";

const BLIT_FS: &str = "precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
void main() {
    gl_FragColor = texture2D(u_tex, v_uv);
}
";

const BLIT_FX_FS: &str = "precision mediump float;
uniform sampler2D u_tex;
uniform sampler2D u_noise;
uniform vec3 u_tint;
uniform vec4 u_grain;
varying vec2 v_uv;
void main() {
    vec3 c = texture2D(u_tex, v_uv).rgb * u_tint;
    float g = texture2D(u_noise, v_uv * vec2(u_grain.w, u_grain.w * 0.5625) + u_grain.xy).r - 0.5;
    gl_FragColor = vec4(c + g * u_grain.z, 1.0);
}
";

const UI_VS: &str = "attribute vec2 a_pos;
attribute vec2 a_uv;
attribute vec4 a_col;
uniform vec2 u_scale;
varying vec2 v_uv;
varying vec4 v_col;
void main() {
    v_uv = a_uv * (1.0 / 128.0);
    v_col = a_col;
    gl_Position = vec4(a_pos * u_scale + vec2(-1.0, 1.0), 0.0, 1.0);
}
";

const UI_FS: &str = "precision mediump float;
uniform sampler2D u_tex;
varying vec2 v_uv;
varying vec4 v_col;
void main() {
    gl_FragColor = vec4(v_col.rgb, v_col.a * texture2D(u_tex, v_uv).r);
}
";

pub struct Program<const N: usize> {
    pub id: u32,
    pub loc: [i32; N],
}

impl<const N: usize> Program<N> {
    fn new(gl: &Gl, vs: &str, fs: &str, attribs: &[&CStr], uniforms: [&CStr; N]) -> Result<Program<N>, String> {
        let attribs: Vec<(u32, &CStr)> = attribs.iter().enumerate().map(|(i, a)| (i as u32, *a)).collect();
        let id = gl.program(vs, fs, &attribs)?;
        let loc = uniforms.map(|u| gl.uniform_location(id, u));
        gl.use_program(id);
        uniforms.iter().zip(loc).for_each(|(name, l)| match name.to_bytes() {
            b"u_tex" => gl.uniform1i(l, 0),
            b"u_dither" => gl.uniform1i(l, 1),
            b"u_noise" => gl.uniform1i(l, 2),
            _ => {}
        });
        Ok(Program { id, loc })
    }
}

pub mod world {
    pub const MVP: usize = 0;
    pub const MODEL: usize = 1;
    pub const LAMP: usize = 2;
    pub const AMB: usize = 3;
    pub const LPOS: usize = 4;
    pub const LDIR: usize = 8;
    pub const EYE: usize = 12;
    pub const FOG: usize = 13;
    pub const SNAP: usize = 14;
    pub const FOGC: usize = 15;
    pub const FX: usize = 16;
}

pub mod glow {
    pub const MVP: usize = 0;
    pub const SNAP: usize = 1;
    pub const COLOR: usize = 2;
    pub const MODEL: usize = 3;
    pub const EYE: usize = 4;
}

pub mod blit {
    pub const RECT: usize = 0;
    pub const CANVAS: usize = 1;
    pub const TINT: usize = 2;
    pub const GRAIN: usize = 3;
}

pub mod ui {
    pub const SCALE: usize = 0;
}

const WORLD_UNIFORMS: [&CStr; 19] = [
    c"u_mvp", c"u_model", c"u_lamp", c"u_amb", c"u_lpos[0]", c"u_lpos[1]", c"u_lpos[2]", c"u_lpos[3]", c"u_ldir[0]",
    c"u_ldir[1]", c"u_ldir[2]", c"u_ldir[3]", c"u_eye", c"u_fog", c"u_snap", c"u_fogc", c"u_fx", c"u_tex", c"u_dither",
];
const WORLD_ATTRIBS: [&CStr; 3] = [c"a_pos", c"a_uv", c"a_nrm"];

pub struct Shaders {
    pub world: Program<19>,
    pub screen: Program<19>,
    pub glow: Program<6>,
    pub blit: Program<3>,
    pub blit_fx: Program<6>,
    pub ui: Program<2>,
}

impl Shaders {
    pub fn new(gl: &Gl) -> Result<Shaders, String> {
        let screen_fs = format!("#define SCREEN\n{WORLD_FS}");
        Ok(Shaders {
            world: Program::new(gl, WORLD_VS, WORLD_FS, &WORLD_ATTRIBS, WORLD_UNIFORMS)?,
            screen: Program::new(gl, WORLD_VS, &screen_fs, &WORLD_ATTRIBS, WORLD_UNIFORMS)?,
            glow: Program::new(gl, GLOW_VS, GLOW_FS, &WORLD_ATTRIBS, [c"u_mvp", c"u_snap", c"u_color", c"u_model", c"u_eye", c"u_dither"])?,
            blit: Program::new(gl, BLIT_VS, BLIT_FS, &[c"a_pos"], [c"u_rect", c"u_canvas", c"u_tex"])?,
            blit_fx: Program::new(
                gl,
                BLIT_VS,
                BLIT_FX_FS,
                &[c"a_pos"],
                [c"u_rect", c"u_canvas", c"u_tint", c"u_grain", c"u_tex", c"u_noise"],
            )?,
            ui: Program::new(gl, UI_VS, UI_FS, &[c"a_pos", c"a_uv", c"a_col"], [c"u_scale", c"u_tex"])?,
        })
        .inspect(|_| gl.use_program(0))
    }

    pub fn delete(&self, gl: &Gl) {
        [self.world.id, self.screen.id, self.glow.id, self.blit.id, self.blit_fx.id, self.ui.id]
            .iter()
            .for_each(|&p| gl.delete_program(p));
    }
}
