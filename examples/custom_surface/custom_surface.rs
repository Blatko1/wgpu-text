mod camera;
mod pipeline;
#[path = "../utils.rs"]
mod utils;

use camera::Camera;
use glyph_brush::{OwnedText, VerticalAlign};
use pipeline::{create_pipeline, Vertex};
use std::time::{Duration, Instant, SystemTime};
use utils::WgpuUtils;
use wgpu::util::DeviceExt;
use wgpu_text::section::{BuiltInLineBreaker, Layout, Section, Text};
use wgpu_text::BrushBuilder;
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{self, ControlFlow},
    window::WindowBuilder,
};
// TODO test with custom .png texture
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5, 0.0],
        tex_pos: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        tex_pos: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
        tex_pos: [1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
        tex_pos: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.0],
        tex_pos: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        tex_pos: [0.0, 1.0],
    },
];

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = event_loop::EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("wgpu-text: 'custom_surface' example")
        .build(&event_loop)
        .unwrap();

    let (device, queue, surface, mut config) = WgpuUtils::init(&window);
    let mut camera = Camera::new(&config);
    let size = wgpu::Extent3d {
        width: 256 * 4,
        height: 256 * 4,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Custom Surface Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let texture_view = texture.create_view(&Default::default());

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Custom Surface Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Custom Surface Global Matrix Buffer"),
        contents: bytemuck::cast_slice(camera.global_matrix.as_slice()),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Custom Surface Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Custom Surface Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: matrix_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let pipeline = create_pipeline(&device, &[&bind_group_layout], &config);

    let font: &[u8] = include_bytes!("../fonts/DejaVuSans.ttf");
    let mut brush = BrushBuilder::using_font_bytes(font).unwrap().build(
        &device,
        size.width,
        size.height,
        texture.format(),
    );
    brush.set_load_op(wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT));

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
        .with_bounds((config.width as f32 / 2.0, config.height as f32))
        .with_layout(
            Layout::default()
                .v_align(VerticalAlign::Center)
                .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
        )
        .with_screen_position((50.0, config.height as f32 * 0.5))
        .to_owned();

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
                    camera.resize(&config);

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
                camera.update();
                queue.write_buffer(
                    &matrix_buffer,
                    0,
                    bytemuck::cast_slice(camera.global_matrix.as_slice()),
                );
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
                        label: Some("Custom Surface Command Encoder"),
                    });

                {
                    let mut rpass =
                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Custom Surface Render Pass"),
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

                    rpass.set_pipeline(&pipeline);
                    rpass.set_bind_group(0, &bind_group, &[]);
                    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    rpass.draw(0..6, 0..1);
                }

                brush.queue(&section);
                match brush.process_queued(&device, &queue) {
                    Ok(_) => (),
                    Err(err) => {
                        panic!("{err}");
                    }
                };

                let cmd_buffer = brush.draw(&device, &texture_view);

                queue.submit([cmd_buffer, encoder.finish()]);
                frame.present();

                fps += 1;
                if now.duration_since(then).unwrap().as_millis() > 1000 {
                    window.set_title(&format!(
                        "wgpu-text: 'custom_surface' example, FPS: {}",
                        fps
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
