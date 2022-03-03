use std::{borrow::Cow, hash::BuildHasher};

use glyph_brush::{
    ab_glyph::{Font, FontArc, FontRef, InvalidFont},
    DefaultSectionHasher, Extra, Section,
};

use crate::pipeline::{Pipeline, Vertex};

pub struct GlyphBrush<F = FontArc, H = DefaultSectionHasher> {
    inner: glyph_brush::GlyphBrush<Vertex, Extra, F, H>,
    pipeline: Pipeline,
}

impl<F: Font + Sync, H: BuildHasher> GlyphBrush<F, H> {
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
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
    ) {
        let brush_action = self
            .inner
            .process_queued(
                |rect, tex_data| {
                    println!("rect: {:?}", rect);
                },
                Vertex::to_vertex,
            )
            .unwrap();
        match brush_action {
            glyph_brush::BrushAction::Draw(vertices) => {
                self.pipeline.draw(device, view, queue, vertices);             
            }
            glyph_brush::BrushAction::ReDraw => self.pipeline.redraw(device, view, queue),
        }
    }
}

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
        config: &wgpu::SurfaceConfiguration,
    ) -> GlyphBrush<F, H> {
        let inner = self.inner.build();
        let pipeline = Pipeline::new(device, config);
        GlyphBrush { inner, pipeline }
    }
}
