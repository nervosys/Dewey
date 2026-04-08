//! Backend-agnostic 2D painting interface.
//!
//! The [`Painter`] trait is the core rendering abstraction in agpu. Widgets
//! render exclusively through this trait, making the rendering backend fully
//! pluggable.
//!
//! # Implementations
//!
//! | Backend            | Purpose                                       |
//! |--------------------|-----------------------------------------------|
//! | `AgpuPainter`      | GPU-accelerated rendering via wgpu             |
//! | `NullPainter`      | Discards all output (headless / agent mode)    |

use crate::core::{Color, Position, Rect, Size, TextStyle};

/// Gradient definition for fill operations.
#[derive(Debug, Clone)]
pub enum Gradient {
    /// Linear gradient from `start` to `end` position.
    Linear {
        start: Position,
        end: Position,
        stops: Vec<GradientStop>,
    },
    /// Radial gradient from `center` outward.
    Radial {
        center: Position,
        radius: f32,
        stops: Vec<GradientStop>,
    },
}

/// A color stop in a gradient.
#[derive(Debug, Clone, Copy)]
pub struct GradientStop {
    pub offset: f32,
    pub color: Color,
}

impl GradientStop {
    pub fn new(offset: f32, color: Color) -> Self {
        Self { offset, color }
    }
}

/// Shadow parameters for shape rendering.
#[derive(Debug, Clone, Copy)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub color: Color,
}

impl Shadow {
    pub fn new(offset_x: f32, offset_y: f32, blur_radius: f32, color: Color) -> Self {
        Self {
            offset_x,
            offset_y,
            blur_radius,
            color,
        }
    }
}

impl Default for Shadow {
    fn default() -> Self {
        Self {
            offset_x: 2.0,
            offset_y: 2.0,
            blur_radius: 4.0,
            color: Color::rgba(0.0, 0.0, 0.0, 0.3),
        }
    }
}

/// Image handle for texture-backed drawing.
#[derive(Debug, Clone)]
pub struct ImageHandle {
    /// Unique image identifier.
    pub id: u64,
    /// Image dimensions.
    pub width: u32,
    pub height: u32,
}

impl ImageHandle {
    pub fn new(id: u64, width: u32, height: u32) -> Self {
        Self { id, width, height }
    }
}

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

    /// Stroke a rounded rectangle with a known background color for proper inner cutout.
    fn stroke_rounded_rect(
        &mut self,
        rect: Rect,
        color: Color,
        width: f32,
        corner_radius: f32,
        bg_color: Color,
    ) {
        // Default: delegate to stroke_rect
        self.stroke_rect(rect, color, width, corner_radius);
        let _ = bg_color;
    }

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

    /// Measure text precisely (requires mutable access for font shaping).
    fn measure_text_mut(&mut self, text: &str, style: &TextStyle) -> Size {
        self.measure_text(text, style)
    }

    /// Push a clipping rectangle. Drawing outside this rect is discarded.
    fn push_clip(&mut self, rect: Rect);

    /// Pop the most recent clipping rectangle.
    fn pop_clip(&mut self);

    /// Fill a rectangle with a gradient.
    fn fill_rect_gradient(&mut self, rect: Rect, gradient: &Gradient, corner_radius: f32) {
        // Default: use the first stop color as solid fill
        let color = match gradient {
            Gradient::Linear { stops, .. } | Gradient::Radial { stops, .. } => {
                stops.first().map(|s| s.color).unwrap_or(Color::WHITE)
            }
        };
        self.fill_rect(rect, color, corner_radius);
    }

    /// Draw a shadow behind a rectangle.
    fn shadow_rect(&mut self, rect: Rect, shadow: &Shadow, corner_radius: f32) {
        // Default: approximate shadow as an offset, expanded semi-transparent rect
        let expand = shadow.blur_radius * 0.5;
        let shadow_rect = Rect::new(
            rect.x + shadow.offset_x - expand,
            rect.y + shadow.offset_y - expand,
            rect.width + expand * 2.0,
            rect.height + expand * 2.0,
        );
        self.fill_rect(shadow_rect, shadow.color, corner_radius + expand);
    }

    /// Draw an image within the given rectangle.
    fn draw_image(&mut self, _handle: &ImageHandle, _rect: Rect) {
        // Default: no-op (not all backends support images)
    }
    /// Draw an image with optional tint color.
    fn draw_image_tinted(&mut self, handle: &ImageHandle, rect: Rect, _tint: Color) {
        self.draw_image(handle, rect);
    }
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
    fn fill_rect_gradient(&mut self, _rect: Rect, _gradient: &Gradient, _corner_radius: f32) {}
    fn shadow_rect(&mut self, _rect: Rect, _shadow: &Shadow, _corner_radius: f32) {}
    fn draw_image(&mut self, _handle: &ImageHandle, _rect: Rect) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_painter_measure_text() {
        let np = NullPainter;
        let style = TextStyle::default();
        let size = np.measure_text("hello", &style);
        assert!(size.width > 0.0);
        assert!(size.height > 0.0);
    }

    #[test]
    fn null_painter_measure_empty() {
        let np = NullPainter;
        let style = TextStyle::default();
        let size = np.measure_text("", &style);
        assert_eq!(size.width, 0.0);
        assert!(size.height > 0.0);
    }

    #[test]
    fn null_painter_operations_do_not_panic() {
        let mut np = NullPainter;
        np.fill_rect(Rect::ZERO, Color::RED, 0.0);
        np.stroke_rect(Rect::ZERO, Color::RED, 1.0, 0.0);
        np.fill_circle(Position::ZERO, 10.0, Color::GREEN);
        np.stroke_circle(Position::ZERO, 10.0, Color::GREEN, 1.0);
        np.line(
            Position::ZERO,
            Position::new(100.0, 100.0),
            Color::BLUE,
            2.0,
        );
        np.text(Position::ZERO, "test", &TextStyle::default());
        np.push_clip(Rect::ZERO);
        np.pop_clip();
    }
}
