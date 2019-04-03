use crate::render::*;
use crate::alloc::*;

use std::f32::consts::PI;

const TOLERANCE: f32 = 0.1;

pub struct Graphics {
    dpi_factor: f32,
    renderer: Renderer,
    fonts: Slab<font_rs::font::Font<'static>>,
    atlas: Atlas,
    atlas_tex: TexId,

    layers: Vec<(usize, usize)>,
    stack: Vec<usize>,
    items: Vec<DisplayItem>,
    glyphs: Vec<Glyph>,
    paths: Vec<PathSegment>,
}

impl Graphics {
    pub fn new(dpi_factor: f32) -> Graphics {
        let mut renderer = Renderer::new();
        let atlas_tex = renderer.create_tex(TexFormat::A, 1024, 1024, &[0; 1024*1024]);
        Graphics {
            dpi_factor,
            renderer,
            fonts: Slab::new(),
            atlas: Atlas::new(1024, 1024),
            atlas_tex,

            layers: Vec::new(),
            stack: Vec::new(),
            items: Vec::new(),
            glyphs: Vec::new(),
            paths: Vec::new(),
        }
    }

    pub fn add_font(&mut self, bytes: &'static [u8]) -> FontId {
        self.fonts.insert(font_rs::font::parse(bytes).unwrap())
    }

    pub fn remove_font(&mut self, font: FontId) {
        self.fonts.remove(font);
    }

    pub fn clear(&mut self, color: Color) {
        self.renderer.clear(color.to_linear());
    }

    pub fn draw(&mut self, width: f32, height: f32) {
        let mut glyphs = Vec::new();
        let mut paths = Vec::new();

        for item in self.items.iter() {
            match item {
                DisplayItem::Glyphs(color, start, end) => {
                    glyphs.push((color, &self.glyphs[*start..*end]));
                }
                DisplayItem::FillPath(color, start, end) => {
                    paths.push((color, &self.paths[*start..*end]));
                }
            }
        }

        let mut path_verts: Vec<Vertex> = Vec::new();
        let mut path_indices: Vec<u16> = Vec::new();
        for (color, path) in paths {
            let col = color.to_linear();
            let mut col_alpha = col;
            col_alpha[3] = 0.0;

            let mut verts = Vec::new();
            for (i, PathSegment(pos, segment)) in path.iter().enumerate() {
                match segment {
                    SegmentType::Line => {
                        verts.push(*pos);
                    }
                    SegmentType::Arc(radius, start_angle, end_angle) => {
                        let PathSegment(next, _) = path[(i+1) % path.len()];
                        let segments: u16 = (((end_angle - start_angle).abs() / (1.0 - TOLERANCE / radius).acos()).ceil() as u16).max(4);
                        let arc = (end_angle - start_angle) / segments as f32;
                        let rotor = [arc.cos(), -arc.sin()];
                        let mut angle = [start_angle.cos(), -start_angle.sin()];
                        let center = [pos[0] - radius * angle[0], pos[1] - radius * angle[1]];
                        for _ in 0..segments {
                            verts.push([center[0] + radius * angle[0], center[1] + radius * angle[1]]);
                            angle = [rotor[0] * angle[0] - rotor[1] * angle[1], rotor[0] * angle[1] + rotor[1] * angle[0]];
                        }
                    }
                }
            }
            let path_start = path_verts.len();
            for i in 0..verts.len() {
                let prev = verts[(i+verts.len()-1)%verts.len()];
                let curr = verts[i];
                let next = verts[(i+1)%verts.len()];
                let prev_normal = normalized([prev[1] - curr[1], curr[0] - prev[0]]);
                let next_normal = normalized([curr[1] - next[1], next[0] - curr[0]]);
                let normal = normalized([(prev_normal[0] + next_normal[0]) / 2.0, (prev_normal[1] + next_normal[1]) / 2.0]);
                let (inner_x, inner_y) = pixel_to_ndc(curr[0] - 0.5 * normal[0], curr[1] - 0.5 * normal[1], width, height);
                let (outer_x, outer_y) = pixel_to_ndc(curr[0] + 0.5 * normal[0], curr[1] + 0.5 * normal[1], width, height);
                path_verts.push(Vertex { pos: [inner_x, inner_y, 0.0], col });
                path_verts.push(Vertex { pos: [outer_x, outer_y, 0.0], col: col_alpha });
            }
            for i in 1..verts.len().saturating_sub(1) {
                path_indices.extend_from_slice(&[path_start as u16, (path_start + 2*i) as u16, (path_start + 2*i + 2) as u16]);
            }
            for i in 0..verts.len() {
                path_indices.extend_from_slice(&[
                    (path_start + 2*i) as u16, (path_start + 2*i + 1) as u16, (path_start + 2*((i+1)%verts.len()) + 1) as u16,
                    (path_start + 2*i) as u16, (path_start + 2*((i+1)%verts.len()) + 1) as u16, (path_start + 2*((i+1)%verts.len())) as u16,
                ]);
            }
        }
        self.renderer.draw(&path_verts, &path_indices);

        let mut glyph_verts: Vec<VertexUV> = Vec::new();
        let mut glyph_indices: Vec<u16> = Vec::new();
        self.atlas.update_counter();

        for (color, glyph_list) in glyphs {
            let col = color.to_linear();
            for glyph in glyph_list.iter() {
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
                let (u1, v1) = (rect.x as f32 / self.atlas.width as f32, (rect.y + rect.h) as f32 / self.atlas.height as f32);
                let (u2, v2) = ((rect.x + rect.w) as f32 / self.atlas.width as f32, rect.y as f32 / self.atlas.height as f32);
                let (x1, y1) = pixel_to_ndc(glyph.pos[0], glyph.pos[1], width, height);
                let (x2, y2) = pixel_to_ndc(glyph.pos[0] + rect.w as f32, glyph.pos[1] + rect.h as f32, width, height);
                glyph_verts.extend_from_slice(&[VertexUV {
                    pos: [x1, y1, 0.0],
                    col,
                    uv: [u1, v1],
                }, VertexUV {
                    pos: [x2, y1, 0.0],
                    col,
                    uv: [u2, v1],
                }, VertexUV {
                    pos: [x2, y2, 0.0],
                    col,
                    uv: [u2, v2],
                }, VertexUV {
                    pos: [x1, y2, 0.0],
                    col,
                    uv: [u1, v2],
                }]);
                glyph_indices.extend_from_slice(&[i, i+1, i+2, i, i+2, i+3]);
            }
        }
        self.renderer.draw_tex(&glyph_verts, &glyph_indices, self.atlas_tex);

        self.layers = Vec::new();
        self.stack = Vec::new();
        self.items = Vec::new();
        self.glyphs = Vec::new();
        self.paths = Vec::new();
    }

