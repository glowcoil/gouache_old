use crate::arena::Arena;
use crate::render::{Vertex, Cmd};

pub struct UI {

}

impl UI {
    pub fn new() -> UI {
        UI {}
    }

    pub fn frame<'a, 'b>(&'a mut self) -> Frame<'a, 'b> {
        Frame {
            ui: self,
            vertices: Vec::new(),
            indices: Vec::new(),
            glyphs: Vec::new(),
        }
    }
}

pub struct Frame<'a, 'b> {
    ui: &'a mut UI,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    glyphs: Vec<rusttype::PositionedGlyph<'b>>,
}

impl<'a, 'b> Frame<'a, 'b> {
    pub fn glyph(&mut self, glyph: rusttype::PositionedGlyph<'b>) {
        self.glyphs.push(glyph);
    }

    pub fn render(self) -> Vec<Cmd<'b>> {
        let mut cmds = Vec::new();
        cmds.push(Cmd::Draw { vertices: self.vertices, indices: self.indices });
        cmds.push(Cmd::DrawGlyphs { glyphs: self.glyphs });
        cmds
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
