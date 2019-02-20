use std::borrow::Cow;
use glium::Surface;
use rusttype;

#[derive(Copy, Clone)]
pub struct Vertex {
    pos: [f32; 2],
    col: [f32; 4],
}

pub enum Cmd<'a> {
    Draw { vertices: Vec<Vertex>, indices: Vec<u16> },
    DrawGlyphs { glyphs: Vec<rusttype::PositionedGlyph<'a>> },
}

const VERT: &'static str = "
#version 140

in vec2 pos;
in vec4 col;

out vec4 v_col;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
    v_col = col;
}
";
const FRAG: &'static str = "
#version 140
in vec4 v_col;
out vec4 f_col;

void main() {
    f_col = v_col;
}
";
const TEXT_VERT: &'static str = "
#version 140

in vec2 pos;
in vec2 uv;
in vec4 col;

out vec2 v_uv;
out vec4 v_col;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
    v_uv = uv;
    v_col = col;
}
";
const TEXT_FRAG: &'static str = "
#version 140
uniform sampler2D tex;
in vec2 v_uv;
in vec4 v_col;
out vec4 f_col;

void main() {
    f_col = v_col * vec4(1, 1, 1, texture(tex, v_uv).r);
}
";

pub struct Renderer<'a> {
    program: glium::Program,

    cache: rusttype::gpu_cache::Cache<'a>,
    cache_tex: glium::texture::Texture2d,
    program_glyph: glium::Program,
}

impl<'a> Renderer<'a> {
    pub fn new(display: &glium::Display, dpi_factor: f32) -> Renderer<'a> {
        let program = program!(
            display,
            140 => {
                vertex: VERT,
                fragment: FRAG,
            }).unwrap();

        let (cache_width, cache_height) = (512 * dpi_factor as u32, 512 * dpi_factor as u32);
        let cache = rusttype::gpu_cache::Cache::builder()
            .dimensions(cache_width, cache_height)
            .build();

        let cache_tex = glium::texture::Texture2d::with_format(
            display,
            glium::texture::RawImage2d {
                data: Cow::Owned(vec![128u8; cache_width as usize * cache_height as usize]),
                width: cache_width,
                height: cache_height,
                format: glium::texture::ClientFormat::U8
            },
            glium::texture::UncompressedFloatFormat::U8,
            glium::texture::MipmapsOption::NoMipmap).unwrap();

        let program_glyph = program!(
            display,
            140 => {
                vertex: TEXT_VERT,
                fragment: TEXT_FRAG,
            }).unwrap();

        Renderer {
            program: program,

            cache: cache,
            cache_tex: cache_tex,
            program_glyph: program_glyph,
        }
    }

    pub fn render(&mut self, display: &glium::Display, cmds: &[Cmd<'a>]) {
        let mut target = display.draw();
        target.clear_color(0.01, 0.015, 0.02, 1.0);

        for cmd in cmds {
            match cmd {
                Cmd::Draw { vertices, indices } => {
                    self.draw(display, &mut target, vertices, indices);
                }
                Cmd::DrawGlyphs { glyphs } => {
                    self.draw_glyphs(display, &mut target, glyphs);
                }
            }
        }

        target.finish().unwrap();
    }

    fn draw(&self, display: &glium::Display, target: &mut glium::Frame, vertices: &[Vertex], indices: &[u16]) {
        implement_vertex!(Vertex, pos, col);

        let vbo = glium::VertexBuffer::new(display, &vertices).unwrap();
        let ibo = glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap();

        target.draw(&vbo, &ibo,
            &self.program,
            &glium::uniforms::EmptyUniforms,
            &glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                ..Default::default()
            }).unwrap();
    }

    fn draw_glyphs(&mut self, display: &glium::Display, target: &mut glium::Frame, glyphs: &[rusttype::PositionedGlyph<'a>]) {
        let (screen_width, screen_height) = {
            let (w, h) = display.get_framebuffer_dimensions();
            (w as f32, h as f32)
        };

        for glyph in glyphs {
            self.cache.queue_glyph(0, glyph.clone());
        }
        let cache_tex = &mut self.cache_tex;
        self.cache.cache_queued(|rect, data| {
            cache_tex.main_level().write(glium::Rect {
                left: rect.min.x,
                bottom: rect.min.y,
                width: rect.width(),
                height: rect.height()
            }, glium::texture::RawImage2d {
                data: Cow::Borrowed(data),
                width: rect.width(),
                height: rect.height(),
                format: glium::texture::ClientFormat::U8
            });
        }).unwrap();

        let uniforms = uniform! {
            tex: self.cache_tex.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        };

        #[derive(Copy, Clone)]
        struct Vertex {
            pos: [f32; 2],
            uv: [f32; 2],
            col: [f32; 4]
        }
        implement_vertex!(Vertex, pos, uv, col);

        let col = [1.0, 1.0, 1.0, 1.0];

        let mut vertices = Vec::with_capacity(glyphs.len() * 4);
        let mut indices = Vec::with_capacity(glyphs.len() * 6);
        let mut vertex_i: u16 = 0;
        for glyph in glyphs {
            if let Ok(Some((uv_rect, screen_rect))) = self.cache.rect_for(0, glyph) {
                let (x1,y1) = self.pixel_to_ndc(screen_rect.min.x as f32, screen_rect.min.y as f32, screen_width, screen_height);
                let (x2,y2) = self.pixel_to_ndc(screen_rect.max.x as f32, screen_rect.max.y as f32, screen_width, screen_height);
                vertices.push(Vertex { pos: [x1,y1], uv: [uv_rect.min.x, uv_rect.min.y], col });
                vertices.push(Vertex { pos: [x1,y2], uv: [uv_rect.min.x, uv_rect.max.y], col });
                vertices.push(Vertex { pos: [x2,y2], uv: [uv_rect.max.x, uv_rect.max.y], col });
                vertices.push(Vertex { pos: [x2,y1], uv: [uv_rect.max.x, uv_rect.min.y], col });
                indices.push(vertex_i);     indices.push(vertex_i + 1); indices.push(vertex_i + 2);
                indices.push(vertex_i + 2); indices.push(vertex_i + 3); indices.push(vertex_i);
            }
            vertex_i += 4;
        }
        let vbo = glium::VertexBuffer::new(display, &vertices).unwrap();
        let ibo = glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap();

        target.draw(&vbo, &ibo,
            &self.program_glyph, &uniforms,
            &glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                ..Default::default()
            }).unwrap();
    }

    #[inline]
    fn pixel_to_ndc(&self, x: f32, y: f32, screen_width: f32, screen_height: f32) -> (f32, f32) {
        (2.0 * (x / screen_width as f32 - 0.5), 2.0 * (1.0 - y / screen_height as f32 - 0.5))
    }
}
