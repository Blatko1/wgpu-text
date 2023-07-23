use glyph_brush::Rectangle;
use std::num::NonZeroU32;
use wgpu::util::DeviceExt;

use crate::{internal::cache::Cache, vertex::Vertex, Matrix};

/// Responsible for drawing text.
#[derive(Debug)]
pub struct Pipeline {
    pub inner: wgpu::RenderPipeline,
    cache: Cache,

    vertex_buffer: wgpu::Buffer,
    vertex_buffer_len: usize,
    vertex_count: u32,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        render_format: wgpu::TextureFormat,
        depth_stencil: Option<wgpu::DepthStencilState>,
        multisample: wgpu::MultisampleState,
        multiview: Option<NonZeroU32>,
        tex_dimensions: (u32, u32),
        matrix: Matrix,
    ) -> Pipeline {
        let cache = Cache::new(device, tex_dimensions, matrix);

        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../shader/shader.wgsl"));

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wgpu-text Vertex Buffer"),
            size: 0,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("wgpu-text Render Pipeline Layout"),
                bind_group_layouts: &[&cache.bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("wgpu-text Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::buffer_layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint16),
                ..Default::default()
            },
            depth_stencil,
            multisample,
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview,
        });

        Self {
            inner: pipeline,
            cache,

            vertex_buffer,
            vertex_buffer_len: 0,
            vertex_count: 0,
        }
    }

    /// Raw draw.
    pub fn draw<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
        if self.vertex_count != 0 {
            rpass.set_pipeline(&self.inner);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_bind_group(0, &self.cache.bind_group, &[]);

            rpass.draw(0..4, 0..self.vertex_count);
        }
    }
    // TODO look into preallocating the vertex buffer instead of constantly reallocating
    pub fn update_vertex_buffer(
        &mut self,
        vertices: Vec<Vertex>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.vertex_count = vertices.len() as u32;
        let data: &[u8] = bytemuck::cast_slice(&vertices);

        if vertices.len() > self.vertex_buffer_len {
            self.vertex_buffer_len = vertices.len();

            self.vertex_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("wgpu-text Vertex Buffer"),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    contents: data,
                });

            return;
        }
        queue.write_buffer(&self.vertex_buffer, 0, data);
    }

    #[inline]
    pub fn update_matrix(&self, matrix: Matrix, queue: &wgpu::Queue) {
        self.cache.update_matrix(matrix, queue);
    }

    #[inline]
    pub fn update_texture(&self, size: Rectangle<u32>, data: &[u8], queue: &wgpu::Queue) {
        self.cache.update_texture(size, data, queue);
    }

    #[inline]
    pub fn resize_texture(&mut self, device: &wgpu::Device, tex_dimensions: (u32, u32)) {
        self.cache.recreate_texture(device, tex_dimensions);
    }
}
