//! Theming — complete theme system with dark/light modes and custom palettes.

use crate::core::Color;
use serde::{Deserialize, Serialize};

/// Color scheme mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
}

/// Spacing scale for consistent layout.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Spacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            xs: 2.0,
            sm: 4.0,
            md: 8.0,
            lg: 16.0,
            xl: 32.0,
        }
    }
}

/// Typography scale.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Typography {
    pub body: f32,
    pub small: f32,
    pub heading1: f32,
    pub heading2: f32,
    pub heading3: f32,
    pub mono: f32,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            body: 14.0,
            small: 11.0,
            heading1: 28.0,
            heading2: 22.0,
            heading3: 17.0,
            mono: 13.0,
        }
    }
}

/// Border radius scale.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BorderRadius {
    pub none: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub full: f32,
}

impl Default for BorderRadius {
    fn default() -> Self {
        Self {
            none: 0.0,
            sm: 2.0,
            md: 4.0,
            lg: 8.0,
            full: 9999.0,
        }
    }
}

/// Color palette for a theme.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Palette {
    pub background: Color,
    pub surface: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    pub border: Color,
    pub hover: Color,
    pub pressed: Color,
    pub focus_ring: Color,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            background: Color::rgba(0.08, 0.08, 0.10, 1.0),
            surface: Color::rgba(0.14, 0.14, 0.18, 1.0),
            primary: Color::rgba(0.35, 0.55, 0.95, 1.0),
            secondary: Color::rgba(0.55, 0.35, 0.85, 1.0),
            accent: Color::rgba(0.0, 0.8, 0.65, 1.0),
            error: Color::rgba(0.9, 0.25, 0.25, 1.0),
            warning: Color::rgba(0.95, 0.7, 0.2, 1.0),
            success: Color::rgba(0.2, 0.8, 0.35, 1.0),
            text_primary: Color::rgba(0.92, 0.92, 0.95, 1.0),
            text_secondary: Color::rgba(0.65, 0.65, 0.7, 1.0),
            text_disabled: Color::rgba(0.4, 0.4, 0.45, 1.0),
            border: Color::rgba(0.25, 0.25, 0.3, 1.0),
            hover: Color::rgba(1.0, 1.0, 1.0, 0.06),
            pressed: Color::rgba(1.0, 1.0, 1.0, 0.1),
            focus_ring: Color::rgba(0.35, 0.55, 0.95, 0.5),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::rgba(0.97, 0.97, 0.98, 1.0),
            surface: Color::WHITE,
            primary: Color::rgba(0.2, 0.4, 0.85, 1.0),
            secondary: Color::rgba(0.5, 0.3, 0.8, 1.0),
            accent: Color::rgba(0.0, 0.65, 0.55, 1.0),
            error: Color::rgba(0.85, 0.2, 0.2, 1.0),
            warning: Color::rgba(0.9, 0.6, 0.1, 1.0),
            success: Color::rgba(0.15, 0.7, 0.3, 1.0),
            text_primary: Color::rgba(0.1, 0.1, 0.12, 1.0),
            text_secondary: Color::rgba(0.4, 0.4, 0.45, 1.0),
            text_disabled: Color::rgba(0.65, 0.65, 0.7, 1.0),
            border: Color::rgba(0.82, 0.82, 0.85, 1.0),
            hover: Color::rgba(0.0, 0.0, 0.0, 0.04),
            pressed: Color::rgba(0.0, 0.0, 0.0, 0.08),
            focus_ring: Color::rgba(0.2, 0.4, 0.85, 0.4),
        }
    }
}

