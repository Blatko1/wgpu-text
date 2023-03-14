use std::{error::Error, fmt::Display};

// TODO docs for each type
#[derive(Debug)]
pub enum BrushError {
    DepthDisabled,
    TooBigCacheTexture(u32),
}

impl Error for BrushError {}

// TODO maybe add tabs before each row when printing.
impl Display for BrushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wgpu-text: ")?;
        match self {
            BrushError::DepthDisabled => write!(
                f,
                "A 'draw_with_depth()' function as been called but 'depth' isn't \
                enabled! Add 'with_depth()' while building the TextBrush."
            ),
            BrushError::TooBigCacheTexture(dimensions) => write!(
                f,
                "{}",
                format!(
                    "While trying to resize the \
                cache texture, the 'wgpu::Limits {{ max_texture_dimension_2d }}' \
                limit of {} was crossed!\n\
                Resizing the cache texture should be avoided \
                from the start by building TextBrush with \
                BrushBuilder::initial_cache_size() and providing bigger cache \
                texture dimensions.",
                    dimensions
                )
            ),
        }
    }
}
