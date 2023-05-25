use crate::{
    error::BrushError,
    pipeline::{Pipeline, Vertex},
    Matrix,
};
use glyph_brush::{
    ab_glyph::{Font, FontArc, FontRef, InvalidFont, Rect},
    BrushAction, DefaultSectionHasher, Extra, GlyphCruncher, Section, SectionGlyphIter,
};

/// Wrapper over [`glyph_brush::GlyphBrush`]. In charge of drawing text.
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
    // TODO docs if using depth.
    /// Queues section for drawing. This method should be called every frame for
    /// each section that is going to be drawn.
    ///
    /// This can be called multiple times for different sections that want to use
    /// the same font and gpu cache.
    ///
    /// To learn about GPU texture caching, see
    /// [`caching behaviour`](https://docs.rs/glyph_brush/latest/glyph_brush/struct.GlyphBrush.html#caching-behaviour)
    #[inline]
    pub fn queue<'a, S>(&mut self, section: S)
    where
        S: Into<std::borrow::Cow<'a, Section<'a>>>,
    {
        self.inner.queue(section)
    }
    // TODO maybe take in all sections in one
    // function and process them at the same time

    /// Returns a bounding box for the section glyphs calculated using each
    /// glyph's vertical & horizontal metrics. For more info, read about
    /// [`GlyphCruncher::glyph_bounds`].
    #[inline]
    pub fn glyph_bounds<'a, S>(&mut self, section: S) -> Option<Rect>
    where
        S: Into<std::borrow::Cow<'a, Section<'a>>>,
    {
        self.inner.glyph_bounds(section)
    }

    /// Returns an iterator over the `PositionedGlyph`s of the given section.
    #[inline]
    pub fn glyphs_iter<'a, 'b, S>(&'b mut self, section: S) -> SectionGlyphIter<'b>
    where
        S: Into<std::borrow::Cow<'a, Section<'a>>>,
    {
        self.inner.glyphs(section)
    }

    /// Returns the available fonts.
    ///
    /// The `FontId` corresponds to the index of the font data.
    pub fn fonts(&self) -> &[F] {
        self.inner.fonts()
    }

    /// Draws all sections queued with [`queue`](#method.queue) function.
    ///
    /// **After queueing sections make sure to call [`TextBrush::process_queued()`]
    /// to update the inner vertex buffer and catch possible errors.**
    ///
    /// You can specify where to draw the text when providing the `view`.
    /// For example, instead of giving the current `frame texture view`
    /// and drawing to it, you can provide different texture view and
    /// draw the text there.
    #[inline]
    pub fn draw<'pass>(&'pass mut self, rpass: &mut wgpu::RenderPass<'pass>) {
        self.pipeline.draw(rpass)
    }

    /// Processes all queued text and updates the vertex buffer, unless the text vertices
    /// remain unmodified when compared to the last frame.
    ///
    /// If not called when required, the draw functions will continue drawing data from the
    /// inner vertex buffer meaning they will redraw old vertices.
    #[inline]
    pub fn process_queued(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), BrushError> {
        loop {
            let brush_action = self.inner.process_queued(
                |rect, data| self.pipeline.update_texture(rect, data, queue),
                Vertex::to_vertex,
            );

            match brush_action {
                Ok(action) => {
                    break match action {
                        BrushAction::Draw(vertices) => {
                            self.pipeline.update_vertex_buffer(vertices, device, queue)
                        }
                        BrushAction::ReDraw => (),
                    }
                }

                Err(glyph_brush::BrushError::TextureTooSmall { suggested }) => {
                    if log::log_enabled!(log::Level::Warn) {
                        log::warn!(
                            "Resizing cache texture! This should be avoided \
                            by building TextBrush with BrushBuilder::initial_cache_size() \
                            and providing bigger cache texture dimensions."
                        );
                    }
                    let max_image_dimension = device.limits().max_texture_dimension_2d;
                    let (width, height) = if suggested.0 > max_image_dimension
                        || suggested.1 > max_image_dimension
                    {
                        if self.inner.texture_dimensions().0 < max_image_dimension
                            || self.inner.texture_dimensions().1 < max_image_dimension
                        {
                            (max_image_dimension, max_image_dimension)
                        } else {
                            return Err(BrushError::TooBigCacheTexture(
                                max_image_dimension,
                            ));
                        }
                    } else {
                        suggested
                    };
                    self.pipeline.resize_texture(device, (width, height));
                    self.inner.resize_texture(width, height);
                }
            }
        }
        Ok(())
    }

    /// Resizes the view. Updates the default orthographic view matrix with
    /// provided dimensions and uses it for rendering.
    ///
    /// Run this function whenever the surface config is resized.
    /// **Surface** dimensions are most commonly *width* and *height*.
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
        self.update_matrix(crate::ortho(width, height), queue);
    }

    /// Resizes the view. Updates text rendering matrix with the provided one.
    ///
    /// Use [`Self::resize_view()`] to update and replace the current render matrix
    /// with a default orthographic matrix.
    ///
    /// Feel free to use [`ortho()`] to create more complex matrices by yourself.
    #[inline]
    pub fn update_matrix<M>(&mut self, matrix: M, queue: &wgpu::Queue)
    where
        M: Into<Matrix>,
    {
        self.pipeline.update_matrix(matrix.into(), queue);
    }
}

