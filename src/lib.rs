//! wgpu-text is a wrapper over [glyph-brush](https://github.com/alexheretic/glyph-brush)
//! for simpler text rendering in [wgpu](https://github.com/gfx-rs/wgpu).
//!
//! This project was inspired by and is similar to [wgpu_glyph](https://github.com/hecrj/wgpu_glyph),
//! but has additional features and is simpler. Also there is no need to
//! include glyph-brush in your project.
//!
//! Some features are directly implemented from glyph-brush so you should go trough
//! [Section docs](https://docs.rs/glyph_brush/latest/glyph_brush/struct.Section.html)
//! for better understanding of adding and managing text.
//!
//! If you want to learn about GPU texture caching see [caching behaviour](#caching-behaviour).
//!
//! * Look trough [examples](https://github.com/Blatko1/wgpu_text/tree/master/examples).

mod cache;
mod pipeline;

/// Contains all needed objects for inserting and styling text.
/// Directly taken from glyph_brush.
///
/// Look into [glyph_brush_layout docs](https://docs.rs/glyph_brush_layout/latest/glyph_brush_layout/#enums)
/// for the real, detailed, documentation.
/// - If anything is missing open an issue on github and I'll add it.
pub mod section {
    #[doc(hidden)]
    pub use glyph_brush::{
        BuiltInLineBreaker, Color, FontId, HorizontalAlign, Layout, LineBreak,
        OwnedSection, OwnedText, Section, SectionText, Text, VerticalAlign,
    };
}

/// Contains all needed objects for font managing.
/// Directly taken from glyph_brush.
///
/// Look into [glyph_brush_font docs](https://docs.rs/glyph_brush/latest/glyph_brush/ab_glyph/index.html)
/// for the real, detailed, documentation.
/// - If anything is missing open an issue on github and I'll add it.
pub mod font {
    #[doc(hidden)]
    pub use glyph_brush::ab_glyph::{Font, FontArc, FontRef, InvalidFont};
}

use glyph_brush::{
    ab_glyph::{Font, FontArc, FontRef, InvalidFont},
    BrushAction, DefaultSectionHasher, Extra, Section,
};
use pipeline::{Pipeline, Vertex};

/// Marks scissor region and tests itself automatically if it can fit inside
/// the surface `config` dimensions to avoid `wgpu` related rendering errors.
///
/// `out_width` and `out_height` are dimensions of the bigger rectangle (*window*)
/// in which the scissor region is located.
pub struct ScissorRegion {
    /// x coordinate of top left region point.
    pub x: u32,

    /// y coordinate of top left region point.
    pub y: u32,

    /// Width of scissor region.
    pub width: u32,

    /// Height of scissor region.
    pub height: u32,

    /// Width of outer rectangle.
    pub out_width: u32,

    /// Height of outer rectangle.
    pub out_height: u32,
}

/// Represents a two dimensional array matrix with 4x4 dimensions.
pub type Matrix = [[f32; 4]; 4];

impl ScissorRegion {
    /// Checks if the region is contained in surface bounds at all.
    pub(crate) fn is_contained(&self) -> bool {
        self.x < self.out_width && self.y < self.out_height
    }

    /// Gives available bounds paying attention to `out_width` and `out_height`.
    pub(crate) fn available_bounds(&self) -> (u32, u32) {
        let width = if (self.x + self.width) > self.out_width {
            self.out_width - self.x
        } else {
            self.width
        };

        let height = if (self.y + self.height) > self.out_height {
            self.out_height - self.y
        } else {
            self.height
        };

        (width, height)
    }
}

/// Wrapper over [`glyph_brush::GlyphBrush`]. Draws text.
///
/// Used for queuing and rendering text with
/// [`TextBrush::draw`] and [`TextBrush::draw_custom`].
pub struct TextBrush<F = FontArc, H = DefaultSectionHasher> {
    inner: glyph_brush::GlyphBrush<Vertex, Extra, F, H>,
    pipeline: Pipeline,
}

