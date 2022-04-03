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

/// Contains all needed structs and enums for inserting and styling text.
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

use glyph_brush::GlyphBrush;
pub use glyph_brush::{
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

impl ScissorRegion {
    /// Checks if the region is contained in surface bounds at all.
    pub(crate) fn is_contained(&self) -> bool {
        if self.x < self.out_width && self.y < self.out_height {
            return true;
        }
        false
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
pub struct TextBrush<Depth, F = FontArc, H = DefaultSectionHasher> {
    inner: GlyphBrush<Vertex, Extra, F, H>,
    pipeline: Pipeline<Depth>,
}

impl<Depth, F, H> TextBrush<Depth, F, H>
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
        M: Into<[[f32; 4]; 4]>,
    {
        self.pipeline.update_matrix(matrix.into(), queue);
    }
}

impl<F, H> TextBrush<wgpu::DepthStencilState, F, H>
where
    F: Font + Sync,
    H: std::hash::BuildHasher,
{
    /// Available if *TextBrush* was built with [`BrushBuilder::with_depth_testing()`].
    /// Resizes depth texture to provided dimensions.
    ///
    /// Should be used every time the window (`wgpu::SurfaceConfiguration`) is resized.
    /// If not used when needed, your program will crash with wgpu error.
    #[inline]
    pub fn resize_depth(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.pipeline.update_depth(device, (width, height));
    }
}

/// Builder for [`TextBrush`].
pub struct BrushBuilder<Depth, F, H = DefaultSectionHasher> {
    inner: glyph_brush::GlyphBrushBuilder<F, H>,
    matrix: Option<[[f32; 4]; 4]>,
    phantom: std::marker::PhantomData<Depth>,
}

impl BrushBuilder<(), ()> {
    /// Creates a [`BrushBuilder`] with [`Font`].
    #[inline]
    pub fn using_font<F: Font>(font: F) -> BrushBuilder<(), F> {
        BrushBuilder::using_fonts(vec![font])
    }

    /// Creates a [`BrushBuilder`] with font byte data.
    #[inline]
    pub fn using_font_bytes(
        data: &[u8],
    ) -> Result<BrushBuilder<(), FontRef>, InvalidFont> {
        let font = FontRef::try_from_slice(data)?;
        Ok(BrushBuilder::using_fonts(vec![font]))
    }

    /// Creates a [`BrushBuilder`] with multiple fonts byte data.
    #[inline]
    pub fn using_font_bytes_vec(
        data: &[u8],
    ) -> Result<BrushBuilder<(), FontRef>, InvalidFont> {
        let font = FontRef::try_from_slice(data)?;
        Ok(BrushBuilder::using_fonts(vec![font]))
    }

    /// Creates a [`BrushBuilder`] with multiple [`Font`].
    pub fn using_fonts<F: Font>(fonts: Vec<F>) -> BrushBuilder<(), F> {
        BrushBuilder {
            inner: glyph_brush::GlyphBrushBuilder::using_fonts(fonts),
            matrix: None,
            phantom: std::marker::PhantomData::<()>,
        }
    }
}

impl<Depth, F, H> BrushBuilder<Depth, F, H>
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
        M: Into<[[f32; 4]; 4]>,
    {
        self.matrix = Some(matrix.into());
        self
    }
}

impl<F, H> BrushBuilder<(), F, H>
where
    F: Font,
    H: std::hash::BuildHasher,
{
    /// If called depth testing will be enabled. By default depth testing
    /// is disabled.
    ///
    /// For each section, depth can be set by modifying
    /// the z coordinate ([`OwnedText::with_z()`]).
    ///
    /// `z` coordinate should be in range
    ///  0.0 - 1.0 not including 1.0.
    pub fn with_depth_testing(self) -> BrushBuilder<wgpu::DepthStencilState, F, H> {
        BrushBuilder {
            inner: self.inner,
            matrix: self.matrix,
            phantom: std::marker::PhantomData::<wgpu::DepthStencilState>,
        }
    }

    /// Builds a [`TextBrush`] consuming [`BrushBuilder`].
    pub fn build(
        self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> TextBrush<(), F, H> {
        let inner: GlyphBrush<Vertex, Extra, F, H> = self.inner.build();

        let matrix = create_matrix(self.matrix, &config);

        let pipeline = Pipeline::new(
            device,
            config.format.clone(),
            None,
            inner.texture_dimensions(),
            matrix,
        );

        TextBrush { inner, pipeline }
    }
}

impl<F, H> BrushBuilder<wgpu::DepthStencilState, F, H>
where
    F: Font,
    H: std::hash::BuildHasher,
{
    /// Builds a [`TextBrush`] with depth testing consuming [`BrushBuilder`].
    ///
    /// To use this build method call [`Self::with_depth_testing()`].
    pub fn build(
        self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> TextBrush<wgpu::DepthStencilState, F, H> {
        let inner = self.inner.build();

        let matrix = create_matrix(self.matrix, &config);

        let depth = wgpu::DepthStencilState {
            format: Pipeline::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let mut pipeline = Pipeline::new(
            device,
            config.format.clone(),
            Some(depth),
            inner.texture_dimensions(),
            matrix,
        )
        .with_depth();
        pipeline.update_depth(device, (config.width, config.height));

        TextBrush { inner, pipeline }
    }
}

fn create_matrix(
    matrix: Option<[[f32; 4]; 4]>,
    config: &wgpu::SurfaceConfiguration,
) -> [[f32; 4]; 4] {
    if let Some(matrix) = matrix {
        matrix
    } else {
        ortho(config.width as f32, config.height as f32)
    }
}

/// Creates an orthographic matrix with given dimensions `width` and `height`.
#[rustfmt::skip]
pub fn ortho(width: f32, height: f32) -> [[f32; 4]; 4] {
    [
        [2.0 / width, 0.0,          0.0, 0.0],
        [0.0,        -2.0 / height, 0.0, 0.0],
        [0.0,         0.0,          1.0, 0.0],
        [-1.0,        1.0,          0.0, 1.0]
    ]
}
