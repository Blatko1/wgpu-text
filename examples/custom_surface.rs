mod simple;

use glyph_brush::{OwnedText, VerticalAlign};
use nalgebra::{Matrix4, Point3, Vector3};
use simple::*;
use std::time::{Duration, Instant, SystemTime};
use wgpu::util::DeviceExt;
use wgpu_text::section::{BuiltInLineBreaker, Layout, Section, Text};
use wgpu_text::BrushBuilder;
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{self, ControlFlow},
    window::WindowBuilder,
};

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
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
    let compiler = shaderc::Compiler::new().unwrap();
    let vs_spirv = compiler
        .compile_into_spirv(
            include_str!("vertex.glsl"),
            shaderc::ShaderKind::Vertex,
            "vertex.glsl",
            "main",
            None,
        )
        .unwrap();

    let fs_spirv = compiler
        .compile_into_spirv(
            include_str!("fragment.glsl"),
            shaderc::ShaderKind::Fragment,
            "fragment.glsl",
            "main",
            None,
        )
        .unwrap();

    let mut camera = Camera::new(&config);

    let size = wgpu::Extent3d {
        width: 100,
        height: 100,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Custom Surface Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

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
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Custom Surface Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: matrix_buffer.as_entire_binding(),
        }],
    });

    let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Custom Surface Vertex Shader"),
        source: wgpu::util::make_spirv(vs_spirv.as_binary_u8()),
    });
    let fragment_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Custom Surface Fragment Shader"),
        source: wgpu::util::make_spirv(fs_spirv.as_binary_u8()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Custom Surface Render Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Custom Surface Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &vertex_module,
            entry_point: "main",
            buffers: &[Vertex::buffer_layout()],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment_module,
            entry_point: "main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
    });

    let font: &[u8] = include_bytes!("fonts/DejaVuSans.ttf");
    let mut brush = BrushBuilder::using_font_bytes(font).unwrap().build_custom(
        &device,
        size.width,
        size.height,
        texture.format(),
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

                //brush.queue(&section);

                //let cmd_buffer = brush.draw(&device, &view, &queue);
                // Has to be submitted last so it won't be overlapped.
                queue.submit([encoder.finish() /*, cmd_buffer*/]);
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

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    up: Vector3<f32>,
    pub aspect: f32,
    pub fov: f32,
    near: f32,
    far: f32,
    pub controller: CameraController,
    pub global_matrix: Matrix4<f32>,
}

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Camera {
    pub fn new(config: &wgpu::SurfaceConfiguration) -> Self {
        let controller = CameraController::new();
        Self {
            eye: Point3::new(0., 0., 1.),
            target: Point3::new(0., 0., -1.),
            up: Vector3::y(),
            aspect: config.width as f32 / config.height as f32,
            fov: 60.,
            near: 0.01,
            far: 100.0,
            controller,
            global_matrix: OPENGL_TO_WGPU_MATRIX,
        }
    }

    fn update_global_matrix(&mut self) {
        let target = Point3::new(
            self.eye.x + self.target.x,
            self.eye.y + self.target.y,
            self.eye.z + self.target.z,
        );
        let projection = Matrix4::new_perspective(
            self.aspect,
            self.fov.to_degrees(),
            self.near,
            self.far,
        );
        let view = Matrix4::look_at_rh(&self.eye, &target, &self.up);
        self.global_matrix = OPENGL_TO_WGPU_MATRIX * projection * view;
    }

    pub fn resize(&mut self, config: &wgpu::SurfaceConfiguration) {
        self.aspect = config.width as f32 / config.height as f32;
    }

    pub fn update(&mut self) {
        self.controller.update();
        self.fov += self.controller.fov_delta;
        self.controller.fov_delta = 0.;
        self.target = Point3::new(
            self.controller.yaw.to_radians().cos()
                * self.controller.pitch.to_radians().cos(),
            self.controller.pitch.to_radians().sin(),
            self.controller.yaw.to_radians().sin()
                * self.controller.pitch.to_radians().cos(),
        );
        let target = Vector3::new(self.target.x, 0.0, self.target.z).normalize();
        self.eye += &target
            * self.controller.speed
            * (self.controller.forward - self.controller.backward);
        self.eye += &target.cross(&self.up)
            * self.controller.speed
            * (self.controller.right - self.controller.left);
        self.eye += Vector3::new(0.0, 1.0, 0.0)
            * self.controller.speed
            * (self.controller.up - self.controller.down);
        self.update_global_matrix();
    }

    pub fn input(&mut self, event: &winit::event::DeviceEvent) {
        self.controller.process_input(event);
    }
}

pub struct CameraController {
    speed: f32,
    sensitivity: f64,
    forward: f32,
    backward: f32,
    left: f32,
    right: f32,
    up: f32,
    down: f32,
    pub yaw: f32,
    pub pitch: f32,
    fov_delta: f32,
}

impl CameraController {
    pub fn new() -> Self {
        CameraController {
            speed: 0.4,
            sensitivity: 0.1,
            forward: 0.,
            backward: 0.,
            left: 0.,
            right: 0.,
            up: 0.,
            down: 0.,
            yaw: 0.,
            pitch: 0.0,
            fov_delta: 0.,
        }
    }

    pub fn update(&mut self) {
        let time = (std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() % 100000) as f64 / 100.0;
        self.yaw = 270.0 + 40.0
            * (time as f64)
                .to_radians().sin() as f32;
        println!("{:?}", time as f32);
    }

    pub fn process_input(&mut self, event: &winit::event::DeviceEvent) {
        /*match event {
            DeviceEvent::MouseMotion { delta } => {
                self.yaw += (delta.0 * self.sensitivity) as f32;
                self.pitch -= (delta.1 * self.sensitivity) as f32;

                if self.pitch > 89.0 {
                    self.pitch = 89.0;
                } else if self.pitch < -89.0 {
                    self.pitch = -89.0;
                }

                if self.yaw > 360.0 {
                    self.yaw = 0.0;
                } else if self.yaw < 0.0 {
                    self.yaw = 360.0;
                }
            }
            DeviceEvent::Motion { .. } => {}
            DeviceEvent::Button { .. } => {}
            DeviceEvent::Key(KeyboardInput {
                state,
                virtual_keycode,
                ..
            }) => {
                let value: f32;
                if *state == winit::event::ElementState::Pressed {
                    value = 1.
                } else {
                    value = 0.;
                }
                match virtual_keycode.unwrap() {
                    VirtualKeyCode::Space => {
                        self.up = value;
                    }
                    VirtualKeyCode::LShift => {
                        self.down = value;
                    }
                    VirtualKeyCode::W => {
                        self.forward = value;
                    }
                    VirtualKeyCode::S => {
                        self.backward = value;
                    }
                    VirtualKeyCode::A => {
                        self.left = value;
                    }
                    VirtualKeyCode::D => {
                        self.right = value;
                    }
                    _ => (),
                }
            }
            _ => (),
        }*/
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    pub fn buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        }
    }
}
