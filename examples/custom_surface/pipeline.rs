use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_pos: [f32; 2],
}

impl Vertex {
    pub fn buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

pub fn create_pipeline(
    device: &wgpu::Device,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::RenderPipeline {
    let compiler = shaderc::Compiler::new().unwrap();
    let vs_spirv = compiler
        .compile_into_spirv(
            include_str!("shaders/vertex.glsl"),
            shaderc::ShaderKind::Vertex,
            "vertex.glsl",
            "main",
            None,
        )
        .unwrap();

    let fs_spirv = compiler
        .compile_into_spirv(
            include_str!("shaders/fragment.glsl"),
            shaderc::ShaderKind::Fragment,
            "fragment.glsl",
            "main",
            None,
        )
        .unwrap();
    std::fs::write("vertex.glsl.spv", vs_spirv.as_binary_u8());
    std::fs::write("fragment.glsl.spv", fs_spirv.as_binary_u8());
    panic!("a");

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
        bind_group_layouts,
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
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
    });
    pipeline
}
