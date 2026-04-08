//! Text rendering via glyphon — GPU-accelerated glyph rasterisation.
//!
//! Wraps the glyphon library to provide `measure` and `draw` operations
//! compatible with the agpu `Painter` trait. Each frame the text
//! renderer collects layout requests, then flushes them in one call.

use crate::context::GpuContext;
use crate::core::{Color, Position, Size, TextStyle};
use crate::types::TextureFormat;
use glyphon::{
    Attrs, Buffer as GlyphonBuffer, Cache, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer as GlyphonRenderer, Viewport,
};

/// GPU text renderer backed by glyphon.
pub struct TextEngine {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub atlas: TextAtlas,
    pub viewport: Viewport,
    pub renderer: GlyphonRenderer,
    /// Pending text draws for the current frame.
    pending: Vec<TextDraw>,
}

struct TextDraw {
    buffer: GlyphonBuffer,
    x: f32,
    y: f32,
    color: Color,
    bounds: TextBounds,
}

impl TextEngine {
    /// Create a new text engine for the given GPU context and surface format.
    pub fn new(gpu: &GpuContext, format: TextureFormat) -> Self {
        Self::with_msaa(gpu, format, 1)
    }

    /// Create a text engine with the given MSAA sample count.
    pub fn with_msaa(gpu: &GpuContext, format: TextureFormat, msaa_samples: u32) -> Self {
        let device = gpu.device();
        let queue = gpu.queue();
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let viewport = Viewport::new(device, &cache);
        let renderer = GlyphonRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            None,
        );
        Self {
            font_system,
            swash_cache,
            atlas,
            viewport,
            renderer,
            pending: Vec::new(),
        }
    }

    /// Measure text without drawing.
    pub fn measure(&mut self, text: &str, style: &TextStyle) -> Size {
        let metrics = Metrics::new(style.font_size, style.font_size * 1.2);
        let mut buffer = GlyphonBuffer::new(&mut self.font_system, metrics);
        buffer.set_size(
            &mut self.font_system,
            Some(f32::MAX),
            Some(style.font_size * 2.0),
        );
        buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;
        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            height += metrics.line_height;
        }
        // Fallback for empty text
        if width == 0.0 {
            width = style.font_size * 0.6 * text.len() as f32;
        }
        if height == 0.0 {
            height = style.font_size * 1.2;
        }
        Size::new(width, height)
    }

    /// Queue text to be drawn at the given position.
    pub fn draw(
        &mut self,
        pos: Position,
        text: &str,
        style: &TextStyle,
        clip: Option<crate::core::Rect>,
    ) {
        let metrics = Metrics::new(style.font_size, style.font_size * 1.2);
        let mut buffer = GlyphonBuffer::new(&mut self.font_system, metrics);
        buffer.set_size(
            &mut self.font_system,
            Some(4096.0),
            Some(style.font_size * 2.0),
        );
        buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        let bounds = if let Some(r) = clip {
            TextBounds {
                left: r.x as i32,
                top: r.y as i32,
                right: (r.x + r.width) as i32,
                bottom: (r.y + r.height) as i32,
            }
        } else {
            TextBounds {
                left: 0,
                top: 0,
                right: i32::MAX,
                bottom: i32::MAX,
            }
        };

        self.pending.push(TextDraw {
            buffer,
            x: pos.x,
            y: pos.y,
            color: style.color,
            bounds,
        });
    }

    /// Flush all pending text draws into the render pass.
    pub fn flush(
        &mut self,
        gpu: &GpuContext,
        pass: &mut wgpu::RenderPass<'_>,
        width: u32,
        height: u32,
    ) {
        if self.pending.is_empty() {
            return;
        }

        let device = gpu.device();
        let queue = gpu.queue();

        let text_areas: Vec<TextArea<'_>> = self
            .pending
            .iter()
            .map(|td| {
                let c = td.color;
                TextArea {
                    buffer: &td.buffer,
                    left: td.x,
                    top: td.y,
                    scale: 1.0,
                    bounds: td.bounds,
                    default_color: glyphon::Color::rgba(
                        (c.r * 255.0) as u8,
                        (c.g * 255.0) as u8,
                        (c.b * 255.0) as u8,
                        (c.a * 255.0) as u8,
                    ),
                    custom_glyphs: &[],
                }
            })
            .collect();

        self.viewport.update(queue, Resolution { width, height });

        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .expect("agpu: text prepare failed");

        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .expect("agpu: text render failed");

        self.pending.clear();
    }

    /// Trim the atlas to free unused textures.
    pub fn trim(&mut self) {
        self.atlas.trim();
    }
}
