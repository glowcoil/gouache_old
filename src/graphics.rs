use crate::render::*;
use crate::alloc;

pub struct Graphics {
    renderer: Renderer,
    fonts: alloc::Slab<font_rs::font::Font<'static>>,
    atlas: Atlas,
    atlas_tex: TexId,
    glyphs: Vec<Glyph>,
}

impl Graphics {
    pub fn new() -> Graphics {
        let mut renderer = Renderer::new();
        let atlas_tex = renderer.create_tex(TexFormat::A, 1024, 1024, &[0; 1024*1024]);
        Graphics {
            renderer,
            fonts: alloc::Slab::new(),
            atlas: Atlas::new(1024, 1024),
            atlas_tex,
            glyphs: Vec::new(),
        }
    }

    pub fn add_font(&mut self, bytes: &'static [u8]) -> FontId {
        self.fonts.insert(font_rs::font::parse(bytes).unwrap())
    }

    pub fn remove_font(&mut self, font: FontId) {
        self.fonts.remove(font);
    }

    pub fn draw(&mut self) {
        let mut glyph_verts: Vec<VertexUV> = Vec::new();
        let mut glyph_indices: Vec<u16> = Vec::new();
        self.atlas.update_counter();

        for glyph in self.glyphs.drain(0..) {
            let rect = if let Some(rect) = self.atlas.get_cached(glyph.id) {
                rect
            } else {
                let font = self.fonts.get(glyph.id.font).unwrap();
                let bbox = font.get_bbox(glyph.id.glyph, glyph.id.scale).unwrap();
                let rect = self.atlas.insert(glyph.id, bbox.width() as u32, bbox.height() as u32).unwrap();
                let rendered = font.render_glyph(glyph.id.glyph, glyph.id.scale).unwrap();
                self.renderer.update_tex(self.atlas_tex, rect.x as usize, rect.y as usize, rendered.width as usize, rendered.height as usize, &rendered.data);
                rect
            };

            let i = glyph_verts.len() as u16;
            let x1 = rect.x as f32 / self.atlas.width as f32;
            let x2 = (rect.x + rect.w) as f32 / self.atlas.width as f32;
            let y1 = rect.y as f32 / self.atlas.width as f32;
            let y2 = (rect.y + rect.h) as f32 / self.atlas.height as f32;
            glyph_verts.extend(&[VertexUV {
                pos: [glyph.pos[0], glyph.pos[1], glyph.pos[2]],
                col: [1.0, 1.0, 1.0, 1.0],
                uv: [x1, y1],
            }, VertexUV {
                pos: [glyph.pos[0] + rect.w as f32 / 400.0, glyph.pos[1], glyph.pos[2]],
                col: [1.0, 1.0, 1.0, 1.0],
                uv: [x2, y1],
            }, VertexUV {
                pos: [glyph.pos[0] + rect.w as f32 / 400.0, glyph.pos[1] + rect.h as f32 / 300.0, glyph.pos[2]],
                col: [1.0, 1.0, 1.0, 1.0],
                uv: [x2, y2],
            }, VertexUV {
                pos: [glyph.pos[0], glyph.pos[1] + rect.h as f32 / 300.0, glyph.pos[2]],
                col: [1.0, 1.0, 1.0, 1.0],
                uv: [x1, y2],
            }]);
            glyph_indices.extend(&[i, i+1, i+2, i, i+2, i+3]);
        }
        self.renderer.draw_tex(&glyph_verts, &glyph_indices, self.atlas_tex);
    }

    pub fn paint<'a>(&'a mut self) -> Paint<'a> {
        Paint {
            graphics: self,
        }
    }
}

pub struct Paint<'a> {
    graphics: &'a mut Graphics,
}

impl<'a> Paint<'a> {
    pub fn glyph(&mut self, pos: [f32; 3], c: char, font: FontId, scale: u32) {
        let glyph = self.graphics.fonts.get(font).unwrap().lookup_glyph_id(c as u32).unwrap();
        self.graphics.glyphs.push(Glyph { id: GlyphId { font, scale, glyph }, pos });
    }

    pub fn text(&mut self, pos: [f32; 3], text: &str, font_id: FontId, scale: u32) {
        let mut pos = pos;
        let font = self.graphics.fonts.get(font_id).unwrap();
        for c in text.chars() {
            let glyph = font.lookup_glyph_id(c as u32).unwrap();
            let h_metrics = font.get_h_metrics(glyph, scale).unwrap();
            if let Some(bbox) = font.get_bbox(glyph, scale) {
                self.graphics.glyphs.push(Glyph {
                    id: GlyphId { font: font_id, scale, glyph },
                    pos: [pos[0] + bbox.l as f32 / 400.0, pos[1] - bbox.b as f32 / 300.0, pos[2]],
                });
            }
            pos[0] += h_metrics.advance_width / 400.0;
        }
    }
}


pub type FontId = usize;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct GlyphId {
    font: FontId,
    scale: u32,
    glyph: u16,
}

#[derive(Copy, Clone)]
struct Glyph {
    id: GlyphId,
    pos: [f32; 3],
}


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
    glyphs: alloc::Slab<AtlasGlyph>,
    next_x: u32,
    last_used: usize,
}

#[derive(Debug)]
struct AtlasGlyph {
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
        let glyph = row.glyphs.insert(AtlasGlyph {
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
        let mut best_last_used = self.counter as f32 + 1.0;
        for i in 0..rows_by_y.len() {
            let mut num_rows = 0;
            let mut rows_height = 0;
            let mut last_used_sum = 0;
            while row_height > rows_height && i + num_rows < rows_by_y.len() {
                let row = self.rows.get(rows_by_y[i]).unwrap();
                // if row.last_used == self.counter { continue; }
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
