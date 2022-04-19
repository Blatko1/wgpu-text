mod simple;

use nalgebra::{Matrix4, Rotation3};
use simple::WgpuUtils;
use std::time::{Duration, Instant, SystemTime};
use wgpu_text::section::{
    BuiltInLineBreaker, Layout, OwnedText, Section, Text, VerticalAlign,
};
use wgpu_text::BrushBuilder;
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{self, ControlFlow},
    window::WindowBuilder,
};

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[allow(unused)]
fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = event_loop::EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("wgpu-text: 'simple' example")
        .build(&event_loop)
        .unwrap();

    let (device, queue, surface, mut config) = WgpuUtils::init(&window);

    // All wgpu-text related below:
    let font: &[u8] = include_bytes!("fonts/Inconsolata-Regular.ttf");
    let mut brush = BrushBuilder::using_font_bytes(font)
        .unwrap()
        .build(&device, &config);

    let mut font_size = 25.;
    let mut section = Section::default()
        .add_text(
            Text::new("Heheh Test")
                .with_scale(font_size)
                .with_color([0.9, 0.5, 0.5, 1.0]),
        )
        .with_bounds((500.0, 600.0))
        .with_layout(
            Layout::default()
                .v_align(VerticalAlign::Center)
                .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
        )
        .with_screen_position((50.0, 100.0))
        .to_owned();

    // FPS and window updating:
    let mut then = SystemTime::now();
    let mut now = SystemTime::now();
    let mut fps = 0;
    let target_framerate = Duration::from_secs_f64(1.0 / 60.0); //<-- change '60.0' if you want other FPS cap
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

                    let roll: f32 = 60.0;
                    let rotation =
                        Rotation3::from_euler_angles(roll.to_radians(), 0.0, 0.0);
                    let ortho = Matrix4::from(wgpu_text::ortho(
                        config.width as f32,
                        config.height as f32,
                    ));
                    let fov: f32 = 60.0;
                    let projection = Matrix4::new_perspective(
                        config.width as f32 / config.height as f32,
                        fov.to_degrees(),
                        0.1,
                        100.0,
                    );

                    let matrix = OPENGL_TO_WGPU_MATRIX * projection;
                    //dbg!(&matrix);
                    brush.update_matrix(matrix, &queue);
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
                    VirtualKeyCode::Delete => section.text.clear(),
                    VirtualKeyCode::Back if !section.text.is_empty() => {
                        let mut end_text = section.text.remove(section.text.len() - 1);
                        end_text.text.pop();
                        if !end_text.text.is_empty() {
                            section.text.push(end_text);
                        }
                    }
                    _ => (),
                },
                WindowEvent::ReceivedCharacter(c) => {
                    if c != '\u{7f}' && c != '\u{8}' {
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
                        color_attachments: &[wgpu::RenderPassColorAttachment {
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
                        }],
                        depth_stencil_attachment: None,
                    });
                }
                brush.queue(&section);

                let cmd_buffer = brush.draw(&device, &view, &queue);

                // Has to be submitted last so it won't be overlapped.
                queue.submit([encoder.finish(), cmd_buffer]);
                frame.present();

                fps += 1;
                if now.duration_since(then).unwrap().as_millis() > 1000 {
                    window
                        .set_title(&format!("wgpu-text: 'simple' example, FPS: {}", fps));
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
                        Instant::now() + target_framerate - delta_time.elapsed(),
                    );
                }
            }
            _ => (),
        }
    });
}
