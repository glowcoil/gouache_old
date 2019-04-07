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
        Padding::uniform(10.0, Text::new(&fps_text, font, 14, Color::rgba(1.0, 1.0, 1.0, 1.0)))
            .draw(&mut ui, Rect { left: 0.0, top: 0.0, right: size.width as f32, bottom: size.height as f32 });
        ui.graphics().draw(size.width as f32, size.height as f32);

        gl_window.swap_buffers().unwrap();

        events_loop.poll_events(|ev| {
            match ev {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(logical_size) => {
                        gl_window.resize(logical_size.to_physical(dpi_factor));
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
