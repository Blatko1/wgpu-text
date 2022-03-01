pub struct Pipeline {
    pipeline: wgpu::RenderPipeline
}

impl Pipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text rendering pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[]
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text rendering pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: todo!(),
                entry_point: "vs_main",
                buffers: todo!(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: todo!(),
            multisample: todo!(),
            fragment: todo!(),
            multiview: todo!(),
        });
        Self {
            pipeline
        }
    }
}