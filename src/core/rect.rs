//! Geometric primitives for GUI layout and rendering.
//!
//! [`Rect`], [`Position`], [`Size`], and [`Margin`] form the spatial
//! foundation for widget placement and hit-testing.

use serde::{Deserialize, Serialize};

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

    pub const fn uniform(v: f32) -> Self {
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }

    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert!(r.contains(Position::new(50.0, 40.0)));
        assert!(!r.contains(Position::new(5.0, 40.0)));
    }

    #[test]
    fn rect_intersection() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(50.0, 50.0, 100.0, 100.0);
        let i = a.intersection(&b).unwrap();
        assert_eq!(i.x, 50.0);
        assert_eq!(i.y, 50.0);
        assert_eq!(i.width, 50.0);
        assert_eq!(i.height, 50.0);
    }

    #[test]
    fn rect_inner_margin() {
        let r = Rect::new(0.0, 0.0, 200.0, 100.0);
        let inner = r.inner(&Margin::uniform(10.0));
        assert_eq!(inner.x, 10.0);
        assert_eq!(inner.y, 10.0);
        assert_eq!(inner.width, 180.0);
        assert_eq!(inner.height, 80.0);
    }
}
