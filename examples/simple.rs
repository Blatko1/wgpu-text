#[path = "utils.rs"]
mod utils;

use glyph_brush::ab_glyph::FontRef;
use glyph_brush::OwnedSection;
use std::borrow::Borrow;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use utils::WgpuUtils;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
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

struct Dqsc {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
}

struct State<'a> {
    // Use an `Option` to allow the window to not be available until the
    // application is properly running.
    window: Option<Arc<Window>>,
    font: &'a [u8],
    brush: Option<TextBrush<FontRef<'a>>>,
    font_size: f32,
    section_0: Option<OwnedSection>,
    section_1: Option<OwnedSection>,
    then: SystemTime,
    now: SystemTime,
    fps: i32,
    target_framerate: Duration,
    delta_time: Instant,

    // wgpu
    dqsc: Option<Dqsc>,
}

impl ApplicationHandler for State<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("wgpu-text: 'simple' example"),
                )
                .unwrap(),
        ));

        if let Some(window) = self.window.borrow() {
            let (device, queue, surface, config) = WgpuUtils::init(window.clone());

            self.dqsc = Some(Dqsc {
                device,
                queue,
                surface,
                config,
            });

            if let Some(dqsc) = &self.dqsc {
                let device = &dqsc.device;
                let config = &dqsc.config;
                self.brush =
                    Some(BrushBuilder::using_font_bytes(self.font).unwrap().build(
                        &device,
                        config.width,
                        config.height,
                        config.format,
                    ));

                self.section_0 = Some(
                    Section::default()
                        .add_text(
                            Text::new(
                                "Try typing some text,\n \
                del - delete all, backspace - remove last character",
                            )
                            .with_scale(self.font_size)
                            .with_color([0.9, 0.5, 0.5, 1.0]),
                        )
                        .with_bounds((config.width as f32 * 0.4, config.height as f32))
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
                        .to_owned(),
                );
            }
        }
    }

    fn window_event(
        &mut self,
        elwt: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                if let Some(dqsc) = &mut self.dqsc {
                    let queue = &dqsc.queue;
                    let device = &dqsc.device;
                    let config = &mut dqsc.config;
                    let surface = &dqsc.surface;

                    config.width = new_size.width.max(1);
                    config.height = new_size.height.max(1);
                    surface.configure(&device, &config);

                    if let Some(section_0) = &mut self.section_0 {
                        section_0.bounds =
                            (config.width as f32 * 0.4, config.height as _);
                        section_0.screen_position.1 = config.height as f32 * 0.5;
                    }

                    if let Some(brush) = &mut self.brush {
                        brush.resize_view(
                            config.width as f32,
                            config.height as f32,
                            &queue,
                        );
                    }
                    // You can also do this!
                    // brush.update_matrix(wgpu_text::ortho(config.width, config.height), &queue);
                }
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
                            println!("{:?}", end_text);
                        }
                    }
                    _ => (),
                },
                Key::Character(char) => {
                    let c = char.as_str();
                    if c != "\u{7f}" && c != "\u{8}" {
                        if self.section_0.clone().unwrap().text.is_empty() {
                            self.section_0.as_mut().unwrap().text.push(
                                OwnedText::default()
                                    .with_scale(self.font_size)
                                    .with_color([0.9, 0.5, 0.5, 1.0]),
                            );
                            println!("Clearning text field");
                        }
                        self.section_0.as_mut().unwrap().text.push(
                            OwnedText::new(c.to_string())
                                .with_scale(self.font_size)
                                .with_color([0.9, 0.5, 0.5, 1.0]),
                        );
                        println!("{:?}", c);
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
                let window = self.window.as_mut().unwrap();

                if let Some(brush) = &mut self.brush {
                    if let Some(dqsc) = &self.dqsc {
                        let queue = &dqsc.queue;
                        let device = &dqsc.device;
                        let config = &dqsc.config;
                        let surface = &dqsc.surface;
                        let section_0 = self.section_0.as_ref().unwrap();
                        let section_1 = self.section_1.as_ref().unwrap();

                        match brush.queue(&device, &queue, vec![section_0, section_1]) {
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

                        let mut encoder = device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor {
                                label: Some("Command Encoder"),
                            },
                        );

                        {
                            let mut rpass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("Render Pass"),
                                    color_attachments: &[Some(
                                        wgpu::RenderPassColorAttachment {
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
                                        },
                                    )],
                                    depth_stencil_attachment: None,
                                    timestamp_writes: None,
                                    occlusion_query_set: None,
                                });

                            brush.draw(&mut rpass);
                        }

                        queue.submit([encoder.finish()]);
                        frame.present();

                        self.fps += 1;
                        if self.now.duration_since(self.then).unwrap().as_millis() > 1000
                        {
                            window.set_title(&format!(
                                "wgpu-text: 'simple' example, FPS: {}",
                                self.fps
                            ));
                            self.fps = 0;
                            self.then = self.now;
                        }
                        self.now = SystemTime::now();
                    }
                }
            }
            _ => (),
        }
    }

    fn new_events(&mut self, elwt: &ActiveEventLoop, _cause: winit::event::StartCause) {
        if self.target_framerate <= self.delta_time.elapsed() {
            self.window.clone().unwrap().request_redraw();
            self.delta_time = Instant::now();
        } else {
            elwt.set_control_flow(ControlFlow::WaitUntil(
                Instant::now()
                    .checked_sub(self.delta_time.elapsed())
                    .unwrap()
                    + self.target_framerate,
            ));
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
        println!("Exiting!");
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
}

// TODO text layout of characters like 'š, ć, ž, đ' doesn't work correctly.
fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = event_loop::EventLoop::new().unwrap();
    let mut state = State {
        window: None,
        font: include_bytes!("fonts/DejaVuSans.ttf"),
        brush: None,
        font_size: 25.,
        section_0: None,
        section_1: None,
        then: SystemTime::now(),
        now: SystemTime::now(),
        fps: 0,

        // FPS and window updating:
        // change '60.0' if you want different FPS cap
        target_framerate: Duration::from_secs_f64(1.0 / 60.0),
        delta_time: Instant::now(),
        dqsc: None,
    };

    let _ = event_loop.run_app(&mut state);
}
