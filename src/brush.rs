use crate::{
    ortho,
    pipeline::{Pipeline, Vertex},
    Matrix, ScissorRegion,
};
use glyph_brush::{
    ab_glyph::{Font, FontArc, FontRef, InvalidFont, Rect},
    BrushAction, DefaultSectionHasher, Extra, GlyphCruncher, Section,
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
    /// Queues section for drawing. This method should be called
    /// every frame for each section that is going to be drawn.
    ///
    /// This can be called multiple times for different sections
    /// that want to use the same font and gpu cache.
    ///
    #[inline]
    pub fn queue<'a, S>(&mut self, section: S)
    where
        S: Into<std::borrow::Cow<'a, Section<'a>>>,
    {
        self.inner.queue(section)
    }

    /// Returns a bounding box for the section glyphs calculated
    /// using each glyph's vertical & horizontal metrics.
    /// For more info, read about: [`GlyphCruncher::glyph_bounds`].
    #[inline]
    pub fn glyph_bounds<'a, S>(&mut self, section: S) -> Option<Rect>
    where
        S: Into<std::borrow::Cow<'a, Section<'a>>>,
    {
        self.inner.glyph_bounds(section)
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

    /// Draws all sections queued with [`queue`](#method.queue) function.
    ///
    /// You can specify where to draw the text when providing the' view'.
    /// For example, instead of giving the current `frame texture view`
    /// and drawing to it, you can provide the texture view of an off-screen
    /// texture and draw the text on there.
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
    /// You can specify where to draw the text when providing the' view'.
    /// For example, instead of giving the current `frame texture view`
    /// and drawing to it, you can provide the texture view of an off-screen
    /// texture and draw the text on there.
    ///
    /// ## Scissoring
    /// With scissoring, you can filter out each glyph fragment that crosses the given `region`.
    // TODO rename to draw_extra
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

    /// Resizes the view. Updates the default orthographic view matrix
    /// with provided dimensions and uses it for rendering.
    ///
    /// Run this function whenever the surface config is resized.
    /// *width* and *height* are most commonly **surface** dimensions.
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

    /// Resizes the view. Updates text rendering matrix with the provided one.
    ///
    /// Use [`Self::resize_view()`] to update and replace the current render matrix
    /// with a default orthographic matrix.
    ///
    /// Feel free to use [`ortho()`] to create more complex matrices by yourself.
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
    /// Should be called every time the window (`wgpu::SurfaceConfiguration`)
    /// is being resized. If not used when required, the program will
    /// crash with *wgpu error*.
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

    /// Defaults to `false`. If set to true all text will be depth tested.
    ///
    /// For each section, depth can be set by modifying
    /// the z coordinate ([`OwnedText::with_z()`]).
    ///
    /// `z` coordinate should be in range
    ///  [0.0, 1.0] not including 1.0.
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

    /// Builds a [`TextBrush`] while consuming [`BrushBuilder`].
    ///
    /// Afterwards, text can only be drawn within the [`wgpu::SurfaceConfiguration`] dimensions
    /// on the surface texture.
    /// Use [`Self::build_custom()`]
    ///
    /// Call [`Self::with_depth_testing()`] before building to utilize `depth testing`.
    pub fn build(
        self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> TextBrush<F, H> {
        self.build_custom(device, config.width, config.height, config.format)
    }

    /// Builds a [`TextBrush`] while consuming [`BrushBuilder`], for later drawing text
    /// onto a texture of the specified width, height and [`wgpu::TextureFormat`].
    /// You can provide [`wgpu::TextureView`] while calling the `draw` function.
    ///
    /// Call [`Self::with_depth_testing()`] before building to utilize `depth testing`.
    pub fn build_custom(
        self,
        device: &wgpu::Device,
        output_width: u32,
        output_height: u32,
        output_format: wgpu::TextureFormat,
    ) -> TextBrush<F, H> {
        let inner = self.inner.build();

        let matrix = self
            .matrix
            .unwrap_or_else(|| ortho(output_width as f32, output_height as f32));

        let had_depth = self.depth_testing.is_some();
        let mut pipeline = Pipeline::new(
            device,
            output_format,
            self.depth_testing,
            inner.texture_dimensions(),
            matrix,
        );
        if had_depth {
            pipeline.update_depth(device, (output_width, output_height));
        }

        TextBrush { inner, pipeline }
    }
}
