//! wgpu-text is a wrapper over [glyph-brush](https://github.com/alexheretic/glyph-brush) for simpler text rendering in [wgpu](https://github.com/gfx-rs/wgpu).
//! 
//! This project was inspired by and is similar to [wgpu_glyph](https://github.com/hecrj/wgpu_glyph), but has additional features and is simpler. Also there is no need to include glyph-brush in your project.
//! 
//! Some features are directly implemented from glyph-brush so you should go trough [Section docs](https://docs.rs/glyph_brush/latest/glyph_brush/struct.Section.html) for better understanding of adding and managing text.
//! 
//! Example:
//! ```rust
//! use wgpu_text::BrushBuilder;
//! use wgpu_text::section::{Section, Text, Layout, HorizontalAlign};
//! let brush = BrushBuilder::using_font_bytes(font).unwrap().build(
//!         &device, format, width, height);
//! let section = Section::default()
//!     .add_text(Text::new("Hello World"))
//!     .with_layout(Layout::default().h_align(HorizontalAlign::Center));
//! 
//! // window event loop:
//!     winit::event::Event::RedrawRequested(_) => {
//!         // Has to be queued every frame.
//!         brush.queue(&section);
//!         let cmd_buffer = brush.draw_queued(&device, &view, &queue);
//!         // Has to be submitted last so text won't be overlapped.
//!         queue.submit(cmd_buffer);
//!         frame.submit();
//!     }
//! ```
//! 
//! * Look trough [examples](https://github.com/Blatko1/wgpu_text/tree/master/examples) for more.

mod brush;
mod pipeline;
mod uniform;

pub use brush::{BrushBuilder, TextBrush};

/// Contains all needed structs and enums for inserting and styling text. Directly taken from glyph_brush.
pub mod section {
    pub use glyph_brush::{
        BuiltInLineBreaker, Color, HorizontalAlign, Layout, OwnedSection, OwnedText, Section, Text,
        VerticalAlign,
    };
}
