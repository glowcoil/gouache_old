mod render;

use render::*;

#[macro_use]
extern crate glium;
extern crate rusttype;

use glium::glutin;

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0))
        .with_title("justitracker");
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let mut renderer = Renderer::new(&display, display.gl_window().get_hidpi_factor() as f32);

    let font = rusttype::FontCollection::from_bytes(include_bytes!("../sawarabi-gothic-medium.ttf") as &[u8]).unwrap().into_font().unwrap();

    events_loop.run_forever(|ev| {
        match ev {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => return glutin::ControlFlow::Break,
                _ => (),
            },
            _ => (),
        }

        renderer.render(&display, &[Cmd::DrawGlyphs { glyphs: vec![font.glyph('a').scaled(rusttype::Scale::uniform(18.0)).positioned(rusttype::point(0.0, 20.0))] }]);

        glutin::ControlFlow::Continue
    });
}
