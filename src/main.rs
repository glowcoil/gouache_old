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

    let dpi_factor = display.gl_window().get_hidpi_factor();
    let mut renderer = Renderer::new(&display, dpi_factor as f32);

    let font = rusttype::FontCollection::from_bytes(include_bytes!("../sawarabi-gothic-medium.ttf") as &[u8]).unwrap().into_font().unwrap();

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
        let mut glyphs = Vec::with_capacity(fps_text.len());
        for (i,c) in fps_text.chars().enumerate() {
            glyphs.push(font.glyph(c).scaled(rusttype::Scale::uniform(18.0)).positioned(rusttype::point(10.0 * i as f32, 20.0)));
        }
        renderer.render(&display, &[Cmd::DrawGlyphs { glyphs }]);

        events_loop.poll_events(|ev| {
            match ev {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(logical_size) => {
                        display.gl_window().resize(logical_size.to_physical(dpi_factor));
                        renderer.render(&display, &[Cmd::DrawGlyphs { glyphs: Vec::new() }]);
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
