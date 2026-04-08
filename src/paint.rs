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
