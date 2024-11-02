#[path = "ctx.rs"]
mod ctx;

use ctx::Ctx;
use glyph_brush::ab_glyph::FontRef;
use glyph_brush::OwnedSection;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::{DepthStencilState, TextureView};
use wgpu_text::glyph_brush::{
    BuiltInLineBreaker, Layout, OwnedText, Section, Text, VerticalAlign,
};
use wgpu_text::{BrushBuilder, TextBrush};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event::{KeyEvent, MouseScrollDelta};
use winit::event_loop::{self, ActiveEventLoop, ControlFlow};
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

struct State<'a> {
    // Use an `Option` to allow the window to not be available until the
    // application is properly running.
    window: Option<Arc<Window>>,
    font: &'a [u8],
    brush: Option<TextBrush<FontRef<'a>>>,
    font_size: f32,
    section_0: Option<OwnedSection>,
    section_1: Option<OwnedSection>,

    target_framerate: Duration,
    delta_time: Instant,
    fps_update_time: Instant,
    fps: i32,

    // wgpu
    ctx: Option<Ctx>,
    depth_stencil: Option<DepthStencilState>,
    depth_view: Option<TextureView>,
}

impl ApplicationHandler for State<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes().with_title("wgpu-text: 'depth' example"),
                )
                .unwrap(),
        );

        self.ctx = Some(Ctx::new(window.clone()));

        let ctx = self.ctx.as_ref().unwrap();
        let device = &ctx.device;
        let config = &ctx.config;
        self.depth_view = Some(create_depth_view(device, config.width, config.height));

        self.brush = Some(
            BrushBuilder::using_font_bytes(self.font)
                .unwrap()
                .with_depth_stencil(self.depth_stencil.clone())
                .build(device, config.width, config.height, config.format),
        );

        self.section_0 = Some(
            Section::default()
                .add_text(
                    Text::new(
                        "Try typing some text,\n \
                del - delete all, backspace - remove last character",
                    )
                    .with_scale(self.font_size)
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
                .to_owned(),
        );

        self.section_1 = Some(
            Section::default()
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
                .to_owned(),
        );

        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        elwt: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                let ctx = self.ctx.as_mut().unwrap();
                let queue = &ctx.queue;
                let device = &ctx.device;
                let config = &mut ctx.config;
                let surface = &ctx.surface;
                let section_0 = self.section_0.as_mut().unwrap();
                let brush = self.brush.as_mut().unwrap();

                config.width = new_size.width.max(1);
                config.height = new_size.height.max(1);
                surface.configure(device, config);

                let width = config.width as f32;
                let height = config.height as f32;

                section_0.bounds = (width * 0.5, height);
                section_0.screen_position.1 = height * 0.5;

                self.depth_view.replace(create_depth_view(
                    device,
                    config.width,
                    config.height,
                ));
                brush.resize_view(width, height, queue);

                // You can also do this!
                // brush.update_matrix(wgpu_text::ortho(config.width, config.height), &queue);
            }
            WindowEvent::CloseRequested => elwt.exit(),
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
                    NamedKey::Escape => elwt.exit(),
                    NamedKey::Delete => self.section_0.as_mut().unwrap().text.clear(),
                    NamedKey::Backspace
                        if !self.section_0.clone().unwrap().text.is_empty() =>
                    {
                        let section = self.section_0.as_mut().unwrap();
                        let mut end_text = section.text.remove(section.text.len() - 1);
                        end_text.text.pop();
                        if !end_text.text.is_empty() {
                            self.section_0.as_mut().unwrap().text.push(end_text.clone());
                        }
                    }
                    _ => (),
                },
                Key::Character(char) => {
                    let c = char.as_str();
                    if c != "\u{7f}" && c != "\u{8}" {
                        let section_0 = self.section_0.as_mut().unwrap();
                        if section_0.text.is_empty() {
                            section_0.text.push(
                                OwnedText::default()
                                    .with_scale(self.font_size)
                                    .with_color([0.9, 0.5, 0.5, 1.0])
                                    .with_z(0.08),
                            );
                        }
                        section_0.text.push(
                            OwnedText::new(c.to_string())
                                .with_scale(self.font_size)
                                .with_color([0.9, 0.5, 0.5, 1.0])
                                .with_z(0.08),
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
                self.font_size = (size.clamp(3.0, 25000.0) * 2.0).round() / 2.0;
            }
            WindowEvent::RedrawRequested => {
                let brush = self.brush.as_mut().unwrap();
                let ctx = self.ctx.as_ref().unwrap();
                let queue = &ctx.queue;
                let device = &ctx.device;
                let config = &ctx.config;
                let surface = &ctx.surface;
                let section_0 = self.section_0.as_ref().unwrap();
                let section_1 = self.section_1.as_ref().unwrap();

                match brush.queue(device, queue, [section_1, section_0]) {
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
                                    view: self.depth_view.as_ref().unwrap(),
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
            }
            _ => (),
        }
    }

    fn new_events(&mut self, _elwt: &ActiveEventLoop, _cause: winit::event::StartCause) {
        if self.target_framerate <= self.delta_time.elapsed() {
            self.window.clone().unwrap().request_redraw();
            self.delta_time = Instant::now();
            self.fps += 1;
            if self.fps_update_time.elapsed().as_millis() > 1000 {
                let window = self.window.as_mut().unwrap();
                window
                    .set_title(&format!("wgpu-text: 'depth' example, FPS: {}", self.fps));
                self.fps = 0;
                self.fps_update_time = Instant::now();
            }
        }
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

    let event_loop = event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut state = State {
        window: None,
        font: include_bytes!("fonts/DejaVuSans.ttf"),
        brush: None,
        font_size: 45.,
        section_0: None,
        section_1: None,

        // FPS and window updating:
        // change '60.0' if you want different FPS cap
        target_framerate: Duration::from_secs_f64(1.0 / 60.0),
        delta_time: Instant::now(),
        fps_update_time: Instant::now(),
        fps: 0,

        ctx: None,
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        depth_view: None,
    };

    let _ = event_loop.run_app(&mut state);
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
