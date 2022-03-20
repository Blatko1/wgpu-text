use std::time::{Duration, Instant, SystemTime};

use pollster::block_on;
use wgpu::{Features, Limits};
use wgpu_text::section::{BuiltInLineBreaker, Layout, OwnedText, Section, Text, VerticalAlign};
use wgpu_text::BrushBuilder;
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{self, ControlFlow},
    window::WindowBuilder,
};

fn main() {
    std::env::set_var("RUST_LOG", "error");
    env_logger::init();
    log::info!("STARTING");

    let event_loop = event_loop::EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Simple text rendering")
        .build(&event_loop)
        .unwrap();

    let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);
    let instance = wgpu::Instance::new(backends);
    let (size, surface) = unsafe {
        let size = window.inner_size();
        let surface = instance.create_surface(&window);
        (size, surface)
    };
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .expect("No adapters found!");

    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("Device"),
            features: Features::empty(),
            limits: Limits::default(),
        },
        None,
    ))
    .unwrap();
    let format = surface.get_preferred_format(&adapter).unwrap();
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    surface.configure(&device, &config);

    let font: &[u8] = include_bytes!("Inconsolata-Regular.ttf");
    let mut brush = BrushBuilder::using_font_bytes(font).unwrap().build(
        &device,
        format,
        config.width as f32,
        size.height as f32,
    );

    let mut section = Section::default()
        .add_text(
            Text::new("* Type text\n")
                .with_scale(25.0)
                .with_color([0.9, 0.5, 0.5, 1.0]),
        )
        .with_bounds((size.width as f32 / 2.0, size.height as f32))
        .with_layout(
            Layout::default()
                .v_align(VerticalAlign::Center)
                .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
        )
        .with_screen_position((50.0, size.height as f32 * 0.5))
        .to_owned();

    let section2 = Section::default()
        .add_text(
            Text::new("* Test 2")
                .with_scale(40.0)
                .with_color([0.2, 0.5, 0.8, 1.0]),
        )
        .with_bounds((size.width as f32 / 2.0, size.height as f32))
        .with_layout(
            Layout::default()
                .v_align(VerticalAlign::Top)
                .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
        )
        .with_screen_position((300.0, size.height as f32 * 0.5))
        .to_owned();

    let mut then = SystemTime::now();
    let mut now = SystemTime::now();
    let mut fps = 0;
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

                    section.bounds = (config.width as f32 * 0.5, config.height as _);
                    section.screen_position.1 = config.height as f32 * 0.5;

                    brush.resize(config.width as f32, config.height as f32, &queue)
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::ReceivedCharacter(c) => {
                    section.text.push(
                        OwnedText::new(c.to_string())
                            .with_color([0.9, 0.5, 0.5, 1.0])
                            .with_scale(25.0),
                    );
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => (),
            },

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

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                // Has to be queued every frame.
                brush.queue(&section);
                brush.queue(&section2);

                let cmd_buffer = brush.draw_queued(&device, &view, &queue);
                // Has to be submitted last so it won't be overlapped.
                queue.submit([encoder.finish(), cmd_buffer]);
                frame.present();

                fps += 1;
                if now.duration_since(then).unwrap().as_millis() > 1000 {
                    // Remove comment to print your FPS.
                    //println!("FPS: {}", fps);
                    fps = 0;
                    then = now;
                }
                now = SystemTime::now();
            }
            winit::event::Event::RedrawEventsCleared => (),
            _ => (),
        }
    });
}