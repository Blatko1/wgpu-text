#[path = "utils.rs"]
mod utils;

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use utils::WgpuUtils;
use wgpu_text::glyph_brush::{
    BuiltInLineBreaker, Layout, OwnedText, Section, Text, VerticalAlign,
};
use wgpu_text::BrushBuilder;
use winit::event::{Event, KeyEvent, MouseScrollDelta};
use winit::event_loop::{self, ControlFlow};
use winit::keyboard::{NamedKey, Key};
use winit::{
    event::{ElementState, WindowEvent},
    window::WindowBuilder,
};

// TODO text layout of characters like 'š, ć, ž, đ' doesn't work correctly.
fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = event_loop::EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("wgpu-text: 'simple' example")
        .build(&event_loop)
        .unwrap();
    let window = Arc::new(window);

    let (device, queue, surface, mut config) = WgpuUtils::init(window.clone());

    // All wgpu-text related below:
    let font: &[u8] = include_bytes!("fonts/DejaVuSans.ttf");
    let mut brush = BrushBuilder::using_font_bytes(font).unwrap().build(
        &device,
        config.width,
        config.height,
        config.format,
    );

    let mut font_size = 25.;
    let mut section = Section::default()
        .add_text(
            Text::new(
                "Try typing some text,\n \
                del - delete all, backspace - remove last character",
            )
            .with_scale(font_size)
            .with_color([0.9, 0.5, 0.5, 1.0]),
        )
        .with_bounds((config.width as f32 * 0.4, config.height as f32))
        .with_layout(
            Layout::default()
                .v_align(VerticalAlign::Center)
                .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
        )
        .with_screen_position((50.0, config.height as f32 * 0.5))
        .to_owned();
    let section2 = Section::default()
        .add_text(
            Text::new("Other section")
                .with_scale(40.0)
                .with_color([0.2, 0.5, 0.8, 1.0]),
        )
        .with_bounds((config.width as f32 * 0.5, config.height as f32))
        .with_layout(
            Layout::default()
                .v_align(VerticalAlign::Top)
                .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
        )
        .with_screen_position((500.0, config.height as f32 * 0.2))
        .to_owned();

    // FPS and window updating:
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

                    section.bounds = (config.width as f32 * 0.4, config.height as _);
                    section.screen_position.1 = config.height as f32 * 0.5;

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
                            NamedKey::Delete => section.text.clear(),
                            NamedKey::Backspace if !section.text.is_empty() => {
                                let mut end_text = section.text.remove(section.text.len() - 1);
                                end_text.text.pop();
                                if !end_text.text.is_empty() {
                                    section.text.push(end_text);
                                }
                            }
                            _ => ()
                        },
                    Key::Character(char) => {
                        let c = char.as_str();
                        if c != "\u{7f}" && c != "\u{8}" {
                            if section.text.is_empty() {
                                section.text.push(
                                    OwnedText::default()
                                        .with_scale(font_size)
                                        .with_color([0.9, 0.5, 0.5, 1.0]),
                                );
                            }
                            section.text.push(
                                OwnedText::new(c.to_string())
                                    .with_scale(font_size)
                                    .with_color([0.9, 0.5, 0.5, 1.0]),
                            );
                        }
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
                    match brush.queue(&device, &queue, vec![&section, &section2]) {
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
    
                        brush.draw(&mut rpass);
                    }
    
                    queue.submit([encoder.finish()]);
                    frame.present();
    
                    fps += 1;
                    if now.duration_since(then).unwrap().as_millis() > 1000 {
                        window
                            .set_title(&format!("wgpu-text: 'simple' example, FPS: {}", fps));
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
