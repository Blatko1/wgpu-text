use pollster::block_on;

// TODO add cache texture preview example
// TODO add mip-mapping example
pub struct WgpuUtils;

impl WgpuUtils {
    pub fn init(
        window: &winit::window::Window,
    ) -> (
        wgpu::Device,
        wgpu::Queue,
        wgpu::Surface,
        wgpu::SurfaceConfiguration,
    ) {
        let backends =
            wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        });
        let (size, surface) = unsafe {
            let size = window.inner_size();
            let surface = instance.create_surface(&window).unwrap();
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
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![format],
        };
        surface.configure(&device, &config);
        (device, queue, surface, config)
    }
}

#[allow(dead_code)]
fn main() {}
