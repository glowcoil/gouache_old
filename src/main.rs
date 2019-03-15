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

    let tex = renderer.create_tex(TexFormat::A, 4, 4, &[
        255, 127, 255, 127,
        255, 127, 255, 127,
        255, 127, 255, 127,
        255, 127, 255, 127,
    ]);


    let font = font_rs::font::parse(include_bytes!("../HelveticaNeue.ttf")).unwrap();
    let mut atlas = Atlas::new(128, 128);
    atlas.update_counter();
    let atlas_tex = renderer.create_tex(TexFormat::A, 128, 128, &[0; 128*128]);
    for c in "qwertayuiopasdfghjklzxcvbnm1234567890`~!@#$%^&*()_+-={{}}[]\\|,./<>?".chars() {
        let glyph_id = font.lookup_glyph_id(c as u32).unwrap();
        let bbox = font.get_bbox(glyph_id, 16).unwrap();
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

struct Atlas {
    width: u32,
    height: u32,
    rows: alloc::Slab<Row>,
    rows_by_height: Vec<usize>,
    next_y: u32,
    map: std::collections::HashMap<GlyphId, Entry>,
    counter: usize,
}

struct Row {
    y: u32,
    height: u32,
    glyphs: alloc::Slab<Glyph>,
    next_x: u32,
    last_used: usize,
}

#[derive(Debug)]
struct Glyph {
    x: u32,
    width: u32,
    height: u32,
    glyph_id: GlyphId,
}

#[derive(Debug)]
struct Entry {
    row: usize,
    glyph: usize,
}

impl Atlas {
    fn new(width: u32, height: u32) -> Atlas {
        Atlas {
            width,
            height,
            rows: alloc::Slab::new(),
            rows_by_height: Vec::new(),
            next_y: 0,
            map: std::collections::HashMap::new(),
            counter: 0,
        }
    }

    fn update_counter(&mut self) {
        self.counter += 1;
    }

    fn get_cached(&mut self, glyph_id: GlyphId) -> Option<Rect> {
        if let Some(&Entry { row, glyph }) = self.map.get(&glyph_id) {
            let row = self.rows.get_mut(row).unwrap();
            row.last_used = self.counter;
            let glyph = row.glyphs.get_mut(glyph).unwrap();
            Some(Rect { x: glyph.x, y: row.y, w: glyph.width, h: glyph.height })
        } else {
            None
        }
    }

    fn insert(&mut self, glyph_id: GlyphId, width: u32, height: u32) -> Option<Rect> {
        if width > self.width || height > self.height { return None; }

        let row_index = self.find_row(width, height);
        if row_index.is_none() { return None; }
        let row_index = row_index.unwrap();

        let mut row = self.rows.get_mut(row_index).unwrap();
        let x = row.next_x;
        let glyph = row.glyphs.insert(Glyph {
            x,
            width,
            height,
            glyph_id,
        });
        row.next_x += width;
        row.last_used = self.counter;

        self.map.insert(glyph_id, Entry { row: row_index, glyph });

        Some(Rect { x, y: row.y, w: width, h: height })
    }

    fn find_row(&mut self, width: u32, height: u32) -> Option<usize> {
        let row_height = nearest_pow_2(height);
        // this logic is to ensure that the search finds the first of a sequence of equal elements
        let mut index = self.rows_by_height
            .binary_search_by_key(&(2 * row_height - 1), |row| 2 * self.rows.get(*row).unwrap().height)
            .unwrap_err();
        // try to find an existing tightly sized row
        while index < self.rows_by_height.len() && row_height == self.rows.get(self.rows_by_height[index]).unwrap().height {
            if width <= self.width - self.rows.get(self.rows_by_height[index]).unwrap().next_x {
                return Some(self.rows_by_height[index]);
            }
            index += 1;
        }
        // if there is no exact match, try to add a tightly sized row
        if let Some(new_row_index) = self.try_add_row(index, row_height) {
            return Some(new_row_index);
        }
        // search rows for room starting at tightest fit
        for i in index..self.rows_by_height.len() {
            if width <= self.width - self.rows.get(self.rows_by_height[i]).unwrap().next_x {
                return Some(self.rows_by_height[i]);
            }
        }
        // if we ran out of rows, try to add a new row
        if let Some(row_index) = self.try_add_row(index, row_height) {
            return Some(row_index);
        }
        // need to overwrite some rows
        if let Some(row_index) = self.try_overwrite_rows(row_height) {
            return Some(row_index);
        }
        None
    }

    fn try_add_row(&mut self, index: usize, row_height: u32) -> Option<usize> {
        if row_height <= self.height - self.next_y {
            let row_index = self.rows.insert(Row {
                y: self.next_y,
                height: row_height,
                glyphs: alloc::Slab::new(),
                next_x: 0,
                last_used: 0,
            });
            self.next_y += row_height;
            self.rows_by_height.insert(index, row_index);
            Some(row_index)
        } else {
            None
        }
    }

    fn try_overwrite_rows(&mut self, row_height: u32) -> Option<usize> {
        let mut rows_by_y = self.rows_by_height.clone();
        rows_by_y.sort_by_key(|row| self.rows.get(*row).unwrap().y);
        let mut best_i = 0;
        let mut best_height = 0;
        let mut best_num_rows = 0;
        let mut best_last_used = self.counter as f32;
        'row: for i in 0..rows_by_y.len() {
            let mut num_rows = 0;
            let mut rows_height = 0;
            let mut last_used_sum = 0;
            while row_height > rows_height && i + num_rows < rows_by_y.len() {
                let row = self.rows.get(rows_by_y[i]).unwrap();
                if row.last_used == self.counter { continue 'row; }
                num_rows += 1;
                rows_height += row.height;
                last_used_sum += row.last_used;
            }
            if row_height <= rows_height {
                let last_used_avg = last_used_sum as f32 / num_rows as f32;
                if last_used_avg < best_last_used {
                    best_i = i;
                    best_height = rows_height;
                    best_num_rows = num_rows;
                    best_last_used = last_used_avg;
                }
            }
        }
        if best_height > 0 {
            let y = self.rows.get(rows_by_y[best_i]).unwrap().y;
            for row_index in &rows_by_y[best_i..(best_i + best_num_rows)] {
                self.rows_by_height.remove(*row_index);
                let row = self.rows.remove(*row_index).unwrap();
                for glyph in row.glyphs.iter() {
                    self.map.remove(&glyph.glyph_id);
                }
            }
            let row_index = self.rows.insert(Row {
                y,
                height: best_height,
                glyphs: alloc::Slab::new(),
                next_x: 0,
                last_used: 0,
            });
            let index = self.rows_by_height
                .binary_search_by_key(&best_height, |row| self.rows.get(*row).unwrap().height)
                .unwrap_or_else(|i| i);
            self.rows_by_height.insert(index, row_index);
            Some(row_index)
        } else {
            None
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
