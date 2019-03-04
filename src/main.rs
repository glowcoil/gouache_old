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
        .with_srgb(true)
        .with_multisampling(4);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe { gl_window.make_current().unwrap(); }
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    let dpi_factor = gl_window.get_hidpi_factor();
    let mut renderer = Renderer::new();
    // let mut ui = UI::new();

    // let font = rusttype::FontCollection::from_bytes(include_bytes!("../sawarabi-gothic-medium.ttf") as &[u8]).unwrap().into_font().unwrap();

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
            renderer.draw(&[
                render::Vertex { pos: [-0.5, -0.5, 0.0], col: [0.0, 1.0, 1.0, 1.0] },
                render::Vertex { pos: [ 0.5, -0.5, 0.0], col: [1.0, 1.0, 1.0, 1.0] },
                render::Vertex { pos: [ 0.5,  0.5, 0.0], col: [1.0, 1.0, 1.0, 1.0] },
                render::Vertex { pos: [-0.5,  0.5, 0.0], col: [1.0, 1.0, 1.0, 1.0] },
            ], &[
                0, 1, 2, 2, 3, 0
            ]);
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
