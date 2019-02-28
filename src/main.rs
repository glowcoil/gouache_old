mod ui;
mod arena;

use ui::*;

extern crate gl;
extern crate glutin;
extern crate nanovg;

use glutin::GlContext;
use nanovg::{Color, Font, Alignment, Gradient, TextOptions, Scissor, Frame, Transform, PathOptions, Clip};

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0))
        .with_title("justitracker");
    let context = glutin::ContextBuilder::new()
        .with_multisampling(4)
        .with_srgb(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
    }

    let context = nanovg::ContextBuilder::new()
        .stencil_strokes()
        .build()
        .unwrap();

    let font = Font::from_memory(&context, "Sawarabi Gothic", include_bytes!("../sawarabi-gothic-medium.ttf")).unwrap();

    let mut mouse = (0.0f32, 0.0f32);

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

        let (width, height): (u32, u32) = gl_window.get_inner_size().unwrap().into();
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
            gl::ClearColor(0.3, 0.3, 0.32, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }

        context.frame((width as f32, height as f32), gl_window.get_hidpi_factor() as f32, |frame| {
            let margin = 50.0;
            let clip = (margin, margin, width as f32 - margin * 2.0, height as f32 - margin * 2.0);
            let transform = Transform::new().with_translation(mouse.0, mouse.1);//.rotate(elapsed * 4.0);
            // render_text(&frame, font, "text", clip, transform);
            let transform = Transform::new().with_translation(150.0, 100.0);//.rotate(-PI / 6.0);
            // draw_paragraph(&frame, font, -150.0 / 2.0, -50.0, 150.0, 100.0, mouse, transform);

            frame.text(font, (100.0, 100.0), fps_text,
                TextOptions {
                    size: 20.0,
                    color: Color::from_rgb(255, 255, 255),
                    align: Alignment::new().bottom().right(),
                    transform: Some(transform),
                    ..Default::default()
                }
            );



            draw_button(&frame, font, "button", 0.0, 0.0, 80.0, 28.0, Color::from_rgba(128, 16, 8, 255));



        });

        gl_window.swap_buffers().unwrap();

        events_loop.poll_events(|ev| {
            match ev {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(logical_size) => {
                    }
                    glutin::WindowEvent::CursorMoved { position, .. } => mouse = (position.x as f32, position.y as f32),
                    _ => (),
                },
                _ => (),
            }
        });

        let elapsed = now.elapsed();
        if elapsed < FRAME {
            // std::thread::sleep(FRAME - elapsed);
        }
    }
}

fn draw_button(frame: &Frame, font: Font, text: &str, x: f32, y: f32, w: f32, h: f32, color: Color) {
    let corner_radius = 4.0;

    // button background
    frame.path(
        |path| {
            path.rounded_rect((x + 1.0, y + 1.0), (w - 2.0, h - 2.0), corner_radius - 0.5);
            path.fill(color, Default::default());

            path.fill(
                Gradient::Linear {
                    start: (x, y),
                    end: (x, y + h),
                    start_color: Color::from_rgba(255, 255, 255, 32),
                    end_color: Color::from_rgba(0, 0, 0, 32),
                },
                Default::default()
            );
        },
        Default::default(),
    );

    // button border
    frame.path(
        |path| {
            path.rounded_rect((x + 0.5, y + 0.5), (w - 1.0, h - 1.0), corner_radius - 0.5);
            path.stroke(Color::from_rgba(0, 0, 0, 48), Default::default());
        },
        Default::default(),
    );

    let (tw, _) = frame.text_bounds(
        font,
        (0.0, 0.0),
        text,
        TextOptions {
            size: 20.0,
            ..Default::default()
        },
    );

    let mut iw = 0.0;

    let mut options = TextOptions {
        size: 20.0,
        align: Alignment::new().left().middle(),
        ..Default::default()
    };

    options.color = Color::from_rgba(0, 0, 0, 160);

    frame.text(
        font,
        (x + w * 0.5 - tw * 0.5 + iw * 0.25, y + h * 0.5 - 1.0),
        text,
        options,
    );

    options.color = Color::from_rgba(255, 255, 255, 160);

    frame.text(
        font,
        (x + w * 0.5 - tw * 0.5 + iw * 0.25, y + h * 0.5),
        text,
        options,
    );
}
