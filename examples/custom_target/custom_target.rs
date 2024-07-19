mod camera;
#[path = "../ctx.rs"]
mod ctx;
mod pipeline;

use camera::Camera;
use ctx::Ctx;
use glyph_brush::ab_glyph::FontRef;
use glyph_brush::{OwnedSection, OwnedText, VerticalAlign};
use pipeline::{create_pipeline, Vertex};
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, Buffer, RenderPipeline, TextureView};
use wgpu_text::glyph_brush::{BuiltInLineBreaker, Layout, Section, Text};
use wgpu_text::{BrushBuilder, TextBrush};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event::{KeyEvent, MouseScrollDelta, StartCause};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};
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

struct State<'a> {
    // Use an `Option` to allow the window to not be available until the
    // application is properly running.
    window: Option<Arc<Window>>,
    font: &'a [u8],
    brush: Option<TextBrush<FontRef<'a>>>,
    font_size: f32,
    section: Option<OwnedSection>,
    camera: Option<Camera>,
    matrix_buffer: Option<Buffer>,
    texture_view: Option<TextureView>,
    pipeline: Option<RenderPipeline>,
    bind_group: Option<BindGroup>,
    vertex_buffer: Option<Buffer>,

    target_framerate: Duration,
    delta_time: Instant,
    fps_update_time: Instant,
    fps: i32,

    // wgpu
    ctx: Option<Ctx>,
}

