//! **wgpu-text** is a wrapper over **[glyph-brush](https://github.com/alexheretic/glyph-brush)**
//! for simpler text rendering in **[wgpu](https://github.com/gfx-rs/wgpu)**.
//!
//! This project was inspired by and is similar to [wgpu_glyph](https://github.com/hecrj/wgpu_glyph),
//! but has additional features and is more straightforward. Also, there is no need to
//! include **glyph-brush** in your project.
//!
//! Since the crate **glyph-brush** is reexported and heavily dependent on, it's recommended to go through
//! [Section docs](https://docs.rs/glyph_brush/latest/glyph_brush/struct.Section.html) and
//! [Section examples](https://github.com/alexheretic/glyph-brush/tree/master/gfx-glyph/examples)
//! for a better understanding of adding and managing text.
//!
//! To learn about GPU texture caching, see
//! [`caching behaviour`](https://docs.rs/glyph_brush/latest/glyph_brush/struct.GlyphBrush.html#caching-behaviour)
//!
//! > Look trough [`examples`](https://github.com/Blatko1/wgpu_text/tree/master/examples).

// TODO fix VULKAN error when running examples
mod brush;
mod cache;
mod error;
mod pipeline;

pub use brush::{BrushBuilder, TextBrush};
pub use error::BrushError;
pub use glyph_brush;

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