/// Builder for [`TextBrush`].
#[non_exhaustive]
pub struct BrushBuilder<F, H = DefaultSectionHasher> {
    inner: glyph_brush::GlyphBrushBuilder<F, H>,
    depth: Option<wgpu::DepthStencilState>,
    matrix: Option<Matrix>,
}

impl BrushBuilder<()> {
    /// Creates a [`BrushBuilder`] with [`Font`].
    pub fn using_font<F: Font>(font: F) -> BrushBuilder<F> {
        BrushBuilder::using_fonts(vec![font])
    }

    /// Creates a [`BrushBuilder`] with font byte data.
    pub fn using_font_bytes(data: &[u8]) -> Result<BrushBuilder<FontRef>, InvalidFont> {
        let font = FontRef::try_from_slice(data)?;
        Ok(BrushBuilder::using_fonts(vec![font]))
    }

    /// Creates a [`BrushBuilder`] with multiple fonts byte data.
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
            depth: None,
            matrix: None,
        }
    }
}

impl<F, H> BrushBuilder<F, H>
where
    F: Font,
    H: std::hash::BuildHasher,
{
    // Default `BrushBuilder` functions:
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

    /// Provide the *depth_stencil* if you are planning to utilize depth testing.
    ///
    /// For each section, depth can be set by modifying the z coordinate
    /// ([`glyph_brush::OwnedText::with_z()`]).
    ///
    /// `z` coordinate should be in range
    ///  [0.0, 1.0] not including 1.0.
    pub fn with_depth_stencil(
        mut self,
        depth: Option<wgpu::DepthStencilState>,
    ) -> BrushBuilder<F, H> {
        self.depth = depth;
        self
    }

    /// Builds a [`TextBrush`] while consuming [`BrushBuilder`], for later drawing text
    /// onto a texture of the specified width, height and [`wgpu::TextureFormat`].
    /// You can provide a [`wgpu::TextureView`] while calling the `draw()` function.
    ///
    /// If you are drawing a basic UI, you'd most likely want to be using
    /// [`wgpu::SurfaceConfiguration`]'s dimensions and texture format.
    pub fn build(
        self,
        device: &wgpu::Device,
        output_width: u32,
        output_height: u32,
        output_format: wgpu::TextureFormat,
    ) -> TextBrush<F, H> {
        let inner = self.inner.build();

        let matrix = self
            .matrix
            .unwrap_or_else(|| crate::ortho(output_width as f32, output_height as f32));

        let pipeline = Pipeline::new(
            device,
            output_format,
            self.depth,
            inner.texture_dimensions(),
            matrix,
        );

        TextBrush { inner, pipeline }
    }
}
