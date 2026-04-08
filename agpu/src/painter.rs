//! AgpuPainter — implements `crate::paint::Painter` using wgpu.
//!
//! Collects all drawing commands during a frame, maintaining a clip stack.
//! The actual GPU rendering happens when `finish()` is called with the
//! render pass.

use crate::core::{Color, Position, Rect, Size, TextStyle};
use crate::paint::{Gradient, ImageHandle, Painter, Shadow};
use crate::renderer::ShapeRenderer;
use crate::text::TextEngine;

/// A [`Painter`] backed by agpu's wgpu-based 2D renderer.
pub struct AgpuPainter<'a> {
    shapes: &'a mut ShapeRenderer,
    text: &'a mut TextEngine,
    clip_stack: Vec<Rect>,
}

impl<'a> AgpuPainter<'a> {
    /// Create a new painter wrapping the shape renderer and text engine.
    pub fn new(shapes: &'a mut ShapeRenderer, text: &'a mut TextEngine) -> Self {
        Self {
            shapes,
            text,
            clip_stack: Vec::new(),
        }
    }

    /// Get the current clip rectangle, if any.
    fn current_clip(&self) -> Option<Rect> {
        self.clip_stack.last().copied()
    }
}

impl Painter for AgpuPainter<'_> {
    fn fill_rect(&mut self, rect: Rect, color: Color, corner_radius: f32) {
        self.shapes.fill_rect(rect, color, corner_radius);
    }

    fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32, corner_radius: f32) {
        self.shapes.stroke_rect(rect, color, width, corner_radius);
    }

    fn stroke_rounded_rect(
        &mut self,
        rect: Rect,
        color: Color,
        width: f32,
        corner_radius: f32,
        bg_color: Color,
    ) {
        self.shapes
            .stroke_rounded_rect(rect, color, width, corner_radius, bg_color);
    }

    fn fill_circle(&mut self, center: Position, radius: f32, color: Color) {
        self.shapes.fill_circle(center, radius, color);
    }

    fn stroke_circle(&mut self, center: Position, radius: f32, color: Color, width: f32) {
        self.shapes.stroke_circle(center, radius, color, width);
    }

    fn line(&mut self, from: Position, to: Position, color: Color, width: f32) {
        self.shapes.line(from, to, color, width);
    }

    fn text(&mut self, pos: Position, text: &str, style: &TextStyle) {
        let clip = self.current_clip();
        self.text.draw(pos, text, style, clip);
    }

    fn measure_text(&self, text: &str, style: &TextStyle) -> Size {
        // Estimation fallback when only &self is available.
        let w = style.font_size * 0.6 * text.len() as f32;
        Size::new(w, style.font_size * 1.2)
    }

    fn measure_text_mut(&mut self, text: &str, style: &TextStyle) -> Size {
        self.text.measure(text, style)
    }

    fn push_clip(&mut self, rect: Rect) {
        self.clip_stack.push(rect);
    }

    fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    fn fill_rect_gradient(&mut self, rect: Rect, gradient: &Gradient, corner_radius: f32) {
        self.shapes
            .fill_rect_gradient(rect, gradient, corner_radius);
    }

    fn shadow_rect(&mut self, rect: Rect, shadow: &Shadow, corner_radius: f32) {
        self.shapes.shadow_rect(rect, shadow, corner_radius);
    }

    fn draw_image(&mut self, handle: &ImageHandle, rect: Rect) {
        self.shapes.draw_image(handle, rect);
    }
}
