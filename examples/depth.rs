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
use winit::keyboard::{Key, NamedKey};
use winit::{
    event::{ElementState, WindowEvent},
    event_loop::{self, ControlFlow},
    window::WindowBuilder,
};

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = event_loop::EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("wgpu-text: 'depth' example")
        .build(&event_loop)
        .unwrap();
    let window = Arc::new(window);

    let (device, queue, surface, mut config) = WgpuUtils::init(window.clone());

    let mut depth_view = create_depth_view(&device, config.width, config.height);

    let depth_stencil = Some(wgpu::DepthStencilState {
        format: DEPTH_FORMAT,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::LessEqual,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    });

    // All wgpu-text related below:
    let font: &[u8] = include_bytes!("fonts/DejaVuSans.ttf");
    let mut brush = BrushBuilder::using_font_bytes(font)
        .unwrap()
        .with_depth_stencil(depth_stencil)
        .build(&device, config.width, config.height, config.format);

    let mut font_size = 45.;
    let mut section = Section::default()
        .add_text(
            Text::new(
                "Try typing some text,\n \
                del - delete all, backspace - remove last character",
            )
            .with_scale(font_size)
            .with_color([0.9, 0.5, 0.5, 1.0])
            .with_z(0.08), // In range 0.0 - 1.0 bigger number means it's more at the back
        )
        .with_bounds((config.width as f32 / 2.0, config.height as f32))
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
                .with_scale(80.0)
                .with_color([0.2, 0.5, 0.8, 1.0])
                .with_z(0.1), // In range 0.0 - 1.0 bigger number means it's more at the back
        )
        .with_bounds((config.width as f32 / 2.0, config.height as f32))
        .with_layout(
            Layout::default()
                .v_align(VerticalAlign::Top)
                .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
        )
        .with_screen_position((50.0, config.height as f32 * 0.5))
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

                    let width = config.width as f32;
                    let height = config.height as f32;

                    section.bounds = (width * 0.5, height);
                    section.screen_position.1 = height * 0.5;

                    depth_view = create_depth_view(&device, config.width, config.height);
                    brush.resize_view(width, height, &queue);
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
                                        .with_color([0.9, 0.5, 0.5, 1.0])
                                        .with_z(0.08),
                                );
                            }
                            section.text.push(
                                OwnedText::new(c.to_string())
                                    .with_scale(font_size)
                                    .with_color([0.9, 0.5, 0.5, 1.0])
                                    .with_z(0.08),
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
                    match brush.queue(&device, &queue, vec![&section2, &section]) {
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
                                depth_stencil_attachment: Some(
                                    wgpu::RenderPassDepthStencilAttachment {
                                        view: &depth_view,
                                        depth_ops: Some(wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(1.0),
                                            store: wgpu::StoreOp::Discard,
                                        }),
                                        stencil_ops: None,
                                    },
                                ),
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
                            .set_title(&format!("wgpu-text: 'depth' example, FPS: {}", fps));
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

fn create_depth_view(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> wgpu::TextureView {
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("Depth Texture"),
        view_formats: &[],
    });

    depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}
