# Theming Guide

Dewey uses a **token-based** theming system. Widgets reference semantic color
tokens rather than hard-coded RGB values, making it straightforward to switch
between themes at runtime.

## Built-in Themes

```rust
use dewey::theme::{Theme, ThemeToken};

let dark = Theme::dark();   // Dark background, light text
let light = Theme::light(); // Light background, dark text
```

## Theme Tokens

| Token        | Purpose                    |
| ------------ | -------------------------- |
| `Primary`    | Primary brand color        |
| `Secondary`  | Secondary brand color      |
| `Accent`     | Accent / highlight         |
| `Background` | General background         |
| `Surface`    | Card and panel backgrounds |
| `Text`       | Default text color         |
| `TextMuted`  | Secondary / muted text     |
| `Error`      | Error / danger states      |
| `Warning`    | Warning states             |
| `Success`    | Positive / success states  |
| `Info`       | Informational states       |
| `Border`     | Borders and separators     |
| `FocusRing`  | Widget focus indicator     |
| `Disabled`   | Disabled elements          |
| `Hover`      | Hover highlight overlay    |
| `Selected`   | Selected / active element  |
| `Overlay`    | Modal scrim / backdrop     |
| `Shadow`     | Drop shadow color          |
| `Link`       | Hyperlink color            |

## Custom Themes

```rust
use dewey::theme::{Theme, ThemeToken};
use dewey::core::Color;

let custom = Theme::new("My Brand")
    .with(ThemeToken::Primary, Color::rgba(0.2, 0.6, 0.9, 1.0))
    .with(ThemeToken::Background, Color::rgba(0.05, 0.05, 0.08, 1.0))
    .with(ThemeToken::Text, Color::rgba(0.95, 0.95, 0.95, 1.0))
    .with_font_size(16.0)
    .with_border_radius(8.0)
    .with_spacing(12.0);
```

## Loading from JSON

Themes can be stored as JSON files and loaded at runtime:

```json
{
  "name": "Corporate",
  "base_font_size": 14.0,
  "border_radius": 4.0,
  "spacing": 8.0,
  "colors": {
    "Primary": [0.15, 0.40, 0.85, 1.0],
    "Background": [0.97, 0.97, 0.98, 1.0],
    "Text": [0.10, 0.10, 0.12, 1.0]
  }
}
```

```rust
use dewey::theme::Theme;
use std::path::Path;

let theme = Theme::load_from_json(Path::new("corporate.json")).unwrap();
```

## Hot Reload with ThemeWatcher

`ThemeWatcher` monitors a theme file and reloads it when modified:

```rust
use dewey::theme::ThemeWatcher;

let watcher = ThemeWatcher::new("theme.json");

// In your frame loop:
if let Some(new_theme) = watcher.check() {
    // Apply new_theme to your application
}
```

This enables live editing of themes during development — save the JSON file
and the application updates immediately.

## Saving Themes

```rust
use std::path::Path;

let dark = Theme::dark();
dark.save_to_json(Path::new("dark-theme.json")).unwrap();
```
