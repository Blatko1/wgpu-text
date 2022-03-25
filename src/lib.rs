//! wgpu-text is a wrapper over [glyph-brush](https://github.com/alexheretic/glyph-brush) for simpler text rendering in [wgpu](https://github.com/gfx-rs/wgpu).
//!
//! This project was inspired by and is similar to [wgpu_glyph](https://github.com/hecrj/wgpu_glyph), but has additional features and is simpler. Also there is no need to include glyph-brush in your project.
//!
//! Some features are directly implemented from glyph-brush so you should go trough [Section docs](https://docs.rs/glyph_brush/latest/glyph_brush/struct.Section.html) for better understanding of adding and managing text.
//!
//! * Look trough [examples](https://github.com/Blatko1/wgpu_text/tree/master/examples).

mod brush;
mod cache;
mod pipeline;

pub use brush::{BrushBuilder, TextBrush};

/// Contains all needed structs and enums for inserting and styling text. Directly taken from glyph_brush.
pub mod section {
    pub use glyph_brush::{
        BuiltInLineBreaker, Color, HorizontalAlign, Layout, OwnedSection, OwnedText, Section, Text,
        VerticalAlign,
    };
}
