use std::{borrow::Cow, hash::BuildHasher};

use glyph_brush::{
    ab_glyph::{point, Font, FontArc, FontRef, InvalidFont, Rect},
    DefaultSectionHasher, Extra, Section,
};

#[repr(C)]
#[derive(Debug, Clone)]
struct Vertex {
    top_left: [f32; 3],
    bottom_right: [f32; 2],
    tex_top_left: [f32; 2],
    tex_bottom_right: [f32; 2],
    color: [f32; 4],
}

pub struct GlyphBrush<F = FontArc, H = DefaultSectionHasher> {
    inner: glyph_brush::GlyphBrush<Vertex, Extra, F, H>,
}

impl<F: Font + Sync, H: BuildHasher> GlyphBrush<F, H> {
    #[inline]
    pub fn queue<'a, S>(&mut self, section: S)
    where
        S: Into<Cow<'a, Section<'a>>>,
    {
        self.inner.queue(section);
    }

    pub fn draw_queued(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let brush_action = self.inner
            .process_queued(|rect, tex_data| {
                println!("rect: {:?}", rect);
            }, Vertex::to_vertex).unwrap();
        match brush_action {
            glyph_brush::BrushAction::Draw(vert) => for v in vert {
                println!("vert: {:?}", v);
            },
            glyph_brush::BrushAction::ReDraw => todo!(),
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

    pub fn build(self) -> GlyphBrush<F, H> {
        let inner = self.inner.build();
        GlyphBrush { inner }
    }
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
            top_left: [pixel_coords.min.x, pixel_coords.min.y, extra.z],
            bottom_right: [pixel_coords.max.x, pixel_coords.max.y],
            tex_top_left: [tex_coords.min.x, tex_coords.min.y],
            tex_bottom_right: [tex_coords.max.x, tex_coords.max.y],
            color: extra.color,
        }
    }
}
