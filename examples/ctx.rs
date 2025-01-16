use std::sync::Arc;

use pollster::block_on;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};

// TODO add cache texture preview example
// TODO add mip-mapping example
// TODO add wasm example
pub struct Ctx {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
}

impl Ctx {
    pub fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();
        let backends =
            wgpu::Backends::from_env().unwrap_or_else(wgpu::Backends::all);
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions {
                gl: wgpu::GlBackendOptions {
                    gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
                },
                dx12: wgpu::Dx12BackendOptions {
                    shader_compiler: wgpu::Dx12Compiler::Fxc,
                },
            },
        });
        let surface = instance.create_surface(window).unwrap();

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("No adapters found!");

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ))
        .unwrap();

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .expect("Surface isn't supported by the adapter.");
        surface.configure(&device, &config);

        Self {
            device,
            queue,
            surface,
            config,
        }
    }
}

#[allow(dead_code)]
fn main() {}