impl<F, H> TextBrush<F, H>
where
    F: Font + Sync,
    H: std::hash::BuildHasher,
{
    /// Queues section for drawing. This should be called
    /// every frame for every section that is going to be drawn.
    ///
    /// This can be called multiple times for different sections
    /// that want to use the same font and gpu cache.
    #[inline]
    pub fn queue<'a, S>(&mut self, section: S)
    where
        S: Into<std::borrow::Cow<'a, Section<'a>>>,
    {
        self.inner.queue(section);
    }

    fn draw_queued(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        region: Option<ScissorRegion>,
    ) -> wgpu::CommandBuffer {
        let mut brush_action;

        loop {
            brush_action = self.inner.process_queued(
                |rect, data| self.pipeline.update_texture(rect, data, queue),
                Vertex::to_vertex,
            );

            match brush_action {
                Ok(_) => break,

                Err(glyph_brush::BrushError::TextureTooSmall { suggested }) => {
                    if log::log_enabled!(log::Level::Warn) {
                        log::warn!(
                            "Resizing cache texture! This should be avoided \
                            by building TextBrush with BrushBuilder::initial_cache_size() \
                            and providing cache texture dimensions."
                        );
                    }
                    let max_image_dimension = device.limits().max_texture_dimension_2d;
                    let (width, height) = if (suggested.0 > max_image_dimension
                        || suggested.1 > max_image_dimension)
                        && (self.inner.texture_dimensions().0 < max_image_dimension
                            || self.inner.texture_dimensions().1 < max_image_dimension)
                    {
                        (max_image_dimension, max_image_dimension)
                    } else {
                        suggested
                    };
                    self.pipeline.resize_texture(device, (width, height));
                    self.inner.resize_texture(width, height);
                }
            }
        }

        match brush_action.unwrap() {
            BrushAction::Draw(vertices) => {
                self.pipeline.update_buffer(vertices, device, queue)
            }
            BrushAction::ReDraw => (),
        }

        let mut depth = None;
        if let Some(d) = &self.pipeline.depth_texture_view {
            depth = Some(wgpu::RenderPassDepthStencilAttachment {
                view: d,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            });
        }

        self.pipeline.draw(device, view, depth, region)
    }

    /// Draws all queued sections with [`queue`](#method.queue) function.
    ///
    /// Use [`TextBrush::draw_custom`] for more rendering options.
    #[inline]
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        queue: &wgpu::Queue,
    ) -> wgpu::CommandBuffer {
        self.draw_queued(device, view, queue, None)
    }

    /// Draws all queued text with extra options.
    ///
    /// # Scissoring
    /// With scissoring, you can filter out each glyph fragment that crosses the given `region`.
    #[inline]
    pub fn draw_custom<R>(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        region: Option<R>,
    ) -> wgpu::CommandBuffer
    where
        R: Into<ScissorRegion>,
    {
        self.draw_queued(device, view, queue, region.map(|r| r.into()))
    }

    /// Resizes "_camera_"(view). Updates default orthographic view matrix
    /// with given arguments and uses it for rendering.
    ///
    /// Run this function whenever the surface config is resized.
    /// _width_ and _height_ should be **surface** dimensions.
    ///
    /// **Matrix**:
    /// ```rust
    /// pub fn ortho(width: f32, height: f32) -> [[f32; 4]; 4] {
    ///     [
    ///         [2.0 / width, 0.0,          0.0, 0.0],
    ///         [0.0,        -2.0 / height, 0.0, 0.0],
    ///         [0.0,         0.0,          1.0, 0.0],
    ///         [-1.0,        1.0,          0.0, 1.0]
    ///     ]
    /// }
    /// ```
    #[inline]
    pub fn resize_view(&mut self, width: f32, height: f32, queue: &wgpu::Queue) {
        self.update_matrix(ortho(width, height), queue);
    }

    /// Resizes "_camera_"(view). Updates text rendering matrix with the provided one.
    ///
    /// Use [`Self::resize_view()`] to update render matrix with default orthographic matrix.
    ///
    /// Feel free to use [`ortho()`] for creating more complex matrices by yourself
    /// (`cross product`-ing).
    pub fn update_matrix<M>(&mut self, matrix: M, queue: &wgpu::Queue)
    where
        M: Into<Matrix>,
    {
        self.pipeline.update_matrix(matrix.into(), queue);
    }

    /// Resizes depth texture to provided dimensions.
    ///
    /// Required if [`BrushBuilder::with_depth_testing()`] is set to `true`.
    ///
    /// Should be used every time the window (`wgpu::SurfaceConfiguration`) is resized.
    /// If not used when needed, program will crash with _wgpu error_.
    ///
    /// If used while [`BrushBuilder::with_depth_testing()`] is set to `false`
    /// nothing will happen.
    #[inline]
    pub fn resize_depth(&mut self, width: u32, height: u32, device: &wgpu::Device) {
        if self.pipeline.depth_texture_view.is_some() {
            self.pipeline.update_depth(device, (width, height));
        }
    }
}

