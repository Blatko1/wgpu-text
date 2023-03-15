# Changelog

## Notable changes from **0.6.6v** to **0.7.0v**

### Breaking changes

#### Behavoiur

Calling any of the **depth functions** while **depth** is disabled will `panic!` and crash the program. This will make sure you are not using any depth function by mistake.

#### Removed and renamed

- Removed `draw()`, `draw_custom()` and inner function `draw_queued()` from the `TextBrush` struct.

- Removed `with_depth_testing()` from the `BrushBuilder`.

- Renamed `resize_depth()` to `resize_depth_view()` from the `TextBrush` struct.

#### Replacement additions

- Added `draw()` - draws all queued text.

- Added `draw_with_depth()` - draws all queued text with depth testing. If depth isn't enabled this function will `panic!` and crash the program.

- Added `with_depth()` - enables **depth testing** if called while creating the `BrushBuilder`

- Added `set_region()` function to `TextBrush` - sets a scissor region which filters out each glyph fragment that crosses the given *bounds*.

### New Features

#### Error type

Added a single error type `BrushError::TooBigCacheTexture(u32)` which is used when glyphs are too big to fit the `cache_texture` but the texture can't increase in size because of `wgpu::Limits`.

#### New **set_load_op()** function

Added `set_load_op()` function to `TextBrush` through which you can determine what operation to perform to the output attachment (*texture view*) at the start of a *render pass*.

#### New **build_custom()** function

Added a new `build_custom()` function providing better support when drawing text to texture views different from the `current_frame_texture` view. You can find an example using this feature in the *examples* folder called *custom_surface*.

#### Examples

- Created a new file `utils.rs` (non-example) in *examples* folder for easier access to the *wgpu tools*.

- Added a new `custom_surface` example which shows how to render text onto *textures* which are then drawn onto a quad detached from the UI.

### Minor changes

- Reduced the amount of arguments for some functions.
- Improved and added more docs.
- Updated crates.
- Fixed *clippy* warnings.
- Reduced code density in `lib.rs` by distributing some to `brush.rs`