# wgpu-text
[![Licence](https://img.shields.io/github/license/Blatko1/wgpu-text?color=%23537aed)](https://github.com/Blatko1/wgpu-text/blob/master/LICENSE)
[![crates.io](https://img.shields.io/crates/v/wgpu-text)](https://crates.io/crates/wgpu_text)
[![Documentation](https://img.shields.io/docsrs/wgpu_text)](https://docs.rs/wgpu_text/0.1.1/wgpu_text/)

**wgpu-text** is a wrapper over **_[glyph-brush](https://github.com/alexheretic/glyph-brush)_** for simpler text rendering in **_[wgpu](https://github.com/gfx-rs/wgpu)_**.

This project was inspired by and is similar to **_[wgpu_glyph](https://github.com/hecrj/wgpu_glyph)_**, but has additional features and is simpler. Also there is no need to include **glyph-brush** in your project.

Some features are directly implemented from **glyph-brush** so you should go trough [Section docs](https://docs.rs/glyph_brush/latest/glyph_brush/struct.Section.html) for better understanding of adding and managing text.

Example:
```rust
use wgpu_text::BrushBuilder;
use wgpu_text::section::{Section, Text, Layout, HorizontalAlign};
let brush = BrushBuilder::using_font_bytes(font).unwrap().build(
        &device, format, width, height);
let section = Section::default()
    .add_text(Text::new("Hello World"))
    .with_layout(Layout::default().h_align(HorizontalAlign::Center));

// window event loop:
    winit::event::Event::RedrawRequested(_) => {
        // Has to be queued every frame.
        brush.queue(&section);
        let cmd_buffer = brush.draw_queued(&device, &view, &queue);
        // Has to be submitted last so text won't be overlapped.
        queue.submit(cmd_buffer);
        frame.submit();
    }
```
### **Examples**
Look trough [examples](https://github.com/Blatko1/wgpu_text/tree/master/examples) for more.
* `cargo run --example <example-name>`

#### **Goals**
- improve docs
- improve examples
- maybe some new features