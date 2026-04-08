//! Color and style system for GUI rendering.
//!
//! Provides [`Color`] (RGBA f32), [`Style`] (composable visual overrides),
//! [`Shadow`], [`TextStyle`], and alignment types.

use serde::{Deserialize, Serialize};

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

    /// Parse hex color string: "#RRGGBB" or "#RRGGBBAA".
    #[must_use]
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::from_rgb8(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::from_rgba8(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Linear interpolation between two colors.
    #[must_use]
    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    #[must_use]
    pub fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }
}

#[cfg(feature = "egui-backend")]
impl From<Color> for egui::Color32 {
    fn from(c: Color) -> Self {
        egui::Color32::from_rgba_unmultiplied(
            (c.r * 255.0) as u8,
            (c.g * 255.0) as u8,
            (c.b * 255.0) as u8,
            (c.a * 255.0) as u8,
        )
    }
}

#[cfg(feature = "egui-backend")]
impl From<egui::Color32> for Color {
    fn from(c: egui::Color32) -> Self {
        let [r, g, b, a] = c.to_array();
        Self::from_rgba8(r, g, b, a)
    }
}

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

/// Visual style for a widget, composable with optional overrides.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: Option<f32>,
    pub border_radius: Option<f32>,
    pub padding: Option<super::rect::Margin>,
    pub text: Option<TextStyle>,
    pub opacity: Option<f32>,
    pub shadow: Option<Shadow>,
}

impl Style {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn fg(mut self, color: Color) -> Self {
        self.foreground = Some(color);
        self
    }

    #[must_use]
    pub fn bg(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    #[must_use]
    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = Some(width);
        self
    }

    #[must_use]
    pub fn rounded(mut self, radius: f32) -> Self {
        self.border_radius = Some(radius);
        self
    }

    #[must_use]
    pub fn padding(mut self, margin: super::rect::Margin) -> Self {
        self.padding = Some(margin);
        self
    }

    #[must_use]
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity.clamp(0.0, 1.0));
        self
    }

    #[must_use]
    pub fn shadow(mut self, shadow: Shadow) -> Self {
        self.shadow = Some(shadow);
        self
    }

    /// Merge another style on top. Non-None fields in `other` override `self`.
    #[must_use]
    pub fn merge(&self, other: &Style) -> Style {
        Style {
            foreground: other.foreground.or(self.foreground),
            background: other.background.or(self.background),
            border_color: other.border_color.or(self.border_color),
            border_width: other.border_width.or(self.border_width),
            border_radius: other.border_radius.or(self.border_radius),
            padding: other.padding.or(self.padding),
            text: other.text.clone().or(self.text.clone()),
            opacity: other.opacity.or(self.opacity),
            shadow: other.shadow.or(self.shadow),
        }
    }

    /// Resolve foreground color with fallback.
    #[must_use]
    pub fn resolved_fg(&self) -> Color {
        self.foreground.unwrap_or(Color::WHITE)
    }

    /// Resolve background color with fallback.
    #[must_use]
    pub fn resolved_bg(&self) -> Color {
        self.background.unwrap_or(Color::TRANSPARENT)
    }
}

/// Drop shadow specification.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

impl Default for Shadow {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            blur: 0.0,
            spread: 0.0,
            color: Color::TRANSPARENT,
        }
    }
}

impl Shadow {
    #[must_use]
    pub fn new(offset_x: f32, offset_y: f32, blur: f32, color: Color) -> Self {
        Self {
            offset_x,
            offset_y,
            blur,
            spread: 0.0,
            color,
        }
    }
}

/// Text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Alignment {
    #[default]
    Start,
    Center,
    End,
}

/// Vertical alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum VerticalAlignment {
    #[default]
    Top,
    Center,
    Bottom,
}

/// Cursor style for the system cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CursorIcon {
    #[default]
    Default,
    Pointer,
    Text,
    Crosshair,
    Move,
    NotAllowed,
    ResizeNS,
    ResizeEW,
    ResizeNESW,
    ResizeNWSE,
    Grab,
    Grabbing,
    Wait,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_from_hex() {
        let c = Color::from_hex("#ff8800").unwrap();
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.533).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn color_lerp() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let mid = a.lerp(&b, 0.5);
        assert!((mid.r - 0.5).abs() < 0.01);
    }

    #[test]
    fn style_merge() {
        let base = Style::new().fg(Color::RED);
        let over = Style::new().bg(Color::BLUE);
        let merged = base.merge(&over);
        assert_eq!(merged.foreground, Some(Color::RED));
        assert_eq!(merged.background, Some(Color::BLUE));
    }
}
