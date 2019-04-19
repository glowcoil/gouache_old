use crate::alloc::*;
use crate::graphics::*;

use std::f32;
use std::borrow::Cow;
use std::collections::{HashSet, HashMap};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

macro_rules! id {
    () => { { static ID: u8 = 0; &ID as *const u8 as usize as u64 } }
}

pub struct UI {
    graphics: Graphics,

    tree: Vec<Node>,
    map: HashMap<u64, usize>,
    hover: HashSet<usize>,
    drag: Option<u64>,

    cursor: (f32, f32),
    modifiers: Modifiers,
    mouse: MouseState,
}

impl UI {
    pub fn new(dpi_factor: f32) -> UI {
        UI {
            graphics: Graphics::new(dpi_factor),

            tree: Vec::new(),
            map: HashMap::new(),
            hover: HashSet::new(),
            drag: None,

            cursor: (-1.0, -1.0),
            modifiers: Modifiers::default(),
            mouse: MouseState::default(),
        }
    }

    pub fn graphics(&mut self) -> &mut Graphics {
        &mut self.graphics
    }

    pub fn run(&mut self, width: f32, height: f32, root: &dyn Widget) {
        self.tree = vec![Node {
            id: 0,
            start: 0,
            len: 0,
            rect: Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
            handler: None,
        }];
        let hasher = DefaultHasher::new();
        let mut context = LayoutContext { ui: self, index: 0, parent_hasher: &hasher, hasher: hasher.clone() };
        context.key(0);
        root.layout(context, width, height);
        self.update_offsets(0, 0.0, 0.0);
        self.hover = HashSet::new();
        self.update_hover(0);
        self.update_map(0);
        root.render(RenderContext { ui: self, index: 0 });
        self.graphics.draw(width, height);
    }

    pub fn cursor(&mut self, x: f32, y: f32) {
        self.cursor = (x, y);
    }

    pub fn modifiers(&mut self, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }

    pub fn input(&mut self, input: Input) {
        match input {
            Input::MouseDown(..) | Input::MouseUp(..) | Input::Scroll(..) => {
                if let Some(i) = self.drag.and_then(|id| self.map.get(&id)) {
                    self.fire(*i, input);
                } else {
                    self.mouse_input(0, input);
                }
            }
            Input::KeyDown(..) | Input::KeyUp(..) | Input::Char(..) => {

            }
        }
    }

    fn update_offsets(&mut self, i: usize, x: f32, y: f32) {
        let mut node = &mut self.tree[i];
        node.rect.x += x; node.rect.y += y;
        let (x, y) = (node.rect.x, node.rect.y);
        for i in node.start..node.start+node.len {
            self.update_offsets(i, x, y);
        }
    }

    fn update_hover(&mut self, i: usize) -> bool {
        let (rect, start, len) = {
            let node = &self.tree[i];
            (node.rect, node.start, node.len)
        };
        if rect.contains(self.cursor.0, self.cursor.1) {
            self.hover.insert(i);
            for i in (start..start+len).rev() {
                if self.update_hover(i) { break; }
            }
            true
        } else {
            false
        }
    }

    fn update_map(&mut self, i: usize) {
        let node = &self.tree[i];
        self.map.insert(node.id, i);
        for i in node.start..node.start+node.len {
            self.update_map(i);
        }
    }

    fn mouse_input(&mut self, i: usize, input: Input) -> bool {
        let (rect, start, len) = {
            let node = &self.tree[i];
            (node.rect, node.start, node.len)
        };
        if rect.contains(self.cursor.0, self.cursor.1) {
            for i in (start..start+len).rev() {
                if self.mouse_input(i, input) { return true; }
            }
            self.fire(i, input)
        } else {
            false
        }
    }

    fn fire(&mut self, i: usize, input: Input) -> bool {
        if self.tree[i].handler.is_some() {
            let handler = self.tree[i].handler.take().unwrap();
            let result = handler(EventContext { ui: self, index: i }, input);
            self.tree[i].handler = Some(handler);
            result
        } else {
            false
        }
    }
}

pub struct LayoutContext<'a> {
    ui: &'a mut UI,
    index: usize,
    parent_hasher: &'a DefaultHasher,
    hasher: DefaultHasher,
}

