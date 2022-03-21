use std::{borrow::Cow, hash::BuildHasher};

use glyph_brush::{
    ab_glyph::{Font, FontArc, FontRef, InvalidFont},
    BrushAction, BrushError, DefaultSectionHasher, Extra, Section,
};
use wgpu::CommandBuffer;

use crate::pipeline::{Pipeline, Vertex};

/// Wrapper over [`glyph_brush::GlyphBrush`].
///
/// Used for queuing and rendering text with [`TextBrush::queue`] and [`TextBrush::draw_queued`].
pub struct TextBrush<F = FontArc, H = DefaultSectionHasher> {
    inner: glyph_brush::GlyphBrush<Vertex, Extra, F, H>,
    pipeline: Pipeline,
}

impl<F: Font + Sync, H: BuildHasher> TextBrush<F, H> {
    #[inline]
    pub fn queue<'a, S>(&mut self, section: S)
    where
        S: Into<Cow<'a, Section<'a>>>,
    {
        self.inner.queue(section);
    }

    pub fn draw_queued(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        queue: &wgpu::Queue,
    ) -> CommandBuffer {
        let mut brush_action;

        loop {
            brush_action = self.inner.process_queued(
                |rect, data| self.pipeline.update_texture(rect, data, queue),
                Vertex::to_vertex,
            );

            match brush_action {
                Ok(_) => break,
                Err(BrushError::TextureTooSmall { suggested }) => {
                    self.inner.resize_texture(suggested.0, suggested.1);
                    self.pipeline.resize_texture(device, suggested.0, suggested.1);
                }
            }
        }

        match brush_action.unwrap() {
            BrushAction::Draw(vertices) => self.pipeline.update(vertices, device, queue),
            BrushAction::ReDraw => (),
        }

        self.pipeline.draw(device, view)
    }

    pub fn resize(&mut self, width: f32, height: f32, queue: &wgpu::Queue) {
        self.pipeline.resize(width, height, queue);
    }
}

/// Builder for [`TextBrush`].
pub struct BrushBuilder<F, H = DefaultSectionHasher> {
    inner: glyph_brush::GlyphBrushBuilder<F, H>,
}

impl BrushBuilder<()> {
    #[inline]
    pub fn using_font<F: Font>(font: F) -> BrushBuilder<F> {
        BrushBuilder::using_fonts(vec![font])
    }

    #[inline]
    pub fn using_font_bytes(data: &[u8]) -> Result<BrushBuilder<FontRef>, InvalidFont> {
        let font = FontRef::try_from_slice(data)?;
        Ok(BrushBuilder::using_fonts(vec![font]))
    }

    pub fn using_fonts<F: Font>(fonts: Vec<F>) -> BrushBuilder<F> {
        BrushBuilder {
            inner: glyph_brush::GlyphBrushBuilder::using_fonts(fonts),
        }
    }
}

impl<F: Font, H: BuildHasher> BrushBuilder<F, H> {
    glyph_brush::delegate_glyph_brush_builder_fns!(inner);

    pub fn build(
        self,
        device: &wgpu::Device,
        render_format: wgpu::TextureFormat,
        width: f32,
        height: f32,
    ) -> TextBrush<F, H> {
        let inner = self.inner.build();
        let pipeline = Pipeline::new(
            device,
            render_format,
            inner.texture_dimensions(),
            (width, height),
        );
        TextBrush { inner, pipeline }
    }
}
