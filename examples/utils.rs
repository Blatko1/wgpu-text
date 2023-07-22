use pollster::block_on;

// TODO add cache texture preview example
// TODO add mip-mapping example
// TODO add wasm example
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
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
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

        let size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .expect("Surface isn't supported by the adapter.");
        surface.configure(&device, &config);

        (device, queue, surface, config)
    }
}

#[allow(dead_code)]
fn main() {}
