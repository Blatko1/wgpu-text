use glyph_brush::{BuiltInLineBreaker, Layout, Section, Text, VerticalAlign};
use pollster::block_on;
use wgpu::{Features, Limits};
use wgpu_text::BrushBuilder;
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{self, ControlFlow},
    window::WindowBuilder,
};

fn main() {
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

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_preferred_format(&adapter).unwrap(),
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &config);

    let font: &[u8] = include_bytes!("Inconsolata-Regular.ttf");
    let mut brush = BrushBuilder::using_font_bytes(font).unwrap().build();

    let mut section = Section::default()
        .add_text(
            Text::new(
                "* Type text\n\
                 * Scroll to set typed size (see window title)\n\
                 * ctrl r  Clear & reorder draw cache\n\
                 * ctrl shift r  Reset & resize draw cache\n\
                 * ctrl backspace  Delete all text\n\
                ",
            )
            .with_scale(25.0)
            .with_color([0.5, 0.5, 0.5, 1.0]),
        )
        .with_bounds((size.width as f32 / 2.0, size.height as f32))
        .with_layout(
            Layout::default()
                .v_align(VerticalAlign::Center)
                .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
        )
        .with_screen_position((0.0, size.height as f32 * 0.5))
        .to_owned();
    brush.queue(&section);
    brush.draw_queued(&device, &queue);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size)
                | WindowEvent::ScaleFactorChanged {
                    new_inner_size: &mut new_size,
                    ..
                } => {
                    config.width = new_size.width;
                    config.height = new_size.height;
                    surface.configure(&device, &config);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::ReceivedCharacter(_) => (),
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

            winit::event::Event::MainEventsCleared => window.request_redraw(),
            winit::event::Event::RedrawRequested(_) => {
                let frame = surface.get_current_texture().unwrap();
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                
                frame.present();
            },
            winit::event::Event::RedrawEventsCleared => (),
            _ => (),
        }
    });
}
