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
//! To learn about GPU texture caching, see
//! [`caching behaviour`](https://docs.rs/glyph_brush/latest/glyph_brush/struct.GlyphBrush.html#caching-behaviour)
//!
//! > Look trough [`examples`](https://github.com/Blatko1/wgpu_text/tree/master/examples).

mod brush;
mod cache;
mod error;
mod pipeline;
// TODO remove anything about scissor regions and set_load_op
pub use brush::{BrushBuilder, TextBrush};

/// Contains all needed objects for inserting, styling and iterating text and glyphs.
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
        LineBreak, OwnedSection, OwnedText, Section, SectionGlyphIter, SectionText, Text,
        VerticalAlign,
    };
}

/// Contains all needed objects for font and glyph management.
/// Directly taken from glyph_brush.
///
/// Look into [`glyph_brush_font docs`](https://docs.rs/glyph_brush/latest/glyph_brush/ab_glyph/index.html)
/// for the accurate, detailed documentation.
///
/// If anything is missing, open an issue on GitHub, and I'll review it.
pub mod font {
    #[doc(hidden)]
    pub use glyph_brush::ab_glyph::{Font, FontArc, FontRef, InvalidFont, ScaleFont};
}

/// Represents a two-dimensional array matrix with 4x4 dimensions.
pub type Matrix = [[f32; 4]; 4];

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
