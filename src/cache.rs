use glyph_brush::Rectangle;
use wgpu::util::DeviceExt;

use crate::Matrix;

/// Responsible for texture caching and the global matrix.
#[derive(Debug)]
pub struct Cache {
    pub(crate) texture: wgpu::Texture,
    pub(crate) sampler: wgpu::Sampler,
}

impl Cache {
    pub fn new(
        device: &wgpu::Device,
        tex_dimensions: (u32, u32)
    ) -> Self {
        let texture = Self::create_cache_texture(device, tex_dimensions);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("wgpu-text Cache Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            texture,
            sampler
        }
    }

    pub fn recreate_texture(
        &mut self,
        device: &wgpu::Device,
        tex_dimensions: (u32, u32),
    ) {
        self.texture = Self::create_cache_texture(device, tex_dimensions);
    }

    pub fn update_texture(&self, size: Rectangle<u32>, data: &[u8], queue: &wgpu::Queue) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: size.min[0],
                    y: size.min[1],
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size.width()),
                rows_per_image: Some(size.height()),
            },
            wgpu::Extent3d {
                width: size.width(),
                height: size.height(),
                depth_or_array_layers: 1,
            },
        )
    }

    fn create_cache_texture(
        device: &wgpu::Device,
        dimensions: (u32, u32),
    ) -> wgpu::Texture {
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("wgpu-text Cache Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }
}
