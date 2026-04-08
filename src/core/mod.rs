//! Core geometric primitives, colors, and styles for GUI rendering.
//!
//! Provides the foundational types used throughout Dewey:
//! [`Rect`], [`Position`], [`Size`], [`Color`], and [`Style`].

pub mod rect;
pub mod style;

pub use rect::{Margin, Position, Rect, Size};
pub use style::{
    Alignment, Color, CursorIcon, FontWeight, Shadow, Style, TextStyle, VerticalAlignment,
};
