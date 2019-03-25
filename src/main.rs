mod ui;
mod graphics;
mod render;
mod alloc;

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
    let context = glutin::ContextBuilder::new()
        .with_srgb(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe { gl_window.make_current().unwrap(); }
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    let size = gl_window.get_inner_size().unwrap();
    let dpi_factor = gl_window.get_hidpi_factor();

    let mut graphics = Graphics::new(size.width as f32, size.height as f32, dpi_factor as f32);
    let font = graphics.add_font(include_bytes!("../sawarabi-gothic-medium.ttf"));

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

        unsafe {
            gl::ClearColor(0.1, 0.15, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        {
            let frame = Frame::new();
            graphics.draw(frame.stack(&[
                frame.glyphs(&graphics.text([0.0, 10.0], "Jackdaws love my big sphinx of quartz.", font, 14)),
                frame.glyphs(&graphics.text([700.0, 580.0], &fps_text, font, 14)),
            ]));
        }

        gl_window.swap_buffers().unwrap();

        events_loop.poll_events(|ev| {
            match ev {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(logical_size) => {
                        gl_window.resize(logical_size.to_physical(dpi_factor));
                        graphics.set_size(logical_size.width as f32, logical_size.height as f32);
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
