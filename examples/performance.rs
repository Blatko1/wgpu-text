#[path = "ctx.rs"]
mod ctx;

use ctx::Ctx;
use glyph_brush::ab_glyph::FontRef;
use rand::Rng;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu_text::glyph_brush::{BuiltInLineBreaker, Layout, Section, Text};
use wgpu_text::{BrushBuilder, TextBrush};
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, MouseScrollDelta};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;
use winit::{
    event::{ElementState, WindowEvent},
    event_loop::{self},
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

struct State<'a> {
    // Use an `Option` to allow the window to not be available until the
    // application is properly running.
    window: Option<Arc<Window>>,
    font: &'a [u8],
    brush: Option<TextBrush<FontRef<'a>>>,
    random_text: String,
    font_size: f32,

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
                let brush = self.brush.as_mut().unwrap();

                config.width = new_size.width.max(1);
                config.height = new_size.height.max(1);
                surface.configure(device, config);

                brush.resize_view(config.width as f32, config.height as f32, queue);
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
                    NamedKey::Delete => self.random_text.clear(),
                    NamedKey::Backspace => {
                        self.random_text.pop();
                    }
                    _ => (),
                },
                Key::Character(char) => {
                    self.random_text.push_str(char.as_str());
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
                self.font_size = (size.clamp(3.0, 300.0) * 2.0).round() / 2.0;
            }
            WindowEvent::RedrawRequested => {
                let brush = self.brush.as_mut().unwrap();
                let ctx = self.ctx.as_ref().unwrap();
                let queue = &ctx.queue;
                let device = &ctx.device;
                let config = &ctx.config;
                let surface = &ctx.surface;

                let section = Section::default()
                    .add_text(
                        Text::new(&self.random_text)
                            .with_scale(self.font_size)
                            .with_color([0.9, 0.5, 0.5, 1.0]),
                    )
                    .with_bounds((config.width as f32, config.height as f32))
                    .with_layout(
                        Layout::default()
                            .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
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
                window.set_title(&format!(
                    "wgpu-text: 'performance' example, FPS: {}",
                    self.fps
                ));
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

    if cfg!(debug_assertions) {
        eprintln!(
            "You should probably run an example called 'performance' in release mode.\n\
            e.g. use `cargo run --example performance --release`\n"
        );
    }

    let event_loop = event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut state = State {
        window: None,
        font: include_bytes!("fonts/DejaVuSans.ttf"),
        brush: None,
        random_text: generate_random_chars(),
        font_size: 9.,

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
