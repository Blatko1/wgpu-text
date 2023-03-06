//! **wgpu-text** is a wrapper over **[glyph-brush](https://github.com/alexheretic/glyph-brush)**
//! for simpler text rendering in **[wgpu](https://github.com/gfx-rs/wgpu)**.
//!
//! This project was inspired by and is similar to [wgpu_glyph](https://github.com/hecrj/wgpu_glyph),
//! but has additional features and is more straightforward. Also, there is no need to
//! include **glyph-brush** in your project.
//!
//! Some features are directly implemented from glyph-brush, so it's recommended to go through
//! [Section docs](https://docs.rs/glyph_brush/latest/glyph_brush/struct.Section.html) and
//! [Section examples](https://github.com/alexheretic/glyph-brush/tree/master/gfx-glyph/examples)
//! for a better understanding of adding and managing text.
//!
//! > Look trough [`examples`](https://github.com/Blatko1/wgpu_text/tree/master/examples).

mod brush;
mod cache;
mod pipeline;

pub use brush::{BrushBuilder, TextBrush};

/// Contains all needed objects for inserting and styling text.
/// Directly taken from glyph_brush.
///
/// Look into [`glyph_brush_layout docs`](https://docs.rs/glyph_brush_layout/latest/glyph_brush_layout/#enums)
/// for the accurate, detailed documentation.
///
/// If anything is missing, open an issue on GitHub, and I'll review it.
pub mod section {
    #[doc(hidden)]
    pub use glyph_brush::{
        BuiltInLineBreaker, Color, FontId, GlyphCruncher, HorizontalAlign, Layout,
        LineBreak, OwnedSection, OwnedText, Section, SectionText, Text, VerticalAlign,
    };
}

/// Contains all needed objects for font management.
/// Directly taken from glyph_brush.
///
/// Look into [`glyph_brush_font docs`](https://docs.rs/glyph_brush/latest/glyph_brush/ab_glyph/index.html)
/// for the accurate, detailed documentation.
///
/// If anything is missing, open an issue on GitHub, and I'll review it.
pub mod font {
    #[doc(hidden)]
    pub use glyph_brush::ab_glyph::{Font, FontArc, FontRef, InvalidFont};
}

/// Marks scissor region and tests itself automatically if it can fit inside
/// the surface `config` dimensions to avoid `wgpu` related rendering errors.
///
/// `out_width` and `out_height` are dimensions of the bigger rectangle
/// (*window*, usually *surface config* dimensions)
/// in which the scissor region is located.
pub struct ScissorRegion {
    /// x coordinate of top left region point.
    pub x: u32,

    /// y coordinate of top left region point.
    pub y: u32,

    /// Width of scissor region.
    pub width: u32,

    /// Height of scissor region.
    pub height: u32,

    /// Width of outer rectangle.
    pub out_width: u32,

    /// Height of outer rectangle.
    pub out_height: u32,
}

/// Represents a two-dimensional array matrix with 4x4 dimensions.
pub type Matrix = [[f32; 4]; 4];

impl ScissorRegion {
    /// Checks if the region is contained in surface bounds at all.
    pub(crate) fn is_contained(&self) -> bool {
        self.x < self.out_width && self.y < self.out_height
    }

    /// Gives available bounds paying attention to `out_width` and `out_height`.
    pub(crate) fn available_bounds(&self) -> (u32, u32) {
        let width = if (self.x + self.width) > self.out_width {
            self.out_width - self.x
        } else {
            self.width
        };

        let height = if (self.y + self.height) > self.out_height {
            self.out_height - self.y
        } else {
            self.height
        };

        (width, height)
    }
}

/// Creates an orthographic matrix with given dimensions `width` and `height`.
#[rustfmt::skip]
pub fn ortho(width: f32, height: f32) -> Matrix {
    [
        [2.0 / width, 0.0,          0.0, 0.0],
        [0.0,        -2.0 / height, 0.0, 0.0],
        [0.0,         0.0,          1.0, 0.0],
        [-1.0,        1.0,          0.0, 1.0]
    ]
}
