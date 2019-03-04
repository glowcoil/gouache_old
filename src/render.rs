use std::ffi::{CStr, CString};
use gl::types::{GLuint, GLint, GLchar, GLenum};

pub enum TexFormat { RGBA, A }
pub type TexId = u32;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub col: [f32; 4],
}

macro_rules! offset {
    ($type:ty, $field:ident) => { unsafe { &(*(0 as *const $type)).$field as *const _ as usize } }
}

const VERT: &[u8] = b"
#version 330

layout(location = 0) in vec3 pos;
layout(location = 1) in vec4 col;

out vec4 v_col;

void main() {
    gl_Position = vec4(pos, 1.0);
    v_col = col;
}
\0";
const FRAG: &[u8] = b"
#version 330

in vec4 v_col;

out vec4 f_col;

void main() {
    f_col = v_col;
}
\0";
const VERT_TEX_RGBA: &[u8] = b"
#version 330

layout(location = 0) in vec3 pos;
layout(location = 1) in vec4 col;
layout(location = 2) in vec2 uv;

out vec2 v_uv;
out vec4 v_col;

void main() {
    gl_Position = vec4(pos, 1.0);
    v_uv = uv;
    v_col = col;
}
\0";
const FRAG_TEX_RGBA: &[u8] = b"
#version 330
uniform sampler2D tex;

in vec2 v_uv;
in vec4 v_col;

out vec4 f_col;

void main() {
    f_col = v_col * texture(tex, v_uv).rgba;
}
\0";
const VERT_TEX_A: &[u8] = b"
#version 330

layout(location = 0) in vec3 pos;
layout(location = 1) in vec4 col;
layout(location = 2) in vec2 uv;

out vec4 v_col;
out vec2 v_uv;

void main() {
    gl_Position = vec4(pos, 1.0);
    v_uv = uv;
    v_col = col;
}
\0";
const FRAG_TEX_A: &[u8] = b"
#version 330
uniform sampler2D tex;

in vec4 v_col;
in vec2 v_uv;

out vec4 f_col;

void main() {
    f_col = v_col * vec4(1, 1, 1, texture(tex, v_uv).r);
}
\0";

pub struct Renderer {
    prog: GLuint,
}

fn shader(shader_src: &CStr, shader_type: GLenum) -> Result<GLuint, String> {
    unsafe {
        let shader: GLuint = gl::CreateShader(shader_type);
        gl::ShaderSource(shader, 1, &shader_src.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        let mut valid: GLint = 1;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut valid);
        if valid == 0 {
            let mut len: GLint = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let error = CString::new(vec![' ' as u8; len as usize]).unwrap();
            gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
            return Err(error.into_string().unwrap());
        }

        Ok(shader)
    }
}

fn program(vert_src: &CStr, frag_src: &CStr) -> Result<GLuint, String> {
    unsafe {
        let vert = shader(vert_src, gl::VERTEX_SHADER).unwrap();
        let frag = shader(frag_src, gl::FRAGMENT_SHADER).unwrap();
        let prog = gl::CreateProgram();
        gl::AttachShader(prog, vert);
        gl::AttachShader(prog, frag);
        gl::LinkProgram(prog);

        let mut valid: GLint = 1;
        gl::GetProgramiv(prog, gl::COMPILE_STATUS, &mut valid);
        if valid == 0 {
            let mut len: GLint = 0;
            gl::GetProgramiv(prog, gl::INFO_LOG_LENGTH, &mut len);
            let error = CString::new(vec![' ' as u8; len as usize]).unwrap();
            gl::GetProgramInfoLog(prog, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
            return Err(error.into_string().unwrap());
        }

        gl::DetachShader(prog, vert);
        gl::DetachShader(prog, frag);

        gl::DeleteShader(vert);
        gl::DeleteShader(frag);

        Ok(prog)
    }
}

impl Renderer {
    pub fn new() -> Renderer {
        let prog: GLuint = program(
            &CStr::from_bytes_with_nul(VERT).unwrap(),
            &CStr::from_bytes_with_nul(FRAG).unwrap()).unwrap();

        Renderer {
            prog,
        }
    }

    pub fn draw(&mut self, vertices: &[Vertex], indices: &[u16]) {
        unsafe {
            let mut vbo: u32 = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * std::mem::size_of::<Vertex>()) as isize, vertices.as_ptr() as *const std::ffi::c_void, gl::STATIC_DRAW);

            let mut ibo: u32 = 0;
            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * std::mem::size_of::<u16>()) as isize, indices.as_ptr() as *const std::ffi::c_void, gl::STATIC_DRAW);

            let mut vao: u32 = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as GLint, offset!(Vertex, pos) as *const gl::types::GLvoid);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as GLint, offset!(Vertex, col) as *const gl::types::GLvoid);

            gl::UseProgram(self.prog);

            gl::DrawElements(gl::TRIANGLES, indices.len() as i32, gl::UNSIGNED_SHORT, 0 as *const gl::types::GLvoid);

            gl::DeleteVertexArrays(1, &vao);
            gl::DeleteBuffers(1, &ibo);
            gl::DeleteBuffers(1, &vbo);
        }
    }

    pub fn draw_tex(&mut self, vertices: &[Vertex], indices: &[u16], tex_id: TexId) {

    }

    #[inline]
    fn pixel_to_ndc(&self, x: f32, y: f32, screen_width: f32, screen_height: f32) -> (f32, f32) {
        (2.0 * (x / screen_width as f32 - 0.5), 2.0 * (1.0 - y / screen_height as f32 - 0.5))
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.prog);
        }
    }
}
