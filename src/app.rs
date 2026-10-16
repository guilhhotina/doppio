use crate::core::{Frontend, HEIGHT, WIDTH};
use crate::ffi::{LOG_ERROR, LOG_INFO};
use crate::gl::{self, Gl};
use std::time::Instant;

const VS: &str = "attribute vec3 a_pos;
attribute vec3 a_col;
uniform mat4 u_mvp;
varying vec3 v_col;
void main() { v_col = a_col; gl_Position = u_mvp * vec4(a_pos, 1.0); }
";

const FS: &str = "precision mediump float;
varying vec3 v_col;
uniform vec3 u_tint;
void main() { gl_FragColor = vec4(v_col * u_tint, 1.0); }
";

const TRIANGLE: [f32; 18] = [
    -1.0, -0.8, 0.0, 1.0, 0.3, 0.2,
    1.0, -0.8, 0.0, 0.2, 1.0, 0.3,
    0.0, 1.0, 0.0, 0.3, 0.4, 1.0,
];

const PALETTE: [[f32; 3]; 10] = [
    [1.0, 0.2, 0.2],
    [1.0, 1.0, 0.2],
    [0.6, 0.6, 0.6],
    [0.2, 1.0, 1.0],
    [1.0, 0.5, 0.0],
    [0.5, 0.0, 1.0],
    [0.0, 1.0, 0.3],
    [1.0, 0.0, 0.8],
    [0.2, 0.4, 1.0],
    [0.0, 0.6, 0.0],
];

struct Scene {
    gl: Gl,
    program: u32,
    vbo: u32,
    u_mvp: i32,
    u_tint: i32,
}

pub struct App {
    start: Instant,
    scene: Option<Scene>,
}

fn mat_mul(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    std::array::from_fn(|i| (0..4).map(|k| a[k * 4 + i % 4] * b[(i / 4) * 4 + k]).sum())
}

fn mvp(angle: f32) -> [f32; 16] {
    let (s, c) = angle.sin_cos();
    let model_view = [c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, -3.0, 1.0];
    let f = 1.0 / (30f32.to_radians()).tan();
    let (near, far) = (0.1f32, 10.0f32);
    let aspect = WIDTH as f32 / HEIGHT as f32;
    let proj = [
        f / aspect, 0.0, 0.0, 0.0,
        0.0, f, 0.0, 0.0,
        0.0, 0.0, (far + near) / (near - far), -1.0,
        0.0, 0.0, 2.0 * far * near / (near - far), 0.0,
    ];
    mat_mul(&proj, &model_view)
}

impl App {
    pub fn new(fe: &Frontend) -> App {
        let night = fe.variable(c"doppio_night_seconds").unwrap_or_else(|| "default".into());
        let user = fe.username().unwrap_or_else(|| "?".into());
        fe.log(LOG_INFO, &format!("{} loaded (night_seconds={night}, user={user})", env!("CARGO_PKG_VERSION")));
        App { start: Instant::now(), scene: None }
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    pub fn context_reset(&mut self, fe: &Frontend, gl: Gl) {
        fe.log(LOG_INFO, &format!("GL renderer: {}", gl.string(gl::RENDERER).unwrap_or("?")));
        self.scene = gl
            .program(VS, FS, &[(0, c"a_pos"), (1, c"a_col")])
            .inspect_err(|e| fe.log(LOG_ERROR, e))
            .ok()
            .map(|program| {
                let vbo = gl.gen_buffer();
                gl.bind_buffer(gl::ARRAY_BUFFER, vbo);
                gl.buffer_data(gl::ARRAY_BUFFER, &TRIANGLE, gl::STATIC_DRAW);
                gl.bind_buffer(gl::ARRAY_BUFFER, 0);
                Scene {
                    gl,
                    program,
                    vbo,
                    u_mvp: gl.uniform_location(program, c"u_mvp"),
                    u_tint: gl.uniform_location(program, c"u_tint"),
                }
            });
    }

    pub fn context_destroy(&mut self) {
        self.scene = None;
    }

    pub fn frame(&mut self, _fe: &Frontend, input: [u16; 4]) -> bool {
        let Some(s) = &self.scene else { return false };
        let gl = &s.gl;
        let held = input.iter().fold(0, |a, b| a | b);
        let tint = match held {
            0 => [1.0; 3],
            _ => PALETTE[held.trailing_zeros() as usize % PALETTE.len()],
        };
        let angle = self.start.elapsed().as_secs_f32() * 1.5;
        gl.bind_target();
        gl.viewport(0, 0, WIDTH as i32, HEIGHT as i32);
        gl.disable(gl::SCISSOR_TEST);
        gl.disable(gl::BLEND);
        gl.disable(gl::CULL_FACE);
        gl.enable(gl::DEPTH_TEST);
        gl.depth_func(gl::LESS);
        gl.depth_mask(true);
        gl.clear_color(0.05, 0.04, 0.07, 1.0);
        gl.clear_depth(1.0);
        gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl.use_program(s.program);
        gl.bind_buffer(gl::ARRAY_BUFFER, s.vbo);
        gl.attrib_pointer(0, 3, gl::FLOAT, false, 24, 0);
        gl.attrib_pointer(1, 3, gl::FLOAT, false, 24, 12);
        gl.enable_attrib(0);
        gl.enable_attrib(1);
        [(angle, tint), (0.9, [0.5; 3])].iter().for_each(|(a, t)| {
            gl.uniform_mat4(s.u_mvp, &mvp(*a));
            gl.uniform3f(s.u_tint, t[0], t[1], t[2]);
            gl.draw_arrays(gl::TRIANGLES, 0, 3);
        });
        gl.disable_attrib(0);
        gl.disable_attrib(1);
        gl.bind_buffer(gl::ARRAY_BUFFER, 0);
        gl.use_program(0);
        gl.disable(gl::DEPTH_TEST);
        true
    }

    pub fn audio(&mut self, out: &mut [i16]) {
        out.fill(0);
    }
}
