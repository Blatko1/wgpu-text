#[path = "utils.rs"]
mod utils;

use rand::Rng;
use std::time::{Duration, Instant, SystemTime};
use utils::WgpuUtils;
use wgpu_text::section::{BuiltInLineBreaker, Layout, Section, Text};
use wgpu_text::BrushBuilder;
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{self, ControlFlow},
    window::WindowBuilder,
};

const RANDOM_CHARACTERS: usize = 30_000;

fn generate_random_chars() -> String {
    let mut result = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..RANDOM_CHARACTERS {
        let rand = rng.gen_range(0x0041..0x0070);
        let char = char::from_u32(rand).unwrap();
        result.push(char);
    }
    result.trim().to_owned()
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    if cfg!(debug_assertions) {
        eprintln!(
            "You should probably run an example called 'performance' in release mode.\n\
            e.g. use `cargo run --example performance --release`\n"
        );
    }

    let event_loop = event_loop::EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("wgpu-text: 'performance' example")
        .build(&event_loop)
        .unwrap();

    let (device, queue, surface, mut config) = WgpuUtils::init(&window);

    let font: &[u8] = include_bytes!("fonts/DejaVuSans.ttf");
    let mut brush = BrushBuilder::using_font_bytes(font).unwrap().build(
        &device,
        config.width,
        config.height,
        config.format,
    );

    let mut random_text = generate_random_chars();
    let mut font_size: f32 = 9.;

    let mut then = SystemTime::now();
    let mut now = SystemTime::now();
    let mut fps = 0;
    // change '60.0' if you want different FPS cap
    let target_framerate = Duration::from_secs_f64(1.0 / 60.0);
    let mut delta_time = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size)
                | WindowEvent::ScaleFactorChanged {
                    new_inner_size: &mut new_size,
                    ..
                } => {
                    config.width = new_size.width.max(1);
                    config.height = new_size.height.max(1);
                    surface.configure(&device, &config);

                    brush.resize_view(config.width as f32, config.height as f32, &queue);
                    // You can also do this!
                    // brush.update_matrix(wgpu_text::ortho(config.width, config.height), &queue);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(keypress),
                            ..
                        },
                    ..
                } => match keypress {
                    VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                    VirtualKeyCode::Delete => random_text.clear(),
                    VirtualKeyCode::Back => {
                        random_text.pop();
                    }
                    _ => (),
                },
                WindowEvent::ReceivedCharacter(c) => {
                    random_text.push(c);
                }
                WindowEvent::MouseWheel {
                    delta: winit::event::MouseScrollDelta::LineDelta(_, y),
                    ..
                } => {
                    // increase/decrease font size
                    let mut size = font_size;
                    if y > 0.0 {
                        size += (size / 4.0).max(2.0)
                    } else {
                        size *= 4.0 / 5.0
                    };
                    font_size = (size.max(3.0).min(2000.0) * 2.0).round() / 2.0;
                }
                _ => (),
            },
            winit::event::Event::RedrawRequested(_) => {
                let frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(_) => {
                        surface.configure(&device, &config);
                        surface
                            .get_current_texture()
                            .expect("Failed to acquire next surface texture!")
                    }
                };
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Command Encoder"),
                    });

                {
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.2,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.,
                                }),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                }

                let section = Section::default()
                    .add_text(
                        Text::new(&random_text)
                            .with_scale(font_size)
                            .with_color([0.9, 0.5, 0.5, 1.0]),
                    )
                    .with_bounds((config.width as f32, config.height as f32))
                    .with_layout(
                        Layout::default()
                            .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
                    );

                brush.queue(&section);
                match brush.process_queued(&device, &queue) {
                    Ok(_) => (),
                    Err(err) => {
                        panic!("{err}");
                    }
                };

                let cmd_buffer = brush.draw(&device, &view);
                // Has to be submitted last so it won't be overlapped.
                queue.submit([encoder.finish(), cmd_buffer]);
                frame.present();

                fps += 1;
                if now.duration_since(then).unwrap().as_millis() > 1000 {
                    window.set_title(&format!(
                        "wgpu-text: 'performance' example, FPS: {}, glyphs: {}",
                        fps,
                        random_text.len()
                    ));
                    fps = 0;
                    then = now;
                }
                now = SystemTime::now();
            }
            winit::event::Event::MainEventsCleared => {
                if target_framerate <= delta_time.elapsed() {
                    window.request_redraw();
                    delta_time = Instant::now();
                } else {
                    *control_flow = ControlFlow::WaitUntil(
                        Instant::now().checked_sub(delta_time.elapsed()).unwrap()
                            + target_framerate,
                    );
                }
            }
            _ => (),
        }
    });
}
