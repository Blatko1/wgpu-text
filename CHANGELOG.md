# Changelog

## v0.9.2

- `wgpu` crate version -> 24.0.0
- `log` crate version -> 0.4.25
- `bytemuck` crate version -> 1.21.0

## v0.9.1

- `wgpu` crate version -> 23.0.0
- `glyph_brush` crate version -> 0.7.11
- `bytemuck` crate version -> 1.19.0

## v0.9.0

- `TextBrush::queue()` function can now take any item which implements IntoIterator trait (i.e. static arrays)
- `wgpu` crate version -> 22.0.0 - by @VladasZ in [#38](https://github.com/Blatko1/wgpu-text/pull/38)
- `glyph_brush` crate version -> 0.7.9
- `bytemuck` crate version -> 1.16.1
- `log` crate version -> 0.4.22

## v0.8.8

- `wgpu` crate version -> 0.20.0

## v0.8.7

- `wgpu` crate version -> 0.19.3
- `log` crate version -> 0.4.21
- `bytemuck` crate version -> 1.15.0

## v0.8.6

- `wgpu` crate version -> 0.19.0
- `winit` crate version in examples -> 0.29.10
- `BrushError` now derives `Clone`, `Copy`, `PartialEq` and `Eq`

## v0.8.5

- `wgpu` crate version -> v0.18.0
- changed example code according to the `wgpu` updates

## v0.8.4

- small performance improvement by adding `min_binding_size` to the matrix BindGroup
- `wgpu` crate version -> v0.17.1
- other crates versions updates

## v0.8.3

- `wgpu` crate version -> v0.17.0

- `draw()` function no longer needs `self` to be mutable - by @PPakalns in [#18](https://github.com/Blatko1/wgpu-text/pull/18).

- removed redundant `mut` from functions `update_matrix()` and `resize_view()`
  
- `BrushError` is now public

## v0.8.2

### Minor changes

- reexported `glyph_brush` as a whole

## v0.8.1

### New functions

Added a new function `with_multisample()` in `BrushBuilder` which specifies the `wgpu::MultisampleState` for the inner pipeline - by @AndriBaal in [#12](https://github.com/Blatko1/wgpu-text/pull/12).

Added a new function `with_multiview()` in `BrushBuilder` which specifies the `multiview` attribute used by the inner pipeline.

### Minor changes

- changed some inner (private) variable names

## v0.8.0

### Breaking changes

Removed `ScissorRegion` since there are hardly any valuable cases for its use, and there are better alternatives like setting the `bounds` parameter of `Section`.

Amplified the *draw* functions according to the *wgpu repository wiki*. Now they need fewer arguments and have better performances since they use the borrowed `render_pass` instead of creating a new one.

After amplifying the *draw* functions, there is no need for the `set_load_op()` function.

Removed *depth functions* (`resize_depth_view`, `with_depth`) since now you have to add your depth stencil and texture to the render pass that is being borrowed to the *draw* functions. The only new function left is `with_depth_stencil()` in the `BrushBuilder`, with which you can add a *depth stencil* to the *inner pipeline*.

Removed old `queue()` and `process_queued()` functions. Added a new `queue()` function, representing both removed functions in one. It now takes a list of `Section`s. If depth is being utilized, you should pay attention to how you order `Section`s (order: furthest to closest). Each will be drawn in the order they are given.

### Minor changes

- renamed example `custom_surface` to `custom_output`
- slight modifications to some examples
- removed example `scissoring.rs`
- renamed the shader to `shader.wgsl`


## Notable changes from **v0.6.6** to **v0.7.0**

### Breaking changes

#### Behaviour

Calling any of the **depth functions** while **depth** is disabled will `panic!` and crash the program. This will make sure you are using the *depth* function correctly.

#### Removed and renamed

- Removed `draw()`, `draw_custom()` and inner function `draw_queued()` from the `TextBrush` struct.

- Removed `with_depth_testing()` from the `BrushBuilder`.

- Renamed `resize_depth()` to `resize_depth_view()` from the `TextBrush` struct.

#### Replacement additions

- Added `draw()` function to `TextBrush` - draws all queued text.

- Added `draw_with_depth()` function to `TextBrush` - draws all queued text with *depth* testing. This function will `panic!` and crash the program if *depth* isn't enabled.

- Added `with_depth()` function to `BrushBuilder` - enables **depth testing** if called while creating the `BrushBuilder`

- Added `set_region()` function to `TextBrush` - sets a scissor region which filters out each glyph fragment that crosses the given *bounds*.

- Added `process_queued()` function to `TextBrush` - processes all queued sections and updates the inner vertex buffer. Returns an Error if cache texture is too big. **Required** if you want to draw anything. 

### New Features

#### Error type

Added a single error type `BrushError::TooBigCacheTexture(u32)`, which is used when glyphs are too big to fit the `cache_texture` but the texture can't increase in size because of `wgpu::Limits`.

#### New **glyphs_iter()** and **fonts()** functions

Added a `glyphs_iter()` function, which returns an iterator over glyphs in the provided section.
Added `fonts()` function, which returns an array of all available fonts. You can then perform various glyph functions such as finding glyphs bounding box.

#### New **set_load_op()** function

Added `set_load_op()` function to `TextBrush` to determine what operation to perform to the output attachment (*texture view*) at the start of a *render pass*.

#### New **build_custom()** function

Added a new `build_custom()` function providing better support when drawing text to texture views different from the `current_frame_texture` view. You can find an example using this feature in the *examples* folder called *custom_surface*.

#### Examples

- Created a new file, `utils.rs` (non-example), in the *examples* folder for easier access to the *wgpu tools*.

- Added a new `custom_surface` example which shows how to render text onto *textures*, which are then drawn onto a quad detached from the UI.

### Minor changes

- reduced the number of arguments for some functions.
- improved and added more docs.
- updated crates.
- fixed *clippy* warnings.
- reduced code density in `lib.rs` by distributing some to `brush.rs`
- added additional `FontArc` implementation from **ab_glyph**