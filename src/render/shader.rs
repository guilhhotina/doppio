use crate::gl::Gl;
use std::ffi::CStr;

pub const MAX_LIGHTS: usize = 4;

const WORLD_VS: &str = "attribute vec3 a_pos;
attribute vec2 a_uv;
attribute vec4 a_nrm;
attribute vec4 a_tile;
attribute vec4 a_lit;
uniform mat4 u_mvp;
uniform vec3 u_lamp;
uniform vec3 u_moon;
uniform vec3 u_flame;
uniform vec3 u_amb;
uniform vec4 u_lpos[4];
uniform vec4 u_ldir[4];
uniform vec4 u_lcol[4];
uniform vec3 u_eye;
uniform vec2 u_fog;
uniform vec2 u_snap;
varying vec3 v_uvq;
varying vec3 v_col;
varying float v_fog;
varying vec4 v_tile;
void main() {
    vec3 wp = a_pos;
    vec3 n = a_nrm.xyz;
    vec3 c = u_amb + u_moon * a_nrm.w + (u_lamp * a_lit.rgb + u_flame * a_lit.a) * 2.0;
    vec4 p = u_mvp * vec4(a_pos, 1.0);
    for (int i = 0; i < 4; i++) {
        vec3 d = u_lpos[i].xyz - wp;
        float dd = dot(d, d);
        vec3 l = d * inversesqrt(dd + 0.0001);
        float cone = smoothstep(u_ldir[i].w, u_ldir[i].w + 0.1, dot(-l, u_ldir[i].xyz));
        float wrap = max(dot(n, l), 0.0) * 0.8 + 0.2;
        c += u_lcol[i].rgb * (u_lpos[i].w * cone * wrap / (1.0 + dd * u_lcol[i].w));
    }
    v_col = c;
    if (p.w > 0.05) {
        p.xy = floor(p.xy / p.w * u_snap + 0.5) / u_snap * p.w;
    }
    v_fog = clamp((length(wp - u_eye) - u_fog.x) * u_fog.y, 0.0, 1.0);
    float q = p.w * 0.05;
    v_uvq = vec3(a_uv * (q / 2048.0), q);
    v_tile = a_tile * vec4(0.0625, 0.03125, 0.0625, 0.03125);
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
varying vec4 v_tile;
void main() {
    vec2 uv = v_uvq.xy / v_uvq.z;
#ifdef SCREEN
    float s = texture2D(u_tex, uv * vec2(0.7, 0.5) + u_fx.xy).r;
    float line = fract(gl_FragCoord.y * 0.5) * 0.25 + 0.75;
    vec3 c = vec3(0.78, 0.86, 1.0) * (s * u_fx.z * line + u_fx.w) + v_col * 0.04;
#else
#ifdef WATER
    vec2 w = uv + vec2(sin(uv.y * 5.0 + u_fx.z) * 0.06, cos(uv.x * 4.0 + u_fx.z * 0.8) * 0.06);
    vec3 a = texture2D(u_tex, v_tile.xy + clamp(fract(w + u_fx.xy), 0.02, 0.98) * v_tile.zw).rgb;
    vec3 b = texture2D(u_tex, v_tile.xy + clamp(fract(w.yx * 0.7 - u_fx.xy), 0.02, 0.98) * v_tile.zw).rgb;
    vec3 m = (a + b) * 0.5;
    vec3 c = m * v_col + vec3(0.55, 0.65, 0.8) * u_fx.w * max(a.g * b.g * 4.0 - 1.6, 0.0);
#else
    vec4 t = texture2D(u_tex, v_tile.xy + clamp(fract(uv), 0.02, 0.98) * v_tile.zw);
#ifdef ALPHA
    if (t.a < 0.5) discard;
#endif
    vec3 c = t.rgb * v_col;
#endif
#endif
    c = mix(c, u_fogc, v_fog);
    float d = texture2D(u_dither, gl_FragCoord.xy * 0.25).r;
    gl_FragColor = vec4(floor(c * 31.0 + d) * (1.0 / 31.0), 1.0);
}
";

const SKY_VS: &str = "attribute vec3 a_pos;
attribute vec2 a_uv;
attribute vec4 a_nrm;
attribute vec4 a_tile;
uniform mat4 u_mvp;
uniform vec3 u_eye;
uniform vec2 u_snap;
varying vec2 v_uv;
varying vec4 v_tile;
varying float v_fog;
void main() {
    vec4 p = u_mvp * vec4(a_pos + u_eye, 1.0);
    if (p.w > 0.05) {
        p.xy = floor(p.xy / p.w * u_snap + 0.5) / u_snap * p.w;
    }
    p.z = p.w * 0.99999;
    v_uv = a_uv * (1.0 / 2048.0);
    v_tile = a_tile * vec4(0.0625, 0.03125, 0.0625, 0.03125);
    v_fog = a_nrm.w;
    gl_Position = p;
}
";

const SKY_FS: &str = "precision mediump float;
uniform sampler2D u_tex;
uniform sampler2D u_dither;
uniform vec3 u_fogc;
uniform vec3 u_tint;
varying vec2 v_uv;
varying vec4 v_tile;
varying float v_fog;
void main() {
    vec4 t = texture2D(u_tex, v_tile.xy + clamp(fract(v_uv), 0.02, 0.98) * v_tile.zw);
    if (t.a < 0.5) discard;
    vec3 c = mix(t.rgb * u_tint, u_fogc, v_fog);
    float d = texture2D(u_dither, gl_FragCoord.xy * 0.25).r;
    gl_FragColor = vec4(floor(c * 31.0 + d) * (1.0 / 31.0), 1.0);
}
";

const GLOW_VS: &str = "attribute vec3 a_pos;
attribute vec2 a_uv;
attribute vec4 a_nrm;
uniform mat4 u_mvp;
uniform vec3 u_eye;
uniform vec2 u_snap;
attribute vec4 a_tile;
varying vec3 v_c;
void main() {
    vec4 p = u_mvp * vec4(a_pos, 1.0);
    if (p.w > 0.05) {
        p.xy = floor(p.xy / p.w * u_snap + 0.5) / u_snap * p.w;
    }
    float f = abs(dot(a_nrm.xyz, normalize(u_eye - a_pos)));
    float k = a_uv.y > 1024.0 ? 1.0 : f * f;
    v_c = a_tile.rgb * (a_uv.x * (1.0 / 2048.0) * k / 255.0);
    gl_Position = p;
}
";

const GLOW_FS: &str = "precision mediump float;
uniform sampler2D u_dither;
uniform vec3 u_color;
varying vec3 v_c;
void main() {
    float d = texture2D(u_dither, gl_FragCoord.xy * 0.25).r;
    gl_FragColor = vec4(floor(u_color * v_c * 31.0 + d) * (1.0 / 31.0), 1.0);
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
    pub const LAMP: usize = 1;
    pub const AMB: usize = 2;
    pub const LPOS: usize = 3;
    pub const LDIR: usize = 7;
    pub const EYE: usize = 11;
    pub const FOG: usize = 12;
    pub const SNAP: usize = 13;
    pub const FOGC: usize = 14;
    pub const FX: usize = 15;
    pub const MOON: usize = 16;
    pub const FLAME: usize = 17;
    pub const LCOL: usize = 18;
}

pub mod sky {
    pub const MVP: usize = 0;
    pub const EYE: usize = 1;
    pub const SNAP: usize = 2;
    pub const FOGC: usize = 3;
    pub const TINT: usize = 4;
}

pub mod glow {
    pub const MVP: usize = 0;
    pub const SNAP: usize = 1;
    pub const COLOR: usize = 2;
    pub const EYE: usize = 3;
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

const WORLD_UNIFORMS: [&CStr; 25] = [
    c"u_mvp", c"u_lamp", c"u_amb", c"u_lpos[0]", c"u_lpos[1]", c"u_lpos[2]", c"u_lpos[3]", c"u_ldir[0]", c"u_ldir[1]",
    c"u_ldir[2]", c"u_ldir[3]", c"u_eye", c"u_fog", c"u_snap", c"u_fogc", c"u_fx", c"u_moon", c"u_flame", c"u_lcol[0]",
    c"u_lcol[1]", c"u_lcol[2]", c"u_lcol[3]", c"u_tex", c"u_dither", c"u_noise",
];
const WORLD_ATTRIBS: [&CStr; 5] = [c"a_pos", c"a_uv", c"a_nrm", c"a_tile", c"a_lit"];

pub type World = Program<25>;

pub struct Shaders {
    pub world: World,
    pub alpha: World,
    pub water: World,
    pub screen: World,
    pub sky: Program<7>,
    pub glow: Program<5>,
    pub blit: Program<3>,
    pub blit_fx: Program<6>,
    pub ui: Program<2>,
}

impl Shaders {
    pub fn new(gl: &Gl) -> Result<Shaders, String> {
        let variant = |def: &str| -> Result<World, String> {
            Program::new(gl, WORLD_VS, &format!("#define {def}\n{WORLD_FS}"), &WORLD_ATTRIBS, WORLD_UNIFORMS)
        };
        Ok(Shaders {
            world: Program::new(gl, WORLD_VS, WORLD_FS, &WORLD_ATTRIBS, WORLD_UNIFORMS)?,
            alpha: variant("ALPHA")?,
            water: variant("WATER")?,
            screen: variant("SCREEN")?,
            sky: Program::new(gl, SKY_VS, SKY_FS, &WORLD_ATTRIBS[..4], [c"u_mvp", c"u_eye", c"u_snap", c"u_fogc", c"u_tint", c"u_tex", c"u_dither"])?,
            glow: Program::new(gl, GLOW_VS, GLOW_FS, &WORLD_ATTRIBS[..4], [c"u_mvp", c"u_snap", c"u_color", c"u_eye", c"u_dither"])?,
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
        [self.world.id, self.alpha.id, self.water.id, self.screen.id, self.sky.id, self.glow.id, self.blit.id, self.blit_fx.id, self.ui.id]
            .iter()
            .for_each(|&p| gl.delete_program(p));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::texture::{ATLAS_H, ATLAS_W, CELL};

    #[test]
    fn tile_scale_matches_atlas() {
        let scale = format!("vec4({}, {}, {}, {})", CELL as f32 / ATLAS_W as f32, CELL as f32 / ATLAS_H as f32, CELL as f32 / ATLAS_W as f32, CELL as f32 / ATLAS_H as f32);
        assert!(WORLD_VS.contains(&scale), "{scale}");
        assert!(SKY_VS.contains(&scale), "{scale}");
    }

    #[test]
    fn atlas_lookups_stay_inside_their_tile_at_mediump() {
        let ulp = 2f32.powi(-11);
        [WORLD_FS, SKY_FS].iter().for_each(|src| {
            assert!(!src.contains("+ fract("), "unclamped atlas lookup");
            src.match_indices("clamp(fract(").for_each(|(i, _)| {
                let tail = &src[i..];
                let args = &tail[tail.find("), ").expect("clamp args") + 3..tail.find(") *").expect("clamp end")];
                let lo: f32 = args.split(", ").next().and_then(|v| v.parse().ok()).expect("margin");
                assert!(lo * CELL as f32 / ATLAS_H as f32 > ulp, "margin {lo} under one fp16 ulp of the atlas");
            });
        });
    }
}
