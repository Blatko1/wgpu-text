# wgpu-text

[![Build Status](https://img.shields.io/github/actions/workflow/status/blatko1/wgpu-text/rust.yml?logo=github)](https://github.com/Blatko1/wgpu-text/actions)
[![Licence](https://img.shields.io/github/license/Blatko1/wgpu-text?color=%23537aed)](https://github.com/Blatko1/wgpu-text/blob/master/LICENSE)
[![crates.io](https://img.shields.io/crates/v/wgpu_text?logo=rust&logoColor=%23bf7d36)](https://crates.io/crates/wgpu_text)
[![Documentation](https://img.shields.io/docsrs/wgpu_text)](https://docs.rs/wgpu_text)

**`wgpu-text`** is a wrapper over **[`glyph-brush`](https://github.com/alexheretic/glyph-brush)** for **fast** and **easy** text rendering in **_[`wgpu`](https://github.com/gfx-rs/wgpu)_**. It accepts **.otf** and **.ttf** fonts.

<img src="showcase.png" align="right" style="padding: 0px 0px 5px 4px; margin-top: -10px" alt="Library Lorem Ipsum Showcase" width="390px">

This project was inspired by and is similar to **[`wgpu_glyph`](https://github.com/hecrj/wgpu_glyph)** but has additional features and is more straightforward. Also, there is no need to include **glyph-brush** in your project.

Some features are directly implemented from **glyph-brush**, so it's recommended to go through [Section docs](https://docs.rs/glyph_brush/latest/glyph_brush/struct.Section.html) and [Section examples](https://github.com/alexheretic/glyph-brush/tree/master/gfx-glyph/examples) for better understanding of managing and adding text.

## **Installation**

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
wgpu_text = "0.7.1"
```

## **Usage**

```rust
use wgpu_text::section::{Section, Text, Layout, HorizontalAlign, BrushBuilder};

let brush = BrushBuilder::using_font_bytes(font).unwrap()
 /* .initial_cache_size((16_384, 16_384))) */ // use this to avoid resizing cache texture
 /* .with_depth() */ // enables depth testing
    .build(&device, &config);

// Directly implemented from glyph_brush.
let section = Section::default().add_text(Text::new("Hello World"));

// on window resize:
        brush.resize_view(config.width as f32, config.height as f32, &queue);

// window event loop:
    winit::event::Event::RedrawRequested(_) => {
        // Has to be queued every frame.
        brush.queue(&section);
        // Crashes if inner cache exceeds limits.
        brush.process_queued(&device, &queue).unwrap();

        let text_buffer = brush.draw(&device, &view);

        // Submit last to avoid overlaps.
        queue.submit([some_other_cmd_buffer, text_buffer]);

        frame.present();
    }
```

## **Examples**

For more detailed examples, look through [examples](https://github.com/Blatko1/wgpu_text/tree/master/examples).

* `cargo run --example <example-name>`

Run examples with `--release` for accurate performance.

## **Features**

Besides basic text rendering and **glyph-brush** features, some features add customization:

- **custom render-to view** - you can specify a custom texture view for it to render the text to instead of rendering it onto the frame view
- **builtin matrix** - default matrix for orthographic projection (feel free to use it for creating custom matrices)
- **custom matrix** - grants the ability to provide a custom matrix for purposes of custom view, rotation, etc. (the downside is that it applies to all rendered text)
- **depth testing** - by adding a *z* coordinate, text can be set on top or below other text (if enabled)

## **Future**

* wgpu stuff: maybe change to StagingBelt instead of Queue
* mip-mapping example if possible

## **Contributing**

All contributions are welcome.
