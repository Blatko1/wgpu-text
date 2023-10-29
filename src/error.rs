use std::{error::Error, fmt::Display};

/// Result of `TextBrush` errors and problems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushError {
    /// Cache texture exceeded the limitations stated in `wgpu::Device`.
    TooBigCacheTexture(u32),
}

impl Error for BrushError {}

impl Display for BrushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wgpu-text: ")?;
        match self {
            BrushError::TooBigCacheTexture(dimensions) => write!(
                f,
                "While trying to resize the \
                cache texture, the 'wgpu::Limits {{ max_texture_dimension_2d }}' \
                limit of {} was crossed!\n\
                Resizing the cache texture should be avoided \
                from the start by building TextBrush with \
                BrushBuilder::initial_cache_size() and providing bigger cache \
                texture dimensions.",
                dimensions
            ),
        }
    }
}