    pub fn text(&mut self, pos: [f32; 2], text: &str, font_id: FontId, scale: u32, color: Color) {
        let font = self.fonts.get(font_id).unwrap();
        let mut pos = pos;
        let start = self.glyphs.len();
        self.glyphs.reserve(text.len());
        for c in text.chars() {
            let glyph = font.lookup_glyph_id(c as u32).unwrap();
            let h_metrics = font.get_h_metrics(glyph, scale).unwrap();
            let v_metrics = font.get_v_metrics(scale).unwrap();
            if let Some(bbox) = font.get_bbox(glyph, scale) {
                self.glyphs.push(Glyph {
                    id: GlyphId { font: font_id, scale, glyph },
                    pos: [pos[0] + bbox.l as f32, pos[1] + bbox.t as f32 + v_metrics.ascent as f32],
                });
            }
            pos[0] += h_metrics.advance_width;
        }
        self.items.push(DisplayItem::Glyphs(color, start, self.glyphs.len()));
    }

    pub fn rect_fill(&mut self, pos: [f32; 2], size: [f32; 2], color: Color) {
        let start = self.paths.len();
        self.paths.extend_from_slice(&[
            PathSegment([pos[0], pos[1]], SegmentType::Line),
            PathSegment([pos[0], pos[1] + size[1]], SegmentType::Line),
            PathSegment([pos[0] + size[0], pos[1] + size[1]], SegmentType::Line),
            PathSegment([pos[0] + size[0], pos[1]], SegmentType::Line),
        ]);
        self.items.push(DisplayItem::FillPath(color, start, self.paths.len()));
    }

