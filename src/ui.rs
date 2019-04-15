use crate::graphics::*;

use std::borrow::Cow;

pub struct UI {
    graphics: Graphics,
    tree: Vec<Node>,
}

impl UI {
    pub fn new(dpi_factor: f32) -> UI {
        UI {
            graphics: Graphics::new(dpi_factor),
            tree: Vec::new(),
        }
    }

    pub fn graphics(&mut self) -> &mut Graphics {
        &mut self.graphics
    }

    pub fn run(&mut self, width: f32, height: f32, root: &dyn Widget) {
        self.tree = vec![Node {
            rect: Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
            start: 0,
            len: 0,
        }];
        root.layout(&mut Context { graphics: &mut self.graphics, tree: &mut self.tree, index: 0 }, width, height);
        self.update_offsets(0, 0.0, 0.0);
        root.render(&mut Context { graphics: &mut self.graphics, tree: &mut self.tree, index: 0 });
        self.graphics.draw(width, height);
    }

    fn update_offsets(&mut self, i: usize, x: f32, y: f32) {
        let mut node = &mut self.tree[i];
        node.rect.x += x; node.rect.y += y;
        let (x, y) = (node.rect.x, node.rect.y);
        for i in node.start..node.start+node.len {
            self.update_offsets(i, x, y);
        }
    }
}

pub struct Context<'a> {
    graphics: &'a mut Graphics,
    tree: &'a mut Vec<Node>,
    index: usize,
}

impl<'a> Context<'a> {
    pub fn graphics<'b>(&'b mut self) -> &'b mut Graphics {
        self.graphics
    }

    pub fn children(&mut self, children: usize) {
        let start = self.tree.len();
        self.tree.resize(start + children, Node {
            rect: Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
            start: 0,
            len: 0,
        });
        let mut node = &mut self.tree[self.index];
        node.start = start;
        node.len = children;
    }

    pub fn child<'b>(&'b mut self, index: usize) -> Context<'b> {
        let (len, start) = (self.tree[self.index].len, self.tree[self.index].start);
        assert!(index < len, "child index out of range");
        Context {
            graphics: self.graphics,
            tree: self.tree,
            index: start + index,
        }
    }

    pub fn offset(&mut self, index: usize, x: f32, y: f32) {
        let (len, start) = (self.tree[self.index].len, self.tree[self.index].start);
        assert!(index < len, "child index out of range");
        let mut node = &mut self.tree[start + index];
        node.rect.x = x;
        node.rect.y = y;
    }

    pub fn size(&mut self, width: f32, height: f32) {
        let mut node = &mut self.tree[self.index];
        node.rect.width = width;
        node.rect.height = height;
    }

    pub fn rect(&self) -> Rect {
        self.tree[self.index].rect
    }
}

#[derive(Clone)]
pub struct Node {
    rect: Rect,
    start: usize,
    len: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub trait Widget {
    fn layout(&self, context: &mut Context, max_width: f32, max_height: f32);
    fn render(&self, context: &mut Context);
}


pub struct Padding<W: Widget> {
    padding: (f32, f32, f32, f32),
    child: W,
}

impl<W: Widget> Padding<W> {
    pub fn new(left: f32, top: f32, right: f32, bottom: f32, child: W) -> Padding<W> {
        Padding {
            padding: (left, top, right, bottom),
            child: child,
        }
    }

    pub fn uniform(padding: f32, child: W) -> Padding<W> {
        Padding::new(padding, padding, padding, padding, child)
    }
}

impl<W: Widget> Widget for Padding<W> {
    fn layout(&self, context: &mut Context, max_width: f32, max_height: f32) {
        context.children(1);
        self.child.layout(&mut context.child(0), max_width - self.padding.0 - self.padding.2, max_height - self.padding.1 - self.padding.3);
        context.offset(0, self.padding.0, self.padding.1);
        let child_rect = context.child(0).rect();
        context.size(child_rect.width + self.padding.0 + self.padding.2, child_rect.height + self.padding.1 + self.padding.3);
    }

    fn render(&self, context: &mut Context) {
        self.child.render(&mut context.child(0));
    }
}

pub struct Text<'a> {
    text: Cow<'a, str>,
    font: FontId,
    scale: u32,
    color: Color,
}

impl<'a> Text<'a> {
    pub fn new<S: Into<Cow<'a, str>>>(text: S, font: FontId, scale: u32, color: Color) -> Text<'a> {
        Text { text: text.into(), font, scale, color }
    }
}

impl<'a> Widget for Text<'a> {
    fn layout(&self, context: &mut Context, max_width: f32, max_height: f32) {
        let (width, height) = context.graphics().text_size(&self.text, self.font, self.scale);
        context.size(width, height);
    }

    fn render(&self, context: &mut Context) {
        let rect = context.rect();
        context.graphics().text([rect.x, rect.y], &self.text, self.font, self.scale, self.color);
    }
}


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
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
}

pub enum MouseButton {
    Left,
    Middle,
    Right,
}
