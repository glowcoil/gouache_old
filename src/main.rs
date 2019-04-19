#[macro_use]
mod ui;
mod graphics;
mod render;
mod alloc;

use alloc::*;
use graphics::*;
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
        .with_title("gouache");
    let context = glutin::ContextBuilder::new();
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe { gl_window.make_current().unwrap(); }
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    let dpi_factor = gl_window.get_hidpi_factor();

    let mut ui = UI::new(dpi_factor as f32);
    let font = ui.graphics().add_font(include_bytes!("../res/sawarabi-gothic-medium.ttf"));

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

        let size = gl_window.get_inner_size().unwrap();

        // let mut graphics = ui.graphics();
        // graphics.clear(Color::rgba(0.1, 0.15, 0.2, 1.0));
        // graphics.text([0.0, 0.0], "Jackdaws love my big sphinx of quartz.", font, 14, Color::rgba(0.8, 0.8, 0.8, 1.0));
        // graphics.text([700.0, 580.0], &fps_text, font, 14, Color::rgba(1.0, 1.0, 1.0, 1.0));
        // graphics.round_rect_fill([100.0, 100.0], [100.0, 100.0], 5.0, Color::rgba(0.8, 0.5, 0.0, 1.0));
        // graphics.circle_fill([225.0, 225.0], 101.0, Color::rgba(0.5, 0.25, 1.0, 0.75));
        // graphics.circle_fill([300.0, 100.0], 150.0, Color::rgba(0.0, 0.5, 1.0, 0.5));
        // graphics.draw(size.width as f32, size.height as f32);

        ui.graphics().clear(Color::rgba(0.1, 0.15, 0.2, 1.0));
        let xs = [1, 2, 3];
        let a = Arena::with_capacity(1024);
        let tree = Padding::uniform(&a, 20.0, Row::new(&a, 10.0, &[
            Row::new(&a, 10.0, &[
                Text::new(&a, &fps_text, font, 14, Color::rgba(1.0, 1.0, 1.0, 1.0)),
                Text::new(&a, "2", font, 14, Color::rgba(1.0, 1.0, 1.0, 1.0)),
            ]),
            Row::new(&a, 10.0, &xs.iter().map(|x|
                Text::new(&a, a.alloc_str(&x.to_string()), font, 14, Color::rgba(1.0, 1.0, 1.0, 1.0)) as &dyn Widget
            ).collect::<Vec<&dyn Widget>>()),
            Button::new(&a, Text::new(&a, "button", font, 14, Color::rgba(1.0, 1.0, 1.0, 1.0))),
        ]));
        ui.run(size.width as f32, size.height as f32, tree);

        gl_window.swap_buffers().unwrap();

        events_loop.poll_events(|ev| {
            match ev {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(logical_size) => {
                        gl_window.resize(logical_size.to_physical(dpi_factor));
                    }
                    glutin::WindowEvent::ReceivedCharacter(char) => {
                        ui.input(Input::Char(char));
                    }
                    glutin::WindowEvent::KeyboardInput { input: glutin::KeyboardInput { state, virtual_keycode, modifiers, .. }, .. } => {
                        ui.modifiers(glutin_modifiers(modifiers));
                        if let Some(key) = virtual_keycode.and_then(glutin_key) {
                            match state {
                                glutin::ElementState::Pressed => ui.input(Input::KeyDown(key)),
                                glutin::ElementState::Released => ui.input(Input::KeyUp(key)),
                            }
                        }
                    }
                    glutin::WindowEvent::CursorMoved { position, .. } => {
                        ui.cursor(position.x as f32, position.y as f32);
                    }
                    glutin::WindowEvent::MouseWheel { delta, modifiers, .. } => {
                        ui.modifiers(glutin_modifiers(modifiers));
                        let (x, y) = match delta {
                            glutin::MouseScrollDelta::LineDelta(x, y) => (x * 48.0, y * 48.0),
                            glutin::MouseScrollDelta::PixelDelta(glutin::dpi::LogicalPosition { x, y }) => (x as f32, y as f32),
                        };
                        ui.input(Input::Scroll(x, y));
                    }
                    glutin::WindowEvent::MouseInput { state, button, modifiers, .. } => {
                        ui.modifiers(glutin_modifiers(modifiers));
                        if let Some(button) = match button {
                            glutin::MouseButton::Left => Some(MouseButton:: Left),
                            glutin::MouseButton::Middle => Some(MouseButton::Middle),
                            glutin::MouseButton::Right => Some(MouseButton:: Right),
                            _ => None,
                        } {
                            match state {
                                glutin::ElementState::Pressed => ui.input(Input::MouseDown(button)),
                                glutin::ElementState::Released => ui.input(Input::MouseUp(button)),
                            }
                        }
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

fn glutin_modifiers(modifiers: glutin::ModifiersState) -> Modifiers {
    Modifiers {
        shift: modifiers.shift,
        ctrl: modifiers.ctrl,
        alt: modifiers.alt,
        meta: modifiers.logo,
    }
}

fn glutin_key(key: glutin::VirtualKeyCode) -> Option<Key> {
    match key {
        glutin::VirtualKeyCode::Key1 => Some(Key::Key1),
        glutin::VirtualKeyCode::Key2 => Some(Key::Key2),
        glutin::VirtualKeyCode::Key3 => Some(Key::Key3),
        glutin::VirtualKeyCode::Key4 => Some(Key::Key4),
        glutin::VirtualKeyCode::Key5 => Some(Key::Key5),
        glutin::VirtualKeyCode::Key6 => Some(Key::Key6),
        glutin::VirtualKeyCode::Key7 => Some(Key::Key7),
        glutin::VirtualKeyCode::Key8 => Some(Key::Key8),
        glutin::VirtualKeyCode::Key9 => Some(Key::Key9),
        glutin::VirtualKeyCode::Key0 => Some(Key::Key0),
        glutin::VirtualKeyCode::A => Some(Key::A),
        glutin::VirtualKeyCode::B => Some(Key::B),
        glutin::VirtualKeyCode::C => Some(Key::C),
        glutin::VirtualKeyCode::D => Some(Key::D),
        glutin::VirtualKeyCode::E => Some(Key::E),
        glutin::VirtualKeyCode::F => Some(Key::F),
        glutin::VirtualKeyCode::G => Some(Key::G),
        glutin::VirtualKeyCode::H => Some(Key::H),
        glutin::VirtualKeyCode::I => Some(Key::I),
        glutin::VirtualKeyCode::J => Some(Key::J),
        glutin::VirtualKeyCode::K => Some(Key::K),
        glutin::VirtualKeyCode::L => Some(Key::L),
        glutin::VirtualKeyCode::M => Some(Key::M),
        glutin::VirtualKeyCode::N => Some(Key::N),
        glutin::VirtualKeyCode::O => Some(Key::O),
        glutin::VirtualKeyCode::P => Some(Key::P),
        glutin::VirtualKeyCode::Q => Some(Key::Q),
        glutin::VirtualKeyCode::R => Some(Key::R),
        glutin::VirtualKeyCode::S => Some(Key::S),
        glutin::VirtualKeyCode::T => Some(Key::T),
        glutin::VirtualKeyCode::U => Some(Key::U),
        glutin::VirtualKeyCode::V => Some(Key::V),
        glutin::VirtualKeyCode::W => Some(Key::W),
        glutin::VirtualKeyCode::X => Some(Key::X),
        glutin::VirtualKeyCode::Y => Some(Key::Y),
        glutin::VirtualKeyCode::Z => Some(Key::Z),
        glutin::VirtualKeyCode::Escape => Some(Key::Escape),
        glutin::VirtualKeyCode::F1 => Some(Key::F1),
        glutin::VirtualKeyCode::F2 => Some(Key::F2),
        glutin::VirtualKeyCode::F3 => Some(Key::F3),
        glutin::VirtualKeyCode::F4 => Some(Key::F4),
        glutin::VirtualKeyCode::F5 => Some(Key::F5),
        glutin::VirtualKeyCode::F6 => Some(Key::F6),
        glutin::VirtualKeyCode::F7 => Some(Key::F7),
        glutin::VirtualKeyCode::F8 => Some(Key::F8),
        glutin::VirtualKeyCode::F9 => Some(Key::F9),
        glutin::VirtualKeyCode::F10 => Some(Key::F10),
        glutin::VirtualKeyCode::F11 => Some(Key::F11),
        glutin::VirtualKeyCode::F12 => Some(Key::F12),
        glutin::VirtualKeyCode::F13 => Some(Key::F13),
        glutin::VirtualKeyCode::F14 => Some(Key::F14),
        glutin::VirtualKeyCode::F15 => Some(Key::F15),
        glutin::VirtualKeyCode::Snapshot => Some(Key::PrintScreen),
        glutin::VirtualKeyCode::Scroll => Some(Key::ScrollLock),
        glutin::VirtualKeyCode::Pause => Some(Key::Pause),
        glutin::VirtualKeyCode::Insert => Some(Key::Insert),
        glutin::VirtualKeyCode::Home => Some(Key::Home),
        glutin::VirtualKeyCode::Delete => Some(Key::Delete),
        glutin::VirtualKeyCode::End => Some(Key::End),
        glutin::VirtualKeyCode::PageDown => Some(Key::PageDown),
        glutin::VirtualKeyCode::PageUp => Some(Key::PageUp),
        glutin::VirtualKeyCode::Left => Some(Key::Left),
        glutin::VirtualKeyCode::Up => Some(Key::Up),
        glutin::VirtualKeyCode::Right => Some(Key::Right),
        glutin::VirtualKeyCode::Down => Some(Key::Down),
        glutin::VirtualKeyCode::Back => Some(Key::Backspace),
        glutin::VirtualKeyCode::Return => Some(Key::Enter),
        glutin::VirtualKeyCode::Space => Some(Key::Space),
        glutin::VirtualKeyCode::Numlock => Some(Key::NumLock),
        glutin::VirtualKeyCode::Numpad0 => Some(Key::Numpad0),
        glutin::VirtualKeyCode::Numpad1 => Some(Key::Numpad1),
        glutin::VirtualKeyCode::Numpad2 => Some(Key::Numpad2),
        glutin::VirtualKeyCode::Numpad3 => Some(Key::Numpad3),
        glutin::VirtualKeyCode::Numpad4 => Some(Key::Numpad4),
        glutin::VirtualKeyCode::Numpad5 => Some(Key::Numpad5),
        glutin::VirtualKeyCode::Numpad6 => Some(Key::Numpad6),
        glutin::VirtualKeyCode::Numpad7 => Some(Key::Numpad7),
        glutin::VirtualKeyCode::Numpad8 => Some(Key::Numpad8),
        glutin::VirtualKeyCode::Numpad9 => Some(Key::Numpad9),
        glutin::VirtualKeyCode::Add => Some(Key::NumpadAdd),
        glutin::VirtualKeyCode::Apostrophe => Some(Key::Apostrophe),
        glutin::VirtualKeyCode::Backslash => Some(Key::Backslash),
        glutin::VirtualKeyCode::Capital => Some(Key::CapsLock),
        glutin::VirtualKeyCode::Comma => Some(Key::Comma),
        glutin::VirtualKeyCode::Decimal => Some(Key::NumpadDecimal),
        glutin::VirtualKeyCode::Divide => Some(Key::NumpadDivide),
        glutin::VirtualKeyCode::Equals => Some(Key::Equals),
        glutin::VirtualKeyCode::Grave => Some(Key::GraveAccent),
        glutin::VirtualKeyCode::LAlt => Some(Key::LeftAlt),
        glutin::VirtualKeyCode::LBracket => Some(Key::LeftBracket),
        glutin::VirtualKeyCode::LControl => Some(Key::LeftControl),
        glutin::VirtualKeyCode::LShift => Some(Key::LeftShift),
        glutin::VirtualKeyCode::LWin => Some(Key::LeftMeta),
        glutin::VirtualKeyCode::Minus => Some(Key::Minus),
        glutin::VirtualKeyCode::Multiply => Some(Key::NumpadMultiply),
        glutin::VirtualKeyCode::NumpadEnter => Some(Key::NumpadEnter),
        glutin::VirtualKeyCode::NumpadEquals => Some(Key::NumpadEquals),
        glutin::VirtualKeyCode::Period => Some(Key::Period),
        glutin::VirtualKeyCode::RAlt => Some(Key::RightAlt),
        glutin::VirtualKeyCode::RBracket => Some(Key::RightBracket),
        glutin::VirtualKeyCode::RControl => Some(Key::RightControl),
        glutin::VirtualKeyCode::RShift => Some(Key::RightShift),
        glutin::VirtualKeyCode::RWin => Some(Key::RightMeta),
        glutin::VirtualKeyCode::Semicolon => Some(Key::Semicolon),
        glutin::VirtualKeyCode::Slash => Some(Key::Slash),
        glutin::VirtualKeyCode::Subtract => Some(Key::NumpadSubtract),
        glutin::VirtualKeyCode::Tab => Some(Key::Tab),
        _ => None,
    }
}
