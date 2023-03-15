# Changelog

## Notable changes from **0.6.6v** to **0.7.0v**

### Breaking changes

Renamed 

### New Features

#### Examples

- Created a new file `utils.rs` (non-example) in *examples* folder for easier access to *wgpu tools*.
- Added a new `custom_surface` example which shows how to render text onto *textures* which are then drawn onto a quad detached from the UI.

#### New **build_custom()** function

Added a new `build_custom()` function providing better support when drawing text to texture views different from the `current_frame_texture` view. You can find an example using this feature in the *examples* folder called *custom_surface*.

### Minor changes

- Improved docs.
- Updated crates.
- Fixed *clippy* warnings.
- Reduced code density in `lib.rs` by distributing some to `brush.rs`