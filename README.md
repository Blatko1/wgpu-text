# wgpu-text
[![Build Status](https://img.shields.io/github/workflow/status/Blatko1/wgpu-text/Rust?logo=github)](https://github.com/Blatko1/wgpu-text/actions)
[![Licence](https://img.shields.io/github/license/Blatko1/wgpu-text?color=%23537aed)](https://github.com/Blatko1/wgpu-text/blob/master/LICENSE)
[![crates.io](https://img.shields.io/crates/v/wgpu_text?logo=rust&logoColor=%23bf7d36)](https://crates.io/crates/wgpu_text)
[![Documentation](https://img.shields.io/docsrs/wgpu_text)](https://docs.rs/wgpu_text)

**wgpu-text** is a wrapper over **_[glyph-brush](https://github.com/alexheretic/glyph-brush)_** for **fast** and **easy** text rendering in **_[wgpu](https://github.com/gfx-rs/wgpu)_**.

This project was inspired by and is similar to **_[wgpu_glyph](https://github.com/hecrj/wgpu_glyph)_**, but has additional features and is simpler. Also there is no need to include **glyph-brush** in your project.

Some features are directly implemented from **glyph-brush** so you should go trough [Section docs](https://docs.rs/glyph_brush/latest/glyph_brush/struct.Section.html) and [Section examples](https://github.com/alexheretic/glyph-brush/tree/master/gfx-glyph/examples) for better understanding of managing and adding text.

## **Installation**
Add the following to your `Cargo.toml` file:

```toml
[dependencies]
wgpu_text = "0.6.1"
```

## **Usage**

```rust
use wgpu_text::BrushBuilder;
use wgpu_text::section::{Section, Text, Layout, HorizontalAlign};

let brush = BrushBuilder::using_font_bytes(font).unwrap()
 /* .initial_cache_size((1024, 1024))) */ // use this to avoid resizing cache texture
 /* .with_depth_testing(true) */ // enable/disable depth testing
    .build(&device, &config);
    
// Directly implemented from glyph_brush.
let section = Section::default()
    .add_text(Text::new("Hello World"))
    .with_layout(Layout::default().h_align(HorizontalAlign::Center));

// on window resize:
        brush.resize_view(config.width as f32, config.height as f32, &queue);

// window event loop:
    winit::event::Event::RedrawRequested(_) => {
        // Has to be queued every frame.
        brush.queue(&section);
        
        let text_buffer = brush.draw(&device, &view, &queue);
        
        // Has to be submitted last so text won't be overlapped.
        queue.submit([some_other_encoder.finish(), text_buffer]);
        
        frame.present();
    }
```

## **Examples**
For more detailed examples look trough [examples](https://github.com/Blatko1/wgpu_text/tree/master/examples).
* `cargo run --example <example-name>`

Run examples with `--release` for true performance.

## **Features**
Besides basic text rendering and **glyph-brush** features, there are some features that add customization:

> **builtin matrix** - default matrix for orthographic projection, feel free to use it for creating custom matrices

> **custom matrix** - ability of providing a custom matrix for purposes of custom view, rotation... (the downside is that it applies to all rendered text)

> **depth testing** - by adding z coordinate, text can be set on top or below other text (if enabled)

## **Goals**
- try to improve docs
- maybe some new features
- (wgpu stuff: maybe change to StagingBelt instead of Queue)

## **Contributing**
All contributions are welcome.
