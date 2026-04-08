//! Test backend for headless rendering and snapshot testing.
//!
//! Provides a simple in-memory canvas that records widget operations
//! without requiring a GPU or display server. Implements the
//! [`Painter`] trait so widgets can render
//! identically in tests and production.

use crate::core::style::TextStyle;
use crate::core::{Color, Position, Rect, Size};
use crate::paint::Painter;

/// A recorded rendering operation.
#[derive(Debug, Clone)]
pub enum RenderOp {
    /// A filled rectangle.
    FillRect {
        rect: Rect,
        color: Color,
        corner_radius: f32,
    },
    /// A stroked rectangle.
    StrokeRect {
        rect: Rect,
        color: Color,
        width: f32,
        corner_radius: f32,
    },
    /// A filled circle.
    FillCircle {
        center: Position,
        radius: f32,
        color: Color,
    },
    /// A stroked circle.
    StrokeCircle {
        center: Position,
        radius: f32,
        color: Color,
        width: f32,
    },
    /// Text drawn at a position.
    Text {
        position: Position,
        text: String,
        size: f32,
        color: Color,
    },
    /// A line between two points.
    Line {
        from: Position,
        to: Position,
        color: Color,
        width: f32,
    },
    /// A clip rectangle was pushed.
    PushClip { rect: Rect },
    /// A clip rectangle was popped.
    PopClip,
}

/// A test backend that records render operations for verification.
pub struct TestBackend {
    width: f32,
    height: f32,
    ops: Vec<RenderOp>,
}

impl TestBackend {
    /// Create a new test backend with the given dimensions.
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            ops: Vec::new(),
        }
    }

    /// Get the viewport area.
    pub fn area(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width, self.height)
    }

    /// Get all recorded operations.
    pub fn ops(&self) -> &[RenderOp] {
        &self.ops
    }

    /// Clear all recorded operations.
    pub fn clear(&mut self) {
        self.ops.clear();
    }

    /// Count operations of a specific type.
    pub fn count_fill_rects(&self) -> usize {
        self.ops
            .iter()
            .filter(|op| matches!(op, RenderOp::FillRect { .. }))
            .count()
    }

    /// Count text operations.
    pub fn count_texts(&self) -> usize {
        self.ops
            .iter()
            .filter(|op| matches!(op, RenderOp::Text { .. }))
            .count()
    }

    /// Find all text operations containing the given substring.
    pub fn find_text(&self, needle: &str) -> Vec<&RenderOp> {
        self.ops
            .iter()
            .filter(|op| {
                if let RenderOp::Text { text, .. } = op {
                    text.contains(needle)
                } else {
                    false
                }
            })
            .collect()
    }
}

impl Painter for TestBackend {
    fn fill_rect(&mut self, rect: Rect, color: Color, corner_radius: f32) {
        self.ops.push(RenderOp::FillRect {
            rect,
            color,
            corner_radius,
        });
    }

    fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32, corner_radius: f32) {
        self.ops.push(RenderOp::StrokeRect {
            rect,
            color,
            width,
            corner_radius,
        });
    }

    fn fill_circle(&mut self, center: Position, radius: f32, color: Color) {
        self.ops.push(RenderOp::FillCircle {
            center,
            radius,
            color,
        });
    }

    fn stroke_circle(&mut self, center: Position, radius: f32, color: Color, width: f32) {
        self.ops.push(RenderOp::StrokeCircle {
            center,
            radius,
            color,
            width,
        });
    }

    fn line(&mut self, from: Position, to: Position, color: Color, width: f32) {
        self.ops.push(RenderOp::Line {
            from,
            to,
            color,
            width,
        });
    }

    fn text(&mut self, pos: Position, text: &str, style: &TextStyle) {
        self.ops.push(RenderOp::Text {
            position: pos,
            text: text.to_string(),
            size: style.font_size,
            color: style.color,
        });
    }

    fn measure_text(&self, text: &str, style: &TextStyle) -> Size {
        let w = style.font_size * 0.6 * text.len() as f32;
        Size::new(w, style.font_size * 1.2)
    }

    fn push_clip(&mut self, rect: Rect) {
        self.ops.push(RenderOp::PushClip { rect });
    }

    fn pop_clip(&mut self) {
        self.ops.push(RenderOp::PopClip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::style::TextStyle;

    #[test]
    fn test_backend_records_ops() {
        let mut backend = TestBackend::new(800.0, 600.0);
        backend.fill_rect(Rect::new(0.0, 0.0, 100.0, 50.0), Color::RED, 0.0);
        backend.text(
            Position::new(10.0, 10.0),
            "Hello",
            &TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..Default::default()
            },
        );
        assert_eq!(backend.count_fill_rects(), 1);
        assert_eq!(backend.count_texts(), 1);
        assert_eq!(backend.find_text("Hello").len(), 1);
        assert!(backend.find_text("missing").is_empty());
    }
}
