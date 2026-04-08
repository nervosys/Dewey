//! Core geometry and style types for agpu.
//!
//! Provides [`Color`], [`Rect`], [`Position`], [`Size`], [`Margin`],
//! [`TextStyle`], and [`FontWeight`] — the spatial and visual primitives
//! used throughout the rendering pipeline.

use serde::{Deserialize, Serialize};

// ── Color ───────────────────────────────────────────────────────────

/// RGBA color with f32 components in [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);
    pub const YELLOW: Self = Self::rgb(1.0, 1.0, 0.0);
    pub const CYAN: Self = Self::rgb(0.0, 1.0, 1.0);
    pub const MAGENTA: Self = Self::rgb(1.0, 0.0, 1.0);
    pub const GRAY: Self = Self::rgb(0.5, 0.5, 0.5);
    pub const DARK_GRAY: Self = Self::rgb(0.25, 0.25, 0.25);
    pub const LIGHT_GRAY: Self = Self::rgb(0.75, 0.75, 0.75);

    #[must_use]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    #[must_use]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create from 0-255 integer components.
    #[must_use]
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }

    /// Create from 0-255 integer components with alpha.
    #[must_use]
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::rgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    /// Linear interpolation between two colors.
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    /// Apply alpha multiplier.
    #[must_use]
    pub fn with_alpha(self, alpha: f32) -> Self {
        Self { a: alpha, ..self }
    }
}

// ── Rect ────────────────────────────────────────────────────────────

/// A 2D rectangle defined by position and size in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for Rect {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Rect {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub fn from_size(width: f32, height: f32) -> Self {
        Self::new(0.0, 0.0, width, height)
    }

    #[must_use]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    #[must_use]
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    #[must_use]
    pub fn center(&self) -> Position {
        Position {
            x: self.x + self.width / 2.0,
            y: self.y + self.height / 2.0,
        }
    }

    #[must_use]
    pub fn contains(&self, pos: Position) -> bool {
        pos.x >= self.x && pos.x < self.right() && pos.y >= self.y && pos.y < self.bottom()
    }

    #[must_use]
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        if right > x && bottom > y {
            Some(Rect::new(x, y, right - x, bottom - y))
        } else {
            None
        }
    }

    #[must_use]
    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Rect::new(x, y, right - x, bottom - y)
    }

    #[must_use]
    pub fn inner(&self, margin: &Margin) -> Rect {
        Rect::new(
            self.x + margin.left,
            self.y + margin.top,
            (self.width - margin.left - margin.right).max(0.0),
            (self.height - margin.top - margin.bottom).max(0.0),
        )
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    #[must_use]
    pub fn size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    #[must_use]
    pub fn position(&self) -> Position {
        Position {
            x: self.x,
            y: self.y,
        }
    }

    #[must_use]
    pub fn translate(&self, dx: f32, dy: f32) -> Self {
        Self::new(self.x + dx, self.y + dy, self.width, self.height)
    }

    #[must_use]
    pub fn inflate(&self, dx: f32, dy: f32) -> Self {
        Self::new(
            self.x - dx,
            self.y - dy,
            self.width + 2.0 * dx,
            self.height + 2.0 * dy,
        )
    }
}

// ── Position ────────────────────────────────────────────────────────

/// A 2D point in logical pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Default for Position {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Position {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

// ── Size ────────────────────────────────────────────────────────────

/// A 2D size in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Default for Size {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Size {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

// ── Margin ──────────────────────────────────────────────────────────

/// Margins around a rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Default for Margin {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Margin {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    #[must_use]
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    #[must_use]
    pub const fn uniform(val: f32) -> Self {
        Self::new(val, val, val, val)
    }
}

// ── TextStyle ───────────────────────────────────────────────────────

/// Font weight categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FontWeight {
    Thin,
    Light,
    #[default]
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
}