/// Builder for [`TextBrush`].
pub struct BrushBuilder<F, H = DefaultSectionHasher> {
    inner: glyph_brush::GlyphBrushBuilder<F, H>,
    depth_testing: Option<wgpu::DepthStencilState>,
    matrix: Option<Matrix>,
}

impl BrushBuilder<()> {
    /// Creates a [`BrushBuilder`] with [`Font`].
    #[inline]
    pub fn using_font<F: Font>(font: F) -> BrushBuilder<F> {
        BrushBuilder::using_fonts(vec![font])
    }

    /// Creates a [`BrushBuilder`] with font byte data.
    #[inline]
    pub fn using_font_bytes(data: &[u8]) -> Result<BrushBuilder<FontRef>, InvalidFont> {
        let font = FontRef::try_from_slice(data)?;
        Ok(BrushBuilder::using_fonts(vec![font]))
    }

    /// Creates a [`BrushBuilder`] with multiple fonts byte data.
    #[inline]
    pub fn using_font_bytes_vec(
        data: &[u8],
    ) -> Result<BrushBuilder<FontRef>, InvalidFont> {
        let font = FontRef::try_from_slice(data)?;
        Ok(BrushBuilder::using_fonts(vec![font]))
    }

    /// Creates a [`BrushBuilder`] with multiple [`Font`].
    pub fn using_fonts<F: Font>(fonts: Vec<F>) -> BrushBuilder<F> {
        BrushBuilder {
            inner: glyph_brush::GlyphBrushBuilder::using_fonts(fonts),
            depth_testing: None,
            matrix: None,
        }
    }
}

impl<F, H> BrushBuilder<F, H>
where
    F: Font,
    H: std::hash::BuildHasher,
{
    glyph_brush::delegate_glyph_brush_builder_fns!(inner);

    /// Uses the provided `matrix` when rendering.
    ///
    /// To update the render matrix use [`TextBrush::update_matrix()`].
    pub fn with_matrix<M>(mut self, matrix: M) -> Self
    where
        M: Into<Matrix>,
    {
        self.matrix = Some(matrix.into());
        self
    }

    /// Defaults to `false`. If set to true all text will be depth tested.
    ///
    /// For each section, depth can be set by modifying
    /// the z coordinate ([`OwnedText::with_z()`]).
    ///
    /// `z` coordinate should be in range
    ///  0.0 - 1.0 not including 1.0.
    pub fn with_depth_testing(mut self, test: bool) -> BrushBuilder<F, H> {
        if test {
            self.depth_testing = Some(wgpu::DepthStencilState {
                format: Pipeline::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            });
        } else {
            self.depth_testing = None;
        }
        self
    }

    /// Builds a [`TextBrush`] with depth testing consuming [`BrushBuilder`].
    ///
    /// To use this build method call [`Self::with_depth_testing()`].
    pub fn build(
        self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> TextBrush<F, H> {
        let inner = self.inner.build();

        let matrix = self
            .matrix
            .unwrap_or(ortho(config.width as f32, config.height as f32));

        let depth = self.depth_testing.is_some();
        let mut pipeline = Pipeline::new(
            device,
            config.format,
            self.depth_testing,
            inner.texture_dimensions(),
            matrix,
        );
        if depth {
            pipeline.update_depth(device, (config.width, config.height));
        }

        TextBrush { inner, pipeline }
    }
}

/// Creates an orthographic matrix with given dimensions `width` and `height`.
#[rustfmt::skip]
pub fn ortho(width: f32, height: f32) -> Matrix {
    [
        [2.0 / width, 0.0,          0.0, 0.0],
        [0.0,        -2.0 / height, 0.0, 0.0],
        [0.0,         0.0,          1.0, 0.0],
        [-1.0,        1.0,          0.0, 1.0]
    ]
}
