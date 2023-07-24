use glyph_brush::{
    ab_glyph::{Font, FontArc},
    DefaultSectionHasher, Rectangle, Section, Extra,
};
use std::{borrow::Cow, num::NonZeroU32, marker::PhantomData};
use wgpu::util::DeviceExt;

use crate::{cache::Cache, Matrix, TextBrush, BrushError, brush::process_queued};

/// Responsible for drawing text.
#[derive(Debug)]
pub struct Pipeline {
    inner: wgpu::RenderPipeline,
    pub(crate) cache: Cache,

    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    matrix_buffer: wgpu::Buffer,

    vertex_buffer: wgpu::Buffer,
    vertex_buffer_max_len: usize,
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
        let cache = Cache::new(device, tex_dimensions);

        let shader =
            device.create_shader_module(wgpu::include_wgsl!("shader/shader.wgsl"));

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wgpu-text Vertex Buffer"),
            size: 0,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let matrix_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("wgpu-text Matrix Buffer"),
                contents: bytemuck::cast_slice(&matrix),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("wgpu-text Matrix, Texture and Sampler Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wgpu-text Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: matrix_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &cache.texture.create_view(&wgpu::TextureViewDescriptor::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&cache.sampler),
                },
            ],
        });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("wgpu-text Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
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

            bind_group_layout,
            bind_group,
            matrix_buffer,

            vertex_buffer,
            vertex_buffer_max_len: 0,
            vertex_count: 0,
        }
    }

    /// Raw draw.
    pub fn draw<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
        if self.vertex_count != 0 {
            rpass.set_pipeline(&self.inner);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_bind_group(0, &self.bind_group, &[]);

            rpass.draw(0..4, 0..self.vertex_count);
        }
    }

    pub fn update_vertex_buffer(
        &mut self,
        vertices: Vec<Vertex>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        Self::_update_vertex_buffer(&mut self.vertex_count, &mut self.vertex_buffer_max_len, &mut self.vertex_buffer,vertices, device, queue);
    }

    // TODO look into preallocating the vertex buffer instead of constantly reallocating
    fn _update_vertex_buffer(
        vertex_count: &mut u32,
        buffer_max_len: &mut usize,
        buffer: &mut wgpu::Buffer,
        vertices: Vec<Vertex>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        *vertex_count = vertices.len() as u32;
        let data: &[u8] = bytemuck::cast_slice(&vertices);

        if vertices.len() > *buffer_max_len {
            *buffer_max_len = vertices.len();

            *buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("wgpu-text Vertex Buffer"),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    contents: data,
                });

            return;
        }
        queue.write_buffer(buffer, 0, data);
    }

    #[inline]
    pub fn update_matrix(&self, matrix: Matrix, queue: &wgpu::Queue) {
        queue.write_buffer(&self.matrix_buffer, 0, bytemuck::cast_slice(&matrix));
    }

    #[inline]
    pub fn update_texture(&self, size: Rectangle<u32>, data: &[u8], queue: &wgpu::Queue) {
        self.cache.update_texture(size, data, queue);
    }

    #[inline]
    pub fn resize_texture(&mut self, device: &wgpu::Device, tex_dimensions: (u32, u32)) {
        self.cache.recreate_texture(device, tex_dimensions);
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wgpu-text Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.matrix_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &self.cache.texture.create_view(&wgpu::TextureViewDescriptor::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.cache.sampler),
                },
            ],
        });
    }
}

pub struct DrawChain<'a, S> where S: Into<Cow<'a, Section<'a>>> + Clone {
    draws: Vec<DrawCall<'a, S>>,
    phantom: PhantomData<&'a S>
}

impl<'a, S> DrawChain<'a, S> where S: Into<Cow<'a, Section<'a>>> + Clone {
    pub(crate) fn new(
    ) -> Self {
        Self {draws: Vec::new(), phantom: PhantomData}
    }

    pub fn add_draw(mut self, content: Vec<S>, matrix: Option<Matrix>) -> Self {
        self.draws.push(DrawCall {
            content: content,
            matrix,
            phantom: PhantomData,
        });
        self
    }

    pub fn draw<'rpass, F, H>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>, brush: &'rpass mut TextBrush<F, H>, device: &'rpass wgpu::Device, queue: &'rpass wgpu::Queue) -> Result<(), BrushError> where
    F: Font + Sync,
    H: std::hash::BuildHasher
    {
        self._draw(rpass, device, queue, &mut brush.pipeline, &mut brush.inner)?;

        Ok(())
    }

    fn _draw<'rpass, F, H>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>, device: &'rpass wgpu::Device, queue: &'rpass wgpu::Queue, pipeline: &'rpass mut Pipeline, inner: &'rpass mut glyph_brush::GlyphBrush<Vertex, Extra, F, H>) -> Result<(), BrushError> where
    F: Font + Sync,
    H: std::hash::BuildHasher
    {
        rpass.set_pipeline(&pipeline.inner);
        rpass.set_bind_group(0, &pipeline.bind_group, &[]);
        rpass.set_vertex_buffer(0, pipeline.vertex_buffer.slice(..));
        for draw in &self.draws {
            if let Some(matrix) = draw.matrix {
                queue.write_buffer(&pipeline.matrix_buffer, 0, bytemuck::cast_slice(&matrix));
            }
            match process_queued(inner, &mut pipeline.cache, device, queue, draw.content.clone())? {
                glyph_brush::BrushAction::Draw(vertices) => Pipeline::_update_vertex_buffer(&mut pipeline.vertex_count, &mut pipeline.vertex_buffer_max_len, &mut pipeline.vertex_buffer, vertices, device, queue),
                glyph_brush::BrushAction::ReDraw => (),
            };

            rpass.draw(0..4, 0..pipeline.vertex_count);
        }

        Ok(())
    }
}

struct DrawCall<'a, S> where S: Into<Cow<'a, Section<'a>>> + Clone {
    content: Vec<S>,
    matrix: Option<Matrix>,
    phantom: PhantomData<&'a S>
}

use glyph_brush::ab_glyph::{point, Rect};

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
            min: point(pixel_coords.min.x, pixel_coords.min.y),
            max: point(pixel_coords.max.x, pixel_coords.max.y),
        };

        // handle overlapping bounds, modify uv_rect to preserve texture aspect
        if rect.max.x > bounds.max.x {
            let old_width = rect.width();
            rect.max.x = bounds.max.x;
            tex_coords.max.x =
                tex_coords.min.x + tex_coords.width() * rect.width() / old_width;
        }
        if rect.min.x < bounds.min.x {
            let old_width = rect.width();
            rect.min.x = bounds.min.x;
            tex_coords.min.x =
                tex_coords.max.x - tex_coords.width() * rect.width() / old_width;
        }
        if rect.max.y > bounds.max.y {
            let old_height = rect.height();
            rect.max.y = bounds.max.y;
            tex_coords.max.y =
                tex_coords.min.y + tex_coords.height() * rect.height() / old_height;
        }
        if rect.min.y < bounds.min.y {
            let old_height = rect.height();
            rect.min.y = bounds.min.y;
            tex_coords.min.y =
                tex_coords.max.y - tex_coords.height() * rect.height() / old_height;
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