impl<'a> LayoutContext<'a> {
    pub fn graphics<'b>(&'b self) -> &'b Graphics {
        &self.ui.graphics
    }

    pub fn children(&mut self, children: usize) {
        let start = self.ui.tree.len();
        self.ui.tree.resize_with(start + children, || Node {
            id: 0,
            start: 0,
            len: 0,
            rect: Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
            handler: None,
        });
        let mut node = &mut self.ui.tree[self.index];
        node.start = start;
        node.len = children;
    }

    pub fn child<'b>(&'b mut self, index: usize) -> LayoutContext<'b> {
        let (len, start) = (self.ui.tree[self.index].len, self.ui.tree[self.index].start);
        assert!(index < len, "child index out of range");
        let mut context = LayoutContext { ui: self.ui, index: start + index, parent_hasher: &self.hasher, hasher: self.hasher.clone() };
        context.key(index);
        context
    }

    pub fn offset_child(&mut self, index: usize, x: f32, y: f32) {
        let (len, start) = (self.ui.tree[self.index].len, self.ui.tree[self.index].start);
        assert!(index < len, "child index out of range");
        let mut node = &mut self.ui.tree[start + index];
        node.rect.x = x;
        node.rect.y = y;
    }

    pub fn child_size(&self, index: usize) -> (f32, f32) {
        let (len, start) = (self.ui.tree[self.index].len, self.ui.tree[self.index].start);
        assert!(index < len, "child index out of range");
        let rect = self.ui.tree[start + index].rect;
        (rect.width, rect.height)
    }

    pub fn size(&mut self, width: f32, height: f32) {
        let mut node = &mut self.ui.tree[self.index];
        node.rect.width = width;
        node.rect.height = height;
    }

    pub fn drag(&self) -> bool {
        self.ui.drag.map_or(false, |id| id == self.ui.tree[self.index].id)
    }

    pub fn key<K: Hash>(&mut self, key: K) {
        self.hasher = self.parent_hasher.clone();
        key.hash(&mut self.hasher);
        self.ui.tree[self.index].id = self.hasher.finish();
    }

    pub fn listen<F>(&mut self, f: F) where F: Fn(EventContext, Input) -> bool + 'static {
        self.ui.tree[self.index].handler = Some(Box::new(f));
    }
}

pub struct RenderContext<'a> {
    ui: &'a mut UI,
    index: usize,
}

impl<'a> RenderContext<'a> {
    pub fn graphics<'b>(&'b mut self) -> &'b mut Graphics {
        &mut self.ui.graphics
    }

    pub fn child<'b>(&'b mut self, index: usize) -> RenderContext<'b> {
        let (len, start) = (self.ui.tree[self.index].len, self.ui.tree[self.index].start);
        assert!(index < len, "child index out of range");
        RenderContext { ui: self.ui, index: start + index }
    }

    pub fn rect(&self) -> Rect {
        self.ui.tree[self.index].rect
    }

    pub fn hover(&self) -> bool {
        self.ui.hover.contains(&self.index)
    }

    pub fn drag(&self) -> bool {
        self.ui.drag.map_or(false, |id| id == self.ui.tree[self.index].id)
    }

    pub fn listen<F>(&mut self, f: F) where F: Fn(EventContext, Input) -> bool + 'static {
        self.ui.tree[self.index].handler = Some(Box::new(f));
    }
}

pub struct EventContext<'a> {
    ui: &'a mut UI,
    index: usize,
}

impl<'a> EventContext<'a> {
    pub fn rect(&self) -> Rect {
        self.ui.tree[self.index].rect
    }

    pub fn hover(&self) -> bool {
        self.ui.hover.contains(&self.index)
    }

    pub fn drag(&self) -> bool {
        self.ui.drag.map_or(false, |id| id == self.ui.tree[self.index].id)
    }

    pub fn begin_drag(&mut self) {
        self.ui.drag = Some(self.ui.tree[self.index].id);
    }

    pub fn end_drag(&mut self) {
        if let Some(id) = self.ui.drag {
            if id == self.ui.tree[self.index].id {
                self.ui.drag = None;
            }
        }
    }
}

pub struct Node {
    id: u64,
    start: usize,
    len: usize,
    rect: Rect,
    handler: Option<Box<Fn(EventContext, Input) -> bool>>,
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    fn contains(&self, x: f32, y: f32) -> bool {
        self.x <= x && x < self.x + self.width &&
        self.y <= y && y < self.y + self.height
    }
}

pub trait Widget {
    fn layout(&self, context: LayoutContext, max_width: f32, max_height: f32);
    fn render(&self, context: RenderContext);
}


#[derive(Copy, Clone)]
pub struct Row<'a> {
    spacing: f32,
    children: &'a [&'a dyn Widget],
}

impl<'a> Row<'a> {
    pub fn new(arena: &'a Arena, spacing: f32, children: &[&'a dyn Widget]) -> &'a Row<'a> {
        arena.alloc(Row { spacing, children: arena.alloc_slice(children) })
    }
}

impl<'a> Widget for Row<'a> {
    fn layout(&self, mut context: LayoutContext, max_width: f32, max_height: f32) {
        context.children(self.children.len());
        let mut x: f32 = 0.0;
        let mut height: f32 = 0.0;
        for (i, child) in self.children.iter().enumerate() {
            child.layout(context.child(i), f32::INFINITY, max_height);
            context.offset_child(i, x, 0.0);
            let (child_width, child_height) = context.child_size(i);
            x += child_width + self.spacing;
            height = height.max(child_height);
        }
        context.size(x - self.spacing, height)
    }

    fn render(&self, mut context: RenderContext) {
        let mut i = 0;
        for (i, child) in self.children.iter().enumerate() {
            child.render(context.child(i));
        }
    }
}

