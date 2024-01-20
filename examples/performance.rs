#[path = "utils.rs"]
mod utils;

use rand::Rng;
use winit::event::{MouseScrollDelta, Event, KeyEvent};
use winit::keyboard::{Key, NamedKey};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use utils::WgpuUtils;
use wgpu_text::glyph_brush::{BuiltInLineBreaker, Layout, Section, Text};
use wgpu_text::BrushBuilder;
use winit::{
    event::{ElementState, WindowEvent},
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

    let event_loop = event_loop::EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("wgpu-text: 'performance' example")
        .build(&event_loop)
        .unwrap();
    let window = Arc::new(window);

    let (device, queue, surface, mut config) = WgpuUtils::init(window.clone());

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
    event_loop.run(move |event, elwt| {
        match event {
            Event::LoopExiting => {
                println!("Exiting!");
            }
            Event::NewEvents(_) => {
                if target_framerate <= delta_time.elapsed() {
                    window.request_redraw();
                    delta_time = Instant::now();
                } else {
                    elwt.set_control_flow(ControlFlow::WaitUntil(
                        Instant::now().checked_sub(delta_time.elapsed()).unwrap()
                            + target_framerate,
                    ));
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size) => {
                    config.width = new_size.width.max(1);
                    config.height = new_size.height.max(1);
                    surface.configure(&device, &config);

                    brush.resize_view(config.width as f32, config.height as f32, &queue);
                    // You can also do this!
                    // brush.update_matrix(wgpu_text::ortho(config.width, config.height), &queue);
                },
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::KeyboardInput { event: KeyEvent {
                    logical_key,
                    state: ElementState::Pressed,
                    ..
                }, .. } => match logical_key {
                    Key::Named(k) => 
                        match k {
                            NamedKey::Escape => elwt.exit(),
                            NamedKey::Delete => random_text.clear(),
                            NamedKey::Backspace => {random_text.pop();},
                            _ => ()
                        },
                    Key::Character(char) => {
                        random_text.push_str(char.as_str());
                    },
                    _ => ()
                },
                WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, y), ..} => {
                    // increase/decrease font size
                    let mut size = font_size;
                    if y > 0.0 {
                        size += (size / 4.0).max(2.0)
                    } else {
                        size *= 4.0 / 5.0
                    };
                    font_size = (size.max(3.0).min(25000.0) * 2.0).round() / 2.0;
                }
                WindowEvent::RedrawRequested => {
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

                match brush.queue(&device, &queue, vec![&section]) {
                    Ok(_) => (),
                    Err(err) => {
                        panic!("{err}");
                    }
                };

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
                    let mut rpass =
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
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                    brush.draw(&mut rpass)
                }

                queue.submit([encoder.finish()]);
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
                },
                _ => ()
            }
            _ => (),
        }
    }).unwrap();
}
