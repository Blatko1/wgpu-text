use glyph_brush::{
    ab_glyph::{point, Rect},
    Rectangle,
};
use wgpu::{util::DeviceExt, CommandBuffer};

use crate::cache::Cache;

/// Responsible for drawing text.
pub struct Pipeline {
    inner: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    vertex_buffer_len: usize,
    vertices: u32,

    cache: Cache,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        render_format: wgpu::TextureFormat,
        tex_dimensions: (u32, u32),
        matrix: [f32; 16],
    ) -> Self {
        let cache = Cache::new(device, tex_dimensions.0, tex_dimensions.1, matrix);

        let shader = device.create_shader_module(&wgpu::include_wgsl!("shader/text.wgsl"));

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wgpu-text Vertex Buffer"),
            size: 0,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: render_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            multiview: None,
        });

        Self {
            inner: pipeline,
            vertex_buffer,
            vertex_buffer_len: 0,
            vertices: 0,
            cache,
        }
    }

    pub fn update_buffer(
        &mut self,
        vertices: Vec<Vertex>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.vertices = vertices.len() as u32;
        let data: &[u8] = bytemuck::cast_slice(&vertices);

        if vertices.len() > self.vertex_buffer_len {
            self.vertex_buffer_len = vertices.len();

            self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("wgpu-text Vertex Buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                contents: data,
            });

            return;
        }
        queue.write_buffer(&self.vertex_buffer, 0, data);
    }

    pub fn draw(
        &self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        region: Option<crate::ScissorRegion>,
    ) -> CommandBuffer {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("wgpu-text Command Encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("wgpu-text Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.inner);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_bind_group(0, &self.cache.bind_group, &[]);

            if let Some(r) = region {
                if r.is_contained() {
                    let (w, h) = r.available_bounds();
                    rpass.set_scissor_rect(r.x, r.y, w, h);
                }
            }

            rpass.draw(0..4, 0..self.vertices);
        }

        encoder.finish()
    }

    pub fn update_matrix(&mut self, matrix: [f32; 16], queue: &wgpu::Queue) {
        self.cache.update_matrix(matrix, queue);
    }

    pub fn update_texture(&mut self, size: Rectangle<u32>, data: &[u8], queue: &wgpu::Queue) {
        self.cache.update_texture(size, data, queue);
    }

    pub fn resize_texture(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.cache.recreate_texture(device, width, height);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    top_left: [f32; 3],
    bottom_right: [f32; 2],
    tex_top_left: [f32; 2],
    tex_bottom_right: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    pub fn to_vertex(
        glyph_brush::GlyphVertex {
            mut tex_coords,
            pixel_coords,
            bounds,
            extra,
        }: glyph_brush::GlyphVertex,
    ) -> Vertex {
        let bounds = bounds;

        let mut rect = Rect {
            min: point(pixel_coords.min.x as f32, pixel_coords.min.y as f32),
            max: point(pixel_coords.max.x as f32, pixel_coords.max.y as f32),
        };

        // handle overlapping bounds, modify uv_rect to preserve texture aspect
        if rect.max.x > bounds.max.x {
            let old_width = rect.width();
            rect.max.x = bounds.max.x;
            tex_coords.max.x = tex_coords.min.x + tex_coords.width() * rect.width() / old_width;
        }
        if rect.min.x < bounds.min.x {
            let old_width = rect.width();
            rect.min.x = bounds.min.x;
            tex_coords.min.x = tex_coords.max.x - tex_coords.width() * rect.width() / old_width;
        }
        if rect.max.y > bounds.max.y {
            let old_height = rect.height();
            rect.max.y = bounds.max.y;
            tex_coords.max.y = tex_coords.min.y + tex_coords.height() * rect.height() / old_height;
        }
        if rect.min.y < bounds.min.y {
            let old_height = rect.height();
            rect.min.y = bounds.min.y;
            tex_coords.min.y = tex_coords.max.y - tex_coords.height() * rect.height() / old_height;
        }

        Vertex {
            top_left: [rect.min.x, rect.min.y, extra.z],
            bottom_right: [rect.max.x, rect.max.y],
            tex_top_left: [tex_coords.min.x, tex_coords.min.y],
            tex_bottom_right: [tex_coords.max.x, tex_coords.max.y],
            color: extra.color,
        }
    }

    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
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
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    shader_location: 4,
                },
            ],
        }
    }
}