/// A complete theme definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub mode: ThemeMode,
    pub palette: Palette,
    pub spacing: Spacing,
    pub typography: Typography,
    pub border_radius: BorderRadius,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            mode: ThemeMode::Dark,
            palette: Palette::dark(),
            spacing: Spacing::default(),
            typography: Typography::default(),
            border_radius: BorderRadius::default(),
        }
    }

    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            mode: ThemeMode::Light,
            palette: Palette::light(),
            spacing: Spacing::default(),
            typography: Typography::default(),
            border_radius: BorderRadius::default(),
        }
    }

    pub fn custom(name: impl Into<String>, mode: ThemeMode, palette: Palette) -> Self {
        Self {
            name: name.into(),
            mode,
            palette,
            spacing: Spacing::default(),
            typography: Typography::default(),
            border_radius: BorderRadius::default(),
        }
    }

    pub fn is_dark(&self) -> bool {
        self.mode == ThemeMode::Dark
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

/// Theme manager that holds the current theme and allows switching.
pub struct ThemeManager {
    themes: Vec<Theme>,
    active: usize,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            themes: vec![Theme::dark(), Theme::light()],
            active: 0,
        }
    }

    pub fn current(&self) -> &Theme {
        &self.themes[self.active]
    }

    pub fn set_active(&mut self, index: usize) {
        if index < self.themes.len() {
            self.active = index;
        }
    }

    pub fn toggle(&mut self) {
        self.active = (self.active + 1) % self.themes.len();
    }

    pub fn add(&mut self, theme: Theme) {
        self.themes.push(theme);
    }

    pub fn list(&self) -> Vec<&str> {
        self.themes.iter().map(|t| t.name.as_str()).collect()
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.themes.iter().position(|t| t.name == name)
    }

    pub fn set_by_name(&mut self, name: &str) -> bool {
        if let Some(idx) = self.find(name) {
            self.active = idx;
            true
        } else {
            false
        }
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_dark_defaults() {
        let theme = Theme::dark();
        assert!(theme.is_dark());
        assert_eq!(theme.mode, ThemeMode::Dark);
    }

    #[test]
    fn theme_light() {
        let theme = Theme::light();
        assert!(!theme.is_dark());
    }

    #[test]
    fn palette_dark_bg_is_dark() {
        let p = Palette::dark();
        assert!(p.background.r < 0.2);
    }

    #[test]
    fn palette_light_bg_is_light() {
        let p = Palette::light();
        assert!(p.background.r > 0.8);
    }

    #[test]
    fn spacing_defaults() {
        let s = Spacing::default();
        assert!(s.xs < s.sm);
        assert!(s.sm < s.md);
        assert!(s.md < s.lg);
        assert!(s.lg < s.xl);
    }

    #[test]
    fn typography_defaults() {
        let t = Typography::default();
        assert!(t.small < t.body);
        assert!(t.body < t.heading3);
    }

    #[test]
    fn border_radius_defaults() {
        let r = BorderRadius::default();
        assert_eq!(r.none, 0.0);
        assert!(r.full > 1000.0);
    }

    #[test]
    fn theme_custom() {
        let theme = Theme::custom("Ocean", ThemeMode::Dark, Palette::dark());
        assert_eq!(theme.name, "Ocean");
    }

    #[test]
    fn theme_manager_toggle() {
        let mut tm = ThemeManager::new();
        assert!(tm.current().is_dark());
        tm.toggle();
        assert!(!tm.current().is_dark());
        tm.toggle();
        assert!(tm.current().is_dark());
    }

    #[test]
    fn theme_manager_add_and_find() {
        let mut tm = ThemeManager::new();
        tm.add(Theme::custom("Solarized", ThemeMode::Light, Palette::light()));
        assert_eq!(tm.list().len(), 3);
        assert!(tm.find("Solarized").is_some());
    }

    #[test]
    fn theme_manager_set_by_name() {
        let mut tm = ThemeManager::new();
        tm.add(Theme::custom("Ocean", ThemeMode::Dark, Palette::dark()));
        assert!(tm.set_by_name("Ocean"));
        assert_eq!(tm.current().name, "Ocean");
        assert!(!tm.set_by_name("Nonexistent"));
    }

    #[test]
    fn theme_serialize_roundtrip() {
        let theme = Theme::dark();
        let json = serde_json::to_string(&theme).unwrap();
        let parsed: Theme = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Dark");
    }
}
