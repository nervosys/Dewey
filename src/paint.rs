//! Backend-agnostic 2D painting interface.
//!
//! The [`Painter`] trait is the core rendering abstraction in Dewey. Widgets
//! render exclusively through this trait, making the rendering backend fully
//! pluggable. Dewey defines its own rendering contract — it is not a wrapper
//! around any single GUI library.
//!
//! # Implementations
//!
//! | Backend            | Purpose                                       |
//! |--------------------|-----------------------------------------------|
//! | `EguiPainter`      | GPU-accelerated rendering via egui/wgpu        |
//! | `AgpuBridgePainter`| GPU-accelerated rendering via agpu/wgpu        |
//! | `ImagePainter`     | Software RGBA rasterizer                       |
//! | `WebPainter`       | Serialisable ops for wasm targets               |
//! | `NullPainter`      | Discards all output (headless / agent mode)    |
//!
//! The [`TestBackend`](crate::backend::test::TestBackend) also implements
//! `Painter`, recording operations for snapshot testing.

use crate::core::style::TextStyle;
use crate::core::{Color, Position, Rect, Size};

/// Backend-agnostic 2D painter.
///
/// Every widget renders through this trait. Backends implement it to
/// produce actual pixels (GPU), record operations (testing), or discard
/// output (headless).
pub trait Painter {
    /// Fill a rectangle with a solid color and optional corner rounding.
    fn fill_rect(&mut self, rect: Rect, color: Color, corner_radius: f32);

    /// Stroke a rectangle outline.
    fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32, corner_radius: f32);

    /// Fill a circle with a solid color.
    fn fill_circle(&mut self, center: Position, radius: f32, color: Color);

    /// Stroke a circle outline.
    fn stroke_circle(&mut self, center: Position, radius: f32, color: Color, width: f32);

    /// Draw a line segment between two points.
    fn line(&mut self, from: Position, to: Position, color: Color, width: f32);

    /// Draw text at a position using the given style.
    fn text(&mut self, pos: Position, text: &str, style: &TextStyle);

    /// Measure how much space a text string would occupy without drawing it.
    fn measure_text(&self, text: &str, style: &TextStyle) -> Size;

    /// Push a clipping rectangle. Drawing outside this rect is discarded.
    fn push_clip(&mut self, rect: Rect);

    /// Pop the most recent clipping rectangle.
    fn pop_clip(&mut self);

    /// Fill a closed polygon.
    ///
    /// Rectangles and circles cover most of a user interface, but not vector
    /// artwork: a chart's plot area, a map, or a page of a PDF is made of
    /// arbitrary paths with their curves already flattened to points. Backends
    /// that can fill a polygon should; the default approximates with the
    /// bounding box, which keeps something visible rather than nothing on
    /// backends that cannot.
    fn fill_path(&mut self, points: &[Position], color: Color) {
        let Some(bounds) = bounding_box(points) else {
            return;
        };
        self.fill_rect(bounds, color, 0.0);
    }

    /// Stroke a polyline through `points`.
    ///
    /// The default walks the points as line segments, which every backend can
    /// already do, so this is only worth overriding where a backend has a
    /// native path primitive that joins segments better at the corners.
    fn stroke_path(&mut self, points: &[Position], color: Color, width: f32) {
        for pair in points.windows(2) {
            self.line(pair[0], pair[1], color, width);
        }
    }

    /// Draw an RGBA image, scaled to fill `rect`.
    ///
    /// The default draws a placeholder outline. That is honest — a backend
    /// without texture support genuinely cannot show the pixels — but it means
    /// any backend used to render documents needs a real implementation, since
    /// a scanned page *is* the image.
    fn draw_image(&mut self, rect: Rect, _image: &ImageData<'_>) {
        self.stroke_rect(rect, Color::GRAY, 1.0, 0.0);
    }
}

/// Borrowed 8-bit RGBA pixels, row-major from the top left.
///
/// Borrowed rather than owned so a caller can hand over a decoded frame, a
/// memory-mapped asset or a slice of a larger atlas without copying it first.
#[derive(Debug, Clone, Copy)]
pub struct ImageData<'a> {
    pub width: u32,
    pub height: u32,
    /// `width * height * 4` bytes. A shorter slice is treated as transparent
    /// past its end rather than as an error, so a truncated decode degrades to
    /// a partial image instead of failing a frame.
    pub pixels: &'a [u8],
}

impl<'a> ImageData<'a> {
    pub fn new(width: u32, height: u32, pixels: &'a [u8]) -> ImageData<'a> {
        ImageData {
            width,
            height,
            pixels,
        }
    }

    /// The pixel at `(x, y)`, or transparent black if it is out of range.
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        if x >= self.width || y >= self.height {
            return [0, 0, 0, 0];
        }
        let at = ((y as usize * self.width as usize) + x as usize) * 4;
        match self.pixels.get(at..at + 4) {
            Some(px) => [px[0], px[1], px[2], px[3]],
            None => [0, 0, 0, 0],
        }
    }
}

/// The axis-aligned bounds of a point set.
pub fn bounding_box(points: &[Position]) -> Option<Rect> {
    let first = points.first()?;
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (first.x, first.y, first.x, first.y);
    for point in points {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
}

/// A no-op painter that discards all operations.
///
/// Used in headless mode (agent protocol, tests) where no visual output
/// is needed but widgets still run their logic and register ontology.
pub struct NullPainter;

impl Painter for NullPainter {
    fn fill_rect(&mut self, _rect: Rect, _color: Color, _corner_radius: f32) {}
    fn stroke_rect(&mut self, _rect: Rect, _color: Color, _width: f32, _corner_radius: f32) {}
    fn fill_circle(&mut self, _center: Position, _radius: f32, _color: Color) {}
    fn stroke_circle(&mut self, _center: Position, _radius: f32, _color: Color, _width: f32) {}
    fn line(&mut self, _from: Position, _to: Position, _color: Color, _width: f32) {}
    fn text(&mut self, _pos: Position, _text: &str, _style: &TextStyle) {}
    fn measure_text(&self, text: &str, style: &TextStyle) -> Size {
        // Rough estimate: average character width ~ 0.6 * font_size
        let w = style.font_size * 0.6 * text.len() as f32;
        Size::new(w, style.font_size * 1.2)
    }
    fn push_clip(&mut self, _rect: Rect) {}
    fn pop_clip(&mut self) {}
}