/// Text style properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_size: f32,
    pub color: Color,
    pub weight: FontWeight,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub line_height: Option<f32>,
    pub letter_spacing: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            color: Color::WHITE,
            weight: FontWeight::Regular,
            italic: false,
            underline: false,
            strikethrough: false,
            line_height: None,
            letter_spacing: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Color ────────────────────────────────────────────────────

    #[test]
    fn color_rgb_constructor() {
        let c = Color::rgb(0.5, 0.6, 0.7);
        assert_eq!(c.a, 1.0);
        assert_eq!(c.r, 0.5);
    }

    #[test]
    fn color_rgba_constructor() {
        let c = Color::rgba(0.1, 0.2, 0.3, 0.4);
        assert_eq!(c.a, 0.4);
    }

    #[test]
    fn color_from_rgb8() {
        let c = Color::from_rgb8(255, 0, 128);
        assert!((c.r - 1.0).abs() < 1e-5);
        assert_eq!(c.g, 0.0);
        assert!((c.b - 128.0 / 255.0).abs() < 1e-5);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn color_from_rgba8() {
        let c = Color::from_rgba8(0, 0, 0, 127);
        assert!((c.a - 127.0 / 255.0).abs() < 1e-5);
    }

    #[test]
    fn color_lerp() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let mid = a.lerp(b, 0.5);
        assert!((mid.r - 0.5).abs() < 1e-5);
        assert!((mid.g - 0.5).abs() < 1e-5);
        assert!((mid.b - 0.5).abs() < 1e-5);
    }

    #[test]
    fn color_lerp_endpoints() {
        let a = Color::RED;
        let b = Color::BLUE;
        let start = a.lerp(b, 0.0);
        let end = a.lerp(b, 1.0);
        assert_eq!(start, a);
        assert_eq!(end, b);
    }

    #[test]
    fn color_with_alpha() {
        let c = Color::RED.with_alpha(0.5);
        assert_eq!(c.r, 1.0);
        assert_eq!(c.a, 0.5);
    }

    #[test]
    fn color_default_is_white() {
        assert_eq!(Color::default(), Color::WHITE);
    }

    #[test]
    fn color_constants_correct() {
        assert_eq!(Color::TRANSPARENT.a, 0.0);
        assert_eq!(Color::BLACK, Color::rgb(0.0, 0.0, 0.0));
        assert_eq!(Color::RED.r, 1.0);
        assert_eq!(Color::GREEN.g, 1.0);
        assert_eq!(Color::BLUE.b, 1.0);
    }

    #[test]
    fn color_serialize_roundtrip() {
        let c = Color::rgba(0.1, 0.2, 0.3, 0.4);
        let json = serde_json::to_string(&c).unwrap();
        let c2: Color = serde_json::from_str(&json).unwrap();
        assert_eq!(c, c2);
    }

    // ── Rect ─────────────────────────────────────────────────────

    #[test]
    fn rect_new_and_accessors() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(r.right(), 110.0);
        assert_eq!(r.bottom(), 70.0);
    }

    #[test]
    fn rect_from_size() {
        let r = Rect::from_size(100.0, 50.0);
        assert_eq!(r.x, 0.0);
        assert_eq!(r.y, 0.0);
        assert_eq!(r.width, 100.0);
    }

    #[test]
    fn rect_center() {
        let r = Rect::new(0.0, 0.0, 100.0, 200.0);
        let c = r.center();
        assert_eq!(c.x, 50.0);
        assert_eq!(c.y, 100.0);
    }

    #[test]
    fn rect_contains() {
        let r = Rect::new(10.0, 10.0, 100.0, 100.0);
        assert!(r.contains(Position::new(50.0, 50.0)));
        assert!(r.contains(Position::new(10.0, 10.0))); // top-left inclusive
        assert!(!r.contains(Position::new(110.0, 110.0))); // right edge exclusive
        assert!(!r.contains(Position::new(5.0, 50.0))); // outside left
    }

    #[test]
    fn rect_intersection() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(50.0, 50.0, 100.0, 100.0);
        let i = a.intersection(&b).unwrap();
        assert_eq!(i, Rect::new(50.0, 50.0, 50.0, 50.0));
    }

    #[test]
    fn rect_intersection_none() {
        let a = Rect::new(0.0, 0.0, 50.0, 50.0);
        let b = Rect::new(100.0, 100.0, 50.0, 50.0);
        assert!(a.intersection(&b).is_none());
    }

    #[test]
    fn rect_union() {
        let a = Rect::new(10.0, 10.0, 50.0, 50.0);
        let b = Rect::new(40.0, 40.0, 80.0, 80.0);
        let u = a.union(&b);
        assert_eq!(u.x, 10.0);
        assert_eq!(u.y, 10.0);
        assert_eq!(u.right(), 120.0);
        assert_eq!(u.bottom(), 120.0);
    }

    #[test]
    fn rect_inner_with_margin() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        let m = Margin::uniform(10.0);
        let inner = r.inner(&m);
        assert_eq!(inner, Rect::new(10.0, 10.0, 80.0, 80.0));
    }

    #[test]
    fn rect_inner_clamps_to_zero() {
        let r = Rect::new(0.0, 0.0, 10.0, 10.0);
        let m = Margin::uniform(20.0);
        let inner = r.inner(&m);
        assert_eq!(inner.width, 0.0);
        assert_eq!(inner.height, 0.0);
    }

    #[test]
    fn rect_is_empty() {
        assert!(Rect::ZERO.is_empty());
        assert!(Rect::new(0.0, 0.0, 0.0, 10.0).is_empty());
        assert!(!Rect::new(0.0, 0.0, 1.0, 1.0).is_empty());
    }

    #[test]
    fn rect_size_and_position() {
        let r = Rect::new(5.0, 10.0, 20.0, 30.0);
        assert_eq!(r.size(), Size::new(20.0, 30.0));
        assert_eq!(r.position(), Position::new(5.0, 10.0));
    }

    #[test]
    fn rect_translate() {
        let r = Rect::new(10.0, 20.0, 50.0, 50.0);
        let t = r.translate(5.0, -10.0);
        assert_eq!(t, Rect::new(15.0, 10.0, 50.0, 50.0));
    }

    #[test]
    fn rect_inflate() {
        let r = Rect::new(10.0, 10.0, 20.0, 20.0);
        let i = r.inflate(5.0, 5.0);
        assert_eq!(i, Rect::new(5.0, 5.0, 30.0, 30.0));
    }

    #[test]
    fn rect_default_is_zero() {
        assert_eq!(Rect::default(), Rect::ZERO);
    }

    #[test]
    fn rect_serialize_roundtrip() {
        let r = Rect::new(1.0, 2.0, 3.0, 4.0);
        let json = serde_json::to_string(&r).unwrap();
        let r2: Rect = serde_json::from_str(&json).unwrap();
        assert_eq!(r, r2);
    }

    // ── Position ─────────────────────────────────────────────────

    #[test]
    fn position_distance() {
        let a = Position::new(0.0, 0.0);
        let b = Position::new(3.0, 4.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn position_distance_to_self_is_zero() {
        let p = Position::new(42.0, 7.0);
        assert_eq!(p.distance_to(&p), 0.0);
    }

    #[test]
    fn position_default_is_zero() {
        assert_eq!(Position::default(), Position::ZERO);
    }

    // ── Size ─────────────────────────────────────────────────────

    #[test]
    fn size_new() {
        let s = Size::new(100.0, 200.0);
        assert_eq!(s.width, 100.0);
        assert_eq!(s.height, 200.0);
    }

    #[test]
    fn size_default_is_zero() {
        assert_eq!(Size::default(), Size::ZERO);
    }

    // ── Margin ───────────────────────────────────────────────────

    #[test]
    fn margin_uniform() {
        let m = Margin::uniform(5.0);
        assert_eq!(m.top, 5.0);
        assert_eq!(m.right, 5.0);
        assert_eq!(m.bottom, 5.0);
        assert_eq!(m.left, 5.0);
    }

    #[test]
    fn margin_default_is_zero() {
        assert_eq!(Margin::default(), Margin::ZERO);
    }

    // ── TextStyle ────────────────────────────────────────────────

    #[test]
    fn text_style_defaults() {
        let ts = TextStyle::default();
        assert_eq!(ts.font_size, 14.0);
        assert_eq!(ts.color, Color::WHITE);
        assert_eq!(ts.weight, FontWeight::Regular);
        assert!(!ts.italic);
        assert!(!ts.underline);
        assert!(!ts.strikethrough);
        assert!(ts.line_height.is_none());
        assert_eq!(ts.letter_spacing, 0.0);
    }

    #[test]
    fn font_weight_default() {
        assert_eq!(FontWeight::default(), FontWeight::Regular);
    }
}