impl ApplicationHandler for State<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("wgpu-text: 'simple' example"),
                )
                .unwrap(),
        );

        self.ctx = Some(Ctx::new(window.clone()));

        let ctx = self.ctx.as_ref().unwrap();
        let device = &ctx.device;
        let config = &ctx.config;

        self.brush = Some(BrushBuilder::using_font_bytes(self.font).unwrap().build(
            device,
            config.width,
            config.height,
            config.format,
        ));

        self.camera = Some(Camera::new(config));
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
        self.texture_view = Some(texture.create_view(&Default::default()));

        self.vertex_buffer = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Custom Surface Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));

        self.matrix_buffer = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Custom Surface Global Matrix Buffer"),
                contents: bytemuck::cast_slice(
                    self.camera.as_ref().unwrap().global_matrix.as_slice(),
                ),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        ));

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
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
            });

        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Custom Surface Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.matrix_buffer.as_ref().unwrap().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        self.texture_view.as_ref().unwrap(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        }));
        self.pipeline = Some(create_pipeline(device, &[&bind_group_layout], config));

        self.section = Some(
            Section::default()
                .add_text(
                    Text::new(
                        "Try typing some text,\n \
                    del - delete all, backspace - remove last character",
                    )
                    .with_scale(self.font_size)
                    .with_color([0.9, 0.5, 0.5, 1.0]),
                )
                .with_bounds((config.width as f32 / 2.0, config.height as f32))
                .with_layout(
                    Layout::default()
                        .v_align(VerticalAlign::Center)
                        .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
                )
                .with_screen_position((50.0, config.height as f32 * 0.5))
                .to_owned(),
        );

        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                let ctx = self.ctx.as_mut().unwrap();
                let queue = &ctx.queue;
                let device = &ctx.device;
                let config = &mut ctx.config;
                let surface = &ctx.surface;
                let brush = self.brush.as_mut().unwrap();

                config.width = new_size.width.max(1);
                config.height = new_size.height.max(1);
                surface.configure(device, config);
                self.camera.as_mut().unwrap().resize(config);

                brush.resize_view(config.width as f32, config.height as f32, queue);

                // You can also do this!
                // brush.update_matrix(wgpu_text::ortho(config.width, config.height), &queue);
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match logical_key {
                Key::Named(k) => match k {
                    NamedKey::Escape => event_loop.exit(),
                    NamedKey::Delete => self.section.as_mut().unwrap().text.clear(),
                    NamedKey::Backspace
                        if !self.section.clone().unwrap().text.is_empty() =>
                    {
                        let section = self.section.as_mut().unwrap();
                        let mut end_text = section.text.remove(section.text.len() - 1);
                        end_text.text.pop();
                        if !end_text.text.is_empty() {
                            self.section.as_mut().unwrap().text.push(end_text.clone());
                        }
                    }
                    _ => (),
                },
                Key::Character(char) => {
                    let c = char.as_str();
                    if c != "\u{7f}" && c != "\u{8}" {
                        if self.section.clone().unwrap().text.is_empty() {
                            self.section.as_mut().unwrap().text.push(
                                OwnedText::default()
                                    .with_scale(self.font_size)
                                    .with_color([0.9, 0.5, 0.5, 1.0]),
                            );
                        }
                        self.section.as_mut().unwrap().text.push(
                            OwnedText::new(c.to_string())
                                .with_scale(self.font_size)
                                .with_color([0.9, 0.5, 0.5, 1.0]),
                        );
                    }
                }
                _ => (),
            },
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                ..
            } => {
                // increase/decrease font size
                let mut size = self.font_size;
                if y > 0.0 {
                    size += (size / 4.0).max(2.0)
                } else {
                    size *= 4.0 / 5.0
                };
                self.font_size = (size.max(3.0).min(25000.0) * 2.0).round() / 2.0;
            }
            WindowEvent::RedrawRequested => {
                let brush = self.brush.as_mut().unwrap();
                let ctx = self.ctx.as_ref().unwrap();
                let queue = &ctx.queue;
                let device = &ctx.device;
                let config = &ctx.config;
                let surface = &ctx.surface;
                let section = self.section.as_ref().unwrap();
                let camera = self.camera.as_mut().unwrap();

                camera.update();
                queue.write_buffer(
                    self.matrix_buffer.as_ref().unwrap(),
                    0,
                    bytemuck::cast_slice(camera.global_matrix.as_slice()),
                );

                match brush.queue(device, queue, [section]) {
                    Ok(_) => (),
                    Err(err) => {
                        panic!("{err}");
                    }
                };

                let frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(_) => {
                        surface.configure(device, config);
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

                // Custom output render pass. Renders to custom texture.
                {
                    let mut rpass =
                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Custom Surface Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: self.texture_view.as_ref().unwrap(),
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                    brush.draw(&mut rpass);
                }

                // Default render pass:
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
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                    rpass.set_pipeline(self.pipeline.as_ref().unwrap());
                    rpass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
                    rpass.set_vertex_buffer(
                        0,
                        self.vertex_buffer.as_ref().unwrap().slice(..),
                    );
                    rpass.draw(0..6, 0..1);
                }

                queue.submit([encoder.finish()]);
                frame.present();
            }
            _ => (),
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
        if self.target_framerate <= self.delta_time.elapsed() {
            self.window.clone().unwrap().request_redraw();
            self.delta_time = Instant::now();
            self.fps += 1;
            if self.fps_update_time.elapsed().as_millis() > 1000 {
                let window = self.window.as_mut().unwrap();
                window.set_title(&format!(
                    "wgpu-text: 'custom_target' example, FPS: {}",
                    self.fps
                ));
                self.fps = 0;
                self.fps_update_time = Instant::now();
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        println!("Exiting!");
    }
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut state = State {
        window: None,
        font: include_bytes!("../fonts/DejaVuSans.ttf"),
        brush: None,
        font_size: 25.,
        section: None,
        camera: None,
        matrix_buffer: None,
        texture_view: None,
        pipeline: None,
        bind_group: None,
        vertex_buffer: None,

        // FPS and window updating:
        // change '60.0' if you want different FPS cap
        target_framerate: Duration::from_secs_f64(1.0 / 60.0),
        delta_time: Instant::now(),
        fps_update_time: Instant::now(),
        fps: 0,

        ctx: None,
    };

    let _ = event_loop.run_app(&mut state);
}
