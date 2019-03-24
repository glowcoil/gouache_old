use std::ffi::{CStr, CString};
use gl::types::{GLuint, GLint, GLchar, GLenum};

use crate::alloc::Slab;

#[derive(Copy, Clone)]
pub enum TexFormat { RGBA, A }
pub type TexId = usize;

macro_rules! offset {
    ($type:ty, $field:ident) => { &(*(0 as *const $type)).$field as *const _ as usize }
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub col: [f32; 4],
}

#[derive(Copy, Clone)]
pub struct VertexUV {
    pub pos: [f32; 3],
    pub col: [f32; 4],
    pub uv: [f32; 2],
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

struct Texture {
    format: TexFormat,
    tex: GLuint,
}

pub struct Renderer {
    prog: GLuint,
    prog_tex_rgba: GLuint,
    prog_tex_a: GLuint,

    textures: Slab<Texture>,
}

impl Renderer {
    pub fn new() -> Renderer {
        let prog: GLuint = program(
            &CStr::from_bytes_with_nul(VERT).unwrap(),
            &CStr::from_bytes_with_nul(FRAG).unwrap()).unwrap();

        let prog_tex_rgba: GLuint = program(
            &CStr::from_bytes_with_nul(VERT_TEX_RGBA).unwrap(),
            &CStr::from_bytes_with_nul(FRAG_TEX_RGBA).unwrap()).unwrap();

        let prog_tex_a: GLuint = program(
            &CStr::from_bytes_with_nul(VERT_TEX_A).unwrap(),
            &CStr::from_bytes_with_nul(FRAG_TEX_A).unwrap()).unwrap();

        unsafe {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::BLEND);
        }

        Renderer {
            prog,
            prog_tex_rgba,
            prog_tex_a,

            textures: Slab::new(),
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
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as GLint, offset!(Vertex, pos) as *const gl::types::GLvoid);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as GLint, offset!(Vertex, col) as *const gl::types::GLvoid);

            gl::UseProgram(self.prog);

            gl::DrawElements(gl::TRIANGLES, indices.len() as i32, gl::UNSIGNED_SHORT, 0 as *const gl::types::GLvoid);

            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);

            gl::DeleteVertexArrays(1, &vao);
            gl::DeleteBuffers(1, &ibo);
            gl::DeleteBuffers(1, &vbo);
        }
    }

    pub fn draw_tex(&mut self, vertices: &[VertexUV], indices: &[u16], tex_id: TexId) {
        let tex = self.textures.get(tex_id).unwrap();
        unsafe {
            let mut vbo: u32 = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * std::mem::size_of::<VertexUV>()) as isize, vertices.as_ptr() as *const std::ffi::c_void, gl::STATIC_DRAW);

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
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, std::mem::size_of::<VertexUV>() as GLint, offset!(VertexUV, pos) as *const gl::types::GLvoid);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, std::mem::size_of::<VertexUV>() as GLint, offset!(VertexUV, col) as *const gl::types::GLvoid);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<VertexUV>() as GLint, offset!(VertexUV, uv) as *const gl::types::GLvoid);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, tex.tex);

            match tex.format {
                TexFormat::RGBA => { gl::UseProgram(self.prog_tex_rgba); }
                TexFormat::A => { gl::UseProgram(self.prog_tex_a); }
            }
            gl::Uniform1i(0, 0);

            gl::DrawElements(gl::TRIANGLES, indices.len() as i32, gl::UNSIGNED_SHORT, 0 as *const gl::types::GLvoid);

            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);
            gl::DisableVertexAttribArray(2);

            gl::DeleteVertexArrays(1, &vao);
            gl::DeleteBuffers(1, &ibo);
            gl::DeleteBuffers(1, &vbo);
        }
    }

    pub fn create_tex(&mut self, format: TexFormat, width: usize, height: usize, pixels: &[u8]) -> TexId {
        let flipped = flip(pixels, width);
        let mut tex: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            match format {
                TexFormat::RGBA => {
                    assert!(flipped.len() == width * height * 4);
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
                    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA32UI as GLint, width as i32, height as i32, 0, gl::RGBA, gl::UNSIGNED_INT_8_8_8_8, flipped.as_ptr() as *const std::ffi::c_void);
                }
                TexFormat::A => {
                    assert!(flipped.len() == width * height);
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8 as GLint, width as i32, height as i32, 0, gl::RED, gl::UNSIGNED_BYTE, flipped.as_ptr() as *const std::ffi::c_void);
                }
            }
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }
        self.textures.insert(Texture { format, tex })
    }

    pub fn update_tex(&mut self, texture: TexId, x: usize, y: usize, width: usize, height: usize, pixels: &[u8]) {
        let flipped = flip(pixels, width);
        let Texture { format, tex } = self.textures.get(texture).unwrap();
        unsafe { gl::BindTexture(gl::TEXTURE_2D, *tex); }
        match format {
            TexFormat::RGBA => {
                if flipped.len() != width * height * 4 { panic!() }
                unsafe {
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
                    gl::TexSubImage2D(gl::TEXTURE_2D, 0, x as i32, y as i32, width as i32, height as i32, gl::RGBA, gl::UNSIGNED_INT_8_8_8_8, flipped.as_ptr() as *const std::ffi::c_void);
                }
            }
            TexFormat::A => {
                if flipped.len() != width * height { panic!() }
                unsafe {
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                    gl::TexSubImage2D(gl::TEXTURE_2D, 0, x as i32, y as i32, width as i32, height as i32, gl::RED, gl::UNSIGNED_BYTE, flipped.as_ptr() as *const std::ffi::c_void);
                }
            }
        }
    }

    pub fn delete_tex(&mut self, texture: TexId) {
        let Texture { tex, .. } = self.textures.remove(texture).unwrap();
        unsafe {
            gl::DeleteTextures(1, &tex);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.prog);
            gl::DeleteProgram(self.prog_tex_rgba);
            gl::DeleteProgram(self.prog_tex_a);

            for Texture { tex, .. } in self.textures.iter() {
                gl::DeleteTextures(1, tex);
            }
        }
    }
}

fn flip(pixels: &[u8], width: usize) -> Vec<u8> {
    let mut flipped: Vec<u8> = Vec::with_capacity(pixels.len());
    for chunk in pixels.rchunks(width) {
        flipped.extend(chunk);
    }
    flipped
}