    pub fn round_rect_fill(&mut self, pos: [f32; 2], size: [f32; 2], radius: f32, color: Color) {
        let start = self.paths.len();
        self.paths.extend_from_slice(&[
            PathSegment([pos[0] + radius, pos[1]], SegmentType::Arc(radius, PI/2.0, PI)),
            PathSegment([pos[0], pos[1] + radius], SegmentType::Line),
            PathSegment([pos[0], pos[1] + size[1] - radius], SegmentType::Arc(radius, PI, 3.0*PI/2.0)),
            PathSegment([pos[0] + radius, pos[1] + size[1]], SegmentType::Line),
            PathSegment([pos[0] + size[0] - radius, pos[1] + size[1]], SegmentType::Arc(radius, 3.0*PI/2.0, 2.0*PI)),
            PathSegment([pos[0] + size[0], pos[1] + size[1] - radius], SegmentType::Line),
            PathSegment([pos[0] + size[0], pos[1] + radius], SegmentType::Arc(radius, 0.0, PI/2.0)),
            PathSegment([pos[0] + size[0] - radius, pos[1]], SegmentType::Line),
        ]);
        self.items.push(DisplayItem::FillPath(color, start, self.paths.len()));
    }

    pub fn circle_fill(&mut self, pos: [f32; 2], radius: f32, color: Color) {
        let start = self.paths.len();
        self.paths.extend_from_slice(&[
            PathSegment([pos[0] + radius, pos[1]], SegmentType::Arc(radius, 0.0, 2.0*PI)),
        ]);
        self.items.push(DisplayItem::FillPath(color, start, self.paths.len()));
    }
}

#[inline]
fn pixel_to_ndc(x: f32, y: f32, screen_width: f32, screen_height: f32) -> (f32, f32) {
    (2.0 * (x / screen_width as f32 - 0.5), 2.0 * (1.0 - y / screen_height as f32 - 0.5))
}

#[inline]
fn distance(p1: [f32; 2], p2: [f32; 2]) -> f32 {
    ((p2[0] - p1[0]) * (p2[0] - p1[0]) + (p2[1] - p1[1]) * (p2[1] - p1[1])).sqrt()
}

#[inline]
fn length(p: [f32; 2]) -> f32 {
    (p[0] * p[0] + p[1] * p[1]).sqrt()
}

#[inline]
fn normalized(p: [f32; 2]) -> [f32; 2] {
    let len = length(p);
    [p[0] / len, p[1] / len]
}


#[derive(Copy, Clone)]
pub enum DisplayItem {
    Glyphs(Color, usize, usize),
    FillPath(Color, usize, usize),
}

#[derive(Copy, Clone)]
pub struct Color {
    r: f32, g: f32, b: f32, a: f32
}

impl Color {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    fn to_linear(&self) -> [f32; 4] {
        [srgb_to_linear(self.r), srgb_to_linear(self.g), srgb_to_linear(self.b), self.a]
    }
}

fn srgb_to_linear(x: f32) -> f32 {
    if x < 0.04045 { x / 12.92 } else { ((x + 0.055)/1.055).powf(2.4)  }
}

#[derive(Copy, Clone)]
pub struct Glyph {
    id: GlyphId,
    pos: [f32; 2],
}

#[derive(Copy, Clone)]
struct PathSegment([f32; 2], SegmentType);

#[derive(Copy, Clone)]
enum SegmentType {
    Line,
    Arc(f32, f32, f32),
}


pub type FontId = usize;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct GlyphId {
    font: FontId,
    scale: u32,
    glyph: u16,
}

struct Atlas {
    width: u32,
    height: u32,
    rows: Slab<Row>,
    rows_by_height: Vec<usize>,
    next_y: u32,
    map: std::collections::HashMap<GlyphId, Entry>,
    counter: usize,
}

struct Row {
    y: u32,
    height: u32,
    glyphs: Slab<AtlasGlyph>,
    next_x: u32,
    last_used: usize,
}

impl Row {
    fn new(y: u32, height: u32) -> Row {
        Row { y, height, glyphs: Slab::new(), next_x: 0, last_used: 0 }
    }
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
            rows: Slab::new(),
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
            let row_index = self.rows.insert(Row::new(self.next_y, row_height));
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
                if row.last_used == self.counter { continue; }
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
            let row_index = self.add_row(Row::new(y, row_height));
            if best_height > row_height {
                self.add_row(Row::new(y + row_height, best_height - row_height));
            }
            Some(row_index)
        } else {
            None
        }
    }

    fn add_row(&mut self, row: Row) -> usize {
        let height = row.height;
        let row_index = self.rows.insert(row);
        let index = self.rows_by_height
            .binary_search_by_key(&height, |row| self.rows.get(*row).unwrap().height)
            .unwrap_or_else(|i| i);
        self.rows_by_height.insert(index, row_index);
        row_index
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
