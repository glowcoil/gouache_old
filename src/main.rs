mod render;
mod ui;
mod alloc;

use render::*;
use ui::*;

extern crate gl;
extern crate glutin;
extern crate font_rs;

use glutin::GlContext;

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0))
        .with_title("justitracker");
    let context = glutin::ContextBuilder::new()
        .with_srgb(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe { gl_window.make_current().unwrap(); }
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    let dpi_factor = gl_window.get_hidpi_factor();
    let mut renderer = Renderer::new();
    // let mut ui = UI::new();

    // let font = rusttype::FontCollection::from_bytes(include_bytes!("../sawarabi-gothic-medium.ttf") as &[u8]).unwrap().into_font().unwrap();

    let tex = renderer.create_tex(TexFormat::A, 4, 4, &[
        255, 127, 255, 127,
        255, 127, 255, 127,
        255, 127, 255, 127,
        255, 127, 255, 127,
    ]);


    let font = font_rs::font::parse(include_bytes!("../HelveticaNeue.ttf")).unwrap();
    let mut atlas = Atlas::new(128, 128);
    atlas.update_counter();
    atlas.update_counter();
    let atlas_tex = renderer.create_tex(TexFormat::A, 128, 128, &[0; 128*128]);
    for c in "qwertyuiopasdfghjklzxcvbnm1234567890`~!@#$%^&*()_+-={{}}[]\\|,./<>?".chars() {
        let glyph_id = font.lookup_glyph_id(c as u32).unwrap();
        let bbox = font.get_bbox(glyph_id, 20).unwrap();
        let rect = atlas.insert(glyph_id, bbox.width() as u32, bbox.height() as u32).unwrap();
        let glyph = font.render_glyph(glyph_id, 16).unwrap();
        renderer.update_tex(atlas_tex, rect.x as usize, rect.y as usize, glyph.width as usize, glyph.height as usize, &glyph.data);
    }


    const FRAME: std::time::Duration = std::time::Duration::from_micros(1_000_000 / 60);
    let mut frames: [u32; 100] = [0; 100];
    let mut i: usize = 0;
    let mut sum: u32 = 0;
    let mut running = true;
    let mut now = std::time::Instant::now();
    while running {
        let elapsed = now.elapsed();
        now = std::time::Instant::now();
        sum -= frames[i];
        frames[i] = elapsed.as_secs() as u32 * 1000 + elapsed.subsec_millis();
        sum += frames[i];
        i = (i + 1) % frames.len();
        let fps = 100000.0 / (sum as f32);
        let fps_text = fps.round().to_string();

        {
            // let mut frame = ui.frame();
            // for (i,c) in fps_text.chars().enumerate() {
            //     frame.glyph(font.glyph(c).scaled(rusttype::Scale::uniform(18.0)).positioned(rusttype::point(10.0 * i as f32, 20.0)));
            // }
            // let cmds = frame.render();
            // renderer.render(&display, &cmds);
        };

        unsafe {
            gl::ClearColor(0.1, 0.15, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            // renderer.draw(&[
            //     render::Vertex { pos: [-0.5, -0.5, 0.0], col: [0.0, 1.0, 1.0, 1.0] },
            //     render::Vertex { pos: [ 0.5, -0.5, 0.0], col: [1.0, 1.0, 1.0, 1.0] },
            //     render::Vertex { pos: [ 0.5,  0.5, 0.0], col: [1.0, 1.0, 1.0, 1.0] },
            //     render::Vertex { pos: [-0.5,  0.5, 0.0], col: [1.0, 1.0, 1.0, 1.0] },
            // ], &[
            //     0, 1, 2, 2, 3, 0
            // ]);
            let size = gl_window.get_inner_size().unwrap();
            let width = 128.0 / size.width as f32;
            let height = 128.0 / size.height as f32;
            renderer.draw_tex(&[
                render::VertexUV { pos: [-width/1.0, -height/1.0, 0.0], col: [0.0, 1.0, 1.0, 1.0], uv: [0.0, 1.0] },
                render::VertexUV { pos: [ width/1.0, -height/1.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [1.0, 1.0] },
                render::VertexUV { pos: [ width/1.0,  height/1.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [1.0, 0.0] },
                render::VertexUV { pos: [-width/1.0,  height/1.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [0.0, 0.0] },
            ], &[
                0, 1, 2, 2, 3, 0
            ], atlas_tex);
        }
        gl_window.swap_buffers().unwrap();

        events_loop.poll_events(|ev| {
            match ev {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(logical_size) => {
                        gl_window.resize(logical_size.to_physical(dpi_factor));
                        // renderer.render(&display, &[Cmd::DrawGlyphs { glyphs: Vec::new() }]);
                    }
                    _ => (),
                },
                _ => (),
            }
        });

        let elapsed = now.elapsed();
        if elapsed < FRAME {
            std::thread::sleep(FRAME - elapsed);
        }
    }
}

type GlyphId = u16;

#[derive(Debug)]
struct Atlas {
    width: u32,
    height: u32,
    nodes: alloc::Slab<Node>,
    map: std::collections::HashMap<GlyphId, Entry>,
    counter: usize,
}

#[derive(Debug)]
struct Entry {
    id: usize,
    rect: Rect,
}

#[derive(Debug)]
struct Node {
    age: usize,
    avg_age: f32,
    contents: Contents,
    parent: Option<usize>,
}

#[derive(Copy, Clone, Debug)]
enum Axis { X, Y }

#[derive(Debug)]
enum Contents {
    Branch {
        axis: Axis,
        split: u32,
        extent: u32,
        fst: usize,
        snd: usize,
    },
    Leaf { id: GlyphId },
    Empty,
}

impl Atlas {
    fn new(width: u32, height: u32) -> Atlas {
        let mut nodes = alloc::Slab::new();
        nodes.insert(Node { age: 0, avg_age: 0.0, contents: Contents::Empty, parent: None });
        Atlas {
            width,
            height,
            nodes,
            map: std::collections::HashMap::new(),
            counter: 0,
        }
    }

    fn update_counter(&mut self) {
        self.counter += 1;
    }

    fn get_cached(&mut self, glyph: GlyphId) -> Option<Rect> {
        if let Some(&Entry { id, rect }) = self.map.get(&glyph) {
            self.mark_used(id);
            Some(rect)
        } else {
            None
        }
    }

    fn insert(&mut self, glyph: GlyphId, width: u32, height: u32) -> Option<Rect> {
        self.insert_inner(glyph, width, height, 0, Rect { x: 0, y: 0, w: self.width, h: self.height })
    }

    fn insert_inner(&mut self, glyph: GlyphId, width: u32, height: u32, index: usize, rect: Rect) -> Option<Rect> {
        println!("{} {:?}", index, rect);
        if width > rect.w || height > rect.h { return None }

        if let Contents::Branch { axis, split, extent, fst, snd } = self.nodes.get(index).unwrap().contents {
            let (fst_rect, snd_rect) = match axis {
                Axis::X => (Rect { w: split, ..rect }, Rect { x: rect.x + split, w: extent - split, ..rect }),
                Axis::Y => (Rect { h: split, ..rect }, Rect { y: rect.y + split, h: extent - split, ..rect }),
            };

            let (fst, fst_rect, snd, snd_rect) = if self.nodes.get(fst).unwrap().avg_age <= self.nodes.get(snd).unwrap().avg_age {
                (fst, fst_rect, snd, snd_rect)
            } else {
                (snd, snd_rect, fst, fst_rect)
            };

            let fst_result = self.insert_inner(glyph, width, height, fst, fst_rect);
            if fst_result.is_some() { return fst_result }

            let snd_result = self.insert_inner(glyph, width, height, snd, snd_rect);
            if snd_result.is_some() { return snd_result }
        }

        if self.nodes.get(index).unwrap().age == self.counter { 
            println!("too new {}", self.nodes.get(index).unwrap().age);
            None
        } else {
            println!("found a place: {:?}", rect);
            Some(self.place(glyph, width, height, index, rect))
        }
    }

    fn place(&mut self, glyph: GlyphId, width: u32, height: u32, index: usize, rect: Rect) -> Rect {
        self.cleanup(index);

        let (axis_1, split_1, extent_1, axis_2, split_2, extent_2) = if rect.h - height > rect.w - width {
            (Axis::Y, height, rect.h, Axis::X, width, rect.w)
        } else {
            (Axis::X, width, rect.w, Axis::Y, height, rect.h)
        };
        let split_1 = nearest_pow_2(split_1).min(extent_1);
        let split_2 = nearest_pow_2(split_2).min(extent_2);

        let fst_1 = self.nodes.insert(Node { age: 0, avg_age: 0.0, contents: Contents::Empty, parent: Some(index) });
        let snd_1 = self.nodes.insert(Node { age: 0, avg_age: 0.0, contents: Contents::Empty, parent: Some(index) });
        let fst_2 = self.nodes.insert(Node { age: 0, avg_age: 0.0, contents: Contents::Leaf { id: glyph }, parent: Some(fst_1) });
        let snd_2 = self.nodes.insert(Node { age: 0, avg_age: 0.0, contents: Contents::Empty, parent: Some(fst_1) });
        self.nodes.get_mut(index).unwrap().contents = Contents::Branch {
            axis: axis_1, split: split_1, extent: extent_1, fst: fst_1, snd: snd_1
        };
        self.nodes.get_mut(fst_1).unwrap().contents = Contents::Branch {
            axis: axis_2, split: split_2, extent: extent_2, fst: fst_2, snd: snd_2
        };

        self.mark_used(fst_2);

        self.map.insert(glyph, Entry { id: index, rect });

        Rect { x: rect.x, y: rect.y, w: width, h: height }
    }

    fn cleanup(&mut self, index: usize) {
        match self.nodes.get(index).unwrap().contents {
            Contents::Branch { fst, snd, .. } => {
                self.cleanup(fst);
                self.cleanup(snd);
            }
            Contents::Leaf { id } => {
                self.map.remove(&id);
            }
            _ => {}
        }
    }

    fn mark_used(&mut self, mut index: usize) {
        let node = self.nodes.get_mut(index).unwrap();
        node.age = self.counter;
        node.avg_age = self.counter as f32;

        while let Some(parent) = self.nodes.get(index).unwrap().parent {
            index = parent;
            if let Contents::Branch { axis, split, extent, fst, snd } = self.nodes.get(index).unwrap().contents {
                let avg_age =
                    self.nodes.get(fst).unwrap().avg_age * (split as f32 / extent as f32) +
                    self.nodes.get(snd).unwrap().avg_age * ((extent - split) as f32 / extent as f32);

                let node = self.nodes.get_mut(index).unwrap();
                node.age = self.counter;
                node.avg_age = avg_age;
            } else {
                unreachable!()
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Rect { x: u32, y: u32, w: u32, h: u32 }

fn nearest_pow_2(x: u32) -> u32 {
    let mut x = x;
    x -= 1;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x += 1;
    x
}

#[test]
fn test_atlas() {
    let mut atlas: Atlas = Atlas::new(1024, 1024);
    atlas.update_counter();
    atlas.insert(0, 10, 10);
    dbg!(&atlas);
    atlas.insert(1, 10, 10);
    dbg!(&atlas);
}