#[derive(Copy, Clone)]
pub struct Padding<'a> {
    padding: (f32, f32, f32, f32),
    child: &'a dyn Widget,
}

impl<'a> Padding<'a> {
    pub fn new(arena: &'a Arena, left: f32, top: f32, right: f32, bottom: f32, child: &'a dyn Widget) -> &'a Padding<'a> {
        arena.alloc(Padding {
            padding: (left, top, right, bottom),
            child: child,
        })
    }

    pub fn uniform(arena: &'a Arena, padding: f32, child: &'a dyn Widget) -> &'a Padding<'a> {
        Padding::new(arena, padding, padding, padding, padding, child)
    }
}

impl<'a> Widget for Padding<'a> {
    fn layout(&self, mut context: LayoutContext, max_width: f32, max_height: f32) {
        context.children(1);
        self.child.layout(context.child(0), max_width - self.padding.0 - self.padding.2, max_height - self.padding.1 - self.padding.3);
        context.offset_child(0, self.padding.0, self.padding.1);
        let (child_width, child_height) = context.child_size(0);
        context.size(child_width + self.padding.0 + self.padding.2, child_height + self.padding.1 + self.padding.3);
    }

    fn render(&self, mut context: RenderContext) {
        self.child.render(context.child(0));
    }
}

#[derive(Copy, Clone)]
pub struct Text<'a> {
    text: &'a str,
    font: FontId,
    scale: u32,
    color: Color,
}

impl<'a> Text<'a> {
    pub fn new(arena: &'a Arena, text: &'a str, font: FontId, scale: u32, color: Color) -> &'a Text<'a> {
        arena.alloc(Text { text, font, scale, color })
    }
}

impl<'a> Widget for Text<'a> {
    fn layout(&self, mut context: LayoutContext, max_width: f32, max_height: f32) {
        let (width, height) = context.graphics().text_size(self.text, self.font, self.scale);
        context.size(width, height);
    }

    fn render(&self, mut context: RenderContext) {
        let rect = context.rect();
        context.graphics().text([rect.x, rect.y], self.text, self.font, self.scale, self.color);
    }
}

#[derive(Copy, Clone)]
pub struct Button<'a> {
    child: &'a dyn Widget,
}

impl<'a> Button<'a> {
    pub fn new(arena: &'a Arena, child: &'a dyn Widget) -> &'a Button<'a> {
        arena.alloc(Button { child: Padding::uniform(arena, 5.0, child) })
    }
}

impl<'a> Widget for Button<'a> {
    fn layout(&self, mut context: LayoutContext, max_width: f32, max_height: f32) {
        context.children(1);
        self.child.layout(context.child(0), max_width, max_height);
        let (child_width, child_height) = context.child_size(0);
        context.size(child_width, child_height);
    }

    fn render(&self, mut context: RenderContext) {
        let color = if context.drag() { Color::rgba(0.2, 0.2, 0.4, 1.0) } else if context.hover() { Color::rgba(0.8, 0.8, 0.9, 1.0) } else { Color::rgba(0.5, 0.5, 0.7, 1.0) };
        let rect = context.rect();
        context.graphics().round_rect_fill([rect.x, rect.y], [rect.width, rect.height], 5.0, color);
        self.child.render(context.child(0));
        context.listen(|mut context, input| {
            match input {
                Input::MouseDown(MouseButton::Left) => {
                    context.begin_drag();
                    true
                }
                Input::MouseUp(MouseButton::Left) => {
                    if context.hover() {
                        println!("click");
                    }
                    context.end_drag();
                    true
                }
                _ => { false }
            }
        });
    }
}


pub struct MouseState {
    left: bool,
    middle: bool,
    right: bool,
}

impl Default for MouseState {
    fn default() -> MouseState {
        MouseState { left: false, middle: false, right: false }
    }
}

pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl Default for Modifiers {
    fn default() -> Modifiers {
        Modifiers { shift: false, ctrl: false, alt: false, meta: false }
    }
}

#[derive(Copy, Clone)]
pub enum Input {
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    Scroll(f32, f32),
    KeyDown(Key),
    KeyUp(Key),
    Char(char),
}

#[derive(Copy, Clone)]
pub enum Key {
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    GraveAccent,
    Minus,
    Equals,
    LeftBracket,
    RightBracket,
    Backslash,
    Semicolon,
    Apostrophe,
    Comma,
    Period,
    Slash,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    PrintScreen,
    ScrollLock,
    Pause,
    Backspace,
    Tab,
    CapsLock,
    Enter,
    Space,
    Insert,
    Delete,
    PageUp,
    PageDown,
    Home,
    End,
    Left,
    Right,
    Up,
    Down,
    NumLock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadDecimal,
    NumpadDivide,
    NumpadMultiply,
    NumpadSubtract,
    NumpadAdd,
    NumpadEnter,
    NumpadEquals,
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftMeta,
    RightShift,
    RightControl,
    RightAlt,
    RightMeta,
}

#[derive(Copy, Clone)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}
