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
    let vertex_module =
        device.create_shader_module(wgpu::include_wgsl!("shaders/vertex.vert.wgsl"));
    let fragment_module =
        device.create_shader_module(wgpu::include_wgsl!("shaders/fragment.frag.wgsl"));

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
            entry_point: Some("main"),
            buffers: &[Vertex::buffer_layout()],
            compilation_options: Default::default(),
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
            entry_point: Some("main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview: None,
        cache: None,
    });
    pipeline
}
