use crate::graphics::*;

pub struct UI {
    graphics: Graphics,
}

impl UI {
    pub fn new(dpi_factor: f32) -> UI {
        UI {
            graphics: Graphics::new(dpi_factor),
        }
    }

    pub fn graphics(&mut self) -> &mut Graphics {
        &mut self.graphics
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

pub trait Widget {
    fn size(&self, ui: &mut UI, bounds: Rect) -> Rect;
    fn draw(&self, ui: &mut UI, bounds: Rect) -> Rect;
}


pub struct Padding<W: Widget> {
    padding: Rect,
    child: W,
}

impl<W: Widget> Padding<W> {
    pub fn new(left: f32, top: f32, right: f32, bottom: f32, child: W) -> Padding<W> {
        Padding {
            padding: Rect { left, top, right, bottom },
            child: child,
        }
    }

    pub fn uniform(padding: f32, child: W) -> Padding<W> {
        Padding::new(padding, padding, padding, padding, child)
    }

    fn inner_rect(&self, rect: Rect) -> Rect {
        Rect {
            left: rect.left + self.padding.left, top: rect.top + self.padding.top,
            right: rect.right - self.padding.right, bottom: rect.bottom - self.padding.bottom,
        }
    }

    fn outer_rect(&self, rect: Rect) -> Rect {
        Rect {
            left: rect.left - self.padding.left, top: rect.top - self.padding.top,
            right: rect.right + self.padding.right, bottom: rect.bottom + self.padding.bottom,
        }
    }
}

impl<W: Widget> Widget for Padding<W> {
    fn size(&self, ui: &mut UI, bounds: Rect) -> Rect {
        self.outer_rect(self.child.size(ui, self.inner_rect(bounds)))
    }

    fn draw(&self, ui: &mut UI, bounds: Rect) -> Rect {
        self.outer_rect(self.child.draw(ui, self.inner_rect(bounds)))
    }
}

pub struct Text<'a> {
    text: &'a str,
    font: FontId,
    scale: u32,
    color: Color,
}

impl<'a> Text<'a> {
    pub fn new(text: &'a str, font: FontId, scale: u32, color: Color) -> Text<'a> {
        Text { text, font, scale, color }
    }

    fn rect(bounds: Rect, width: f32, height: f32) -> Rect {
        let mut rect = bounds;
        if rect.left.is_infinite() {
            rect.left = rect.right - width;
        } else if rect.right.is_infinite() {
            rect.right = rect.left + width;
        }
        if rect.top.is_infinite() {
            rect.top = rect.bottom - height;
        } else if rect.bottom.is_infinite() {
            rect.bottom = rect.top + height;
        }
        rect
    }
}

impl<'a> Widget for Text<'a> {
    fn size(&self, ui: &mut UI, bounds: Rect) -> Rect {
        let (width, height) = ui.graphics().text_size(self.text, self.font, self.scale);
        Self::rect(bounds, width, height)
    }

    fn draw(&self, ui: &mut UI, bounds: Rect) -> Rect {
        let (width, height) = ui.graphics().text_size(self.text, self.font, self.scale);
        let rect = Self::rect(bounds, width, height);
        let x_space = (rect.right - rect.left) - width;
        let x = if x_space > 0.0 { x_space / 2.0 } else { rect.left };
        let y_space = (rect.bottom - rect.top) - height;
        let y = if y_space > 0.0 { y_space / 2.0 } else { rect.top };
        ui.graphics().text([x, y], self.text, self.font, self.scale, self.color);
        rect
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
