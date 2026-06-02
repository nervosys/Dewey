//! System tray integration for Dewey.
//!
//! Provides a platform-abstracted system tray API. The actual platform
//! implementation is behind the `system-tray` feature (future work).
//! This module defines the types and a trait for tray interaction.

use crate::ontology::*;

/// A menu item in the system tray context menu.
#[derive(Debug, Clone)]
pub enum TrayMenuItem {
    /// A clickable text item.
    Item {
        id: String,
        label: String,
        enabled: bool,
    },
    /// A separator line.
    Separator,
    /// A submenu.
    SubMenu {
        label: String,
        items: Vec<TrayMenuItem>,
    },
    /// A checkable item.
    CheckItem {
        id: String,
        label: String,
        checked: bool,
    },
}

impl TrayMenuItem {
    /// Create a simple menu item.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Item {
            id: id.into(),
            label: label.into(),
            enabled: true,
        }
    }

    /// Create a separator.
    pub fn separator() -> Self {
        Self::Separator
    }

    /// Create a submenu.
    pub fn submenu(label: impl Into<String>, items: Vec<TrayMenuItem>) -> Self {
        Self::SubMenu {
            label: label.into(),
            items,
        }
    }

    /// Create a checkable item.
    pub fn check(id: impl Into<String>, label: impl Into<String>, checked: bool) -> Self {
        Self::CheckItem {
            id: id.into(),
            label: label.into(),
            checked,
        }
    }
}

/// Configuration for a system tray icon.
#[derive(Debug, Clone)]
pub struct TrayConfig {
    /// Tooltip text shown on hover.
    pub tooltip: String,
    /// Context menu items.
    pub menu: Vec<TrayMenuItem>,
}

impl TrayConfig {
    /// Create tray config with a tooltip.
    pub fn new(tooltip: impl Into<String>) -> Self {
        Self {
            tooltip: tooltip.into(),
            menu: Vec::new(),
        }
    }

    /// Set the context menu.
    pub fn with_menu(mut self, menu: Vec<TrayMenuItem>) -> Self {
        self.menu = menu;
        self
    }
}

/// Events from the system tray.
#[derive(Debug, Clone)]
pub enum TrayEvent {
    /// A menu item was clicked.
    MenuItemClicked(String),
    /// The tray icon was double-clicked.
    DoubleClick,
}

/// Trait for system tray backends.
///
/// Implementors provide the platform-specific tray interaction.
pub trait TrayBackend {
    /// Show the tray icon with the given configuration.
    fn show(&mut self, config: &TrayConfig) -> Result<(), String>;

    /// Update the tooltip text.
    fn set_tooltip(&mut self, tooltip: &str) -> Result<(), String>;

    /// Update the context menu.
    fn set_menu(&mut self, menu: &[TrayMenuItem]) -> Result<(), String>;

    /// Hide and remove the tray icon.
    fn hide(&mut self) -> Result<(), String>;

    /// Poll for tray events (non-blocking).
    fn poll_event(&mut self) -> Option<TrayEvent>;
}

/// A stub tray backend that does nothing (for headless / test mode).
pub struct NullTrayBackend;

impl TrayBackend for NullTrayBackend {
    fn show(&mut self, _config: &TrayConfig) -> Result<(), String> {
        Ok(())
    }
    fn set_tooltip(&mut self, _tooltip: &str) -> Result<(), String> {
        Ok(())
    }
    fn set_menu(&mut self, _menu: &[TrayMenuItem]) -> Result<(), String> {
        Ok(())
    }
    fn hide(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn poll_event(&mut self) -> Option<TrayEvent> {
        None
    }
}

impl Discoverable for NullTrayBackend {
    fn schema(&self) -> WidgetSchema {
        let mut schema = WidgetSchema::new(
            "TrayBackend",
            "System tray icon with context menu and event polling",
            SemanticRole::Configuration,
        );
        schema.usage_hint = Some("tray.show(&TrayConfig::new(\"tooltip\"))".into());
        schema.tags = vec![
            "tray".into(),
            "system".into(),
            "notification".into(),
            "icon".into(),
        ];
        schema
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::Clickable]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::with_params(
                "show",
                "Show the system tray icon",
                vec![ActionParam::required(
                    "tooltip",
                    "Tooltip text",
                    ActionParamType::String,
                )],
                true,
            ),
            AgentAction::with_params(
                "set_tooltip",
                "Update the tray tooltip",
                vec![ActionParam::required(
                    "tooltip",
                    "Tooltip text",
                    ActionParamType::String,
                )],
                true,
            ),
            AgentAction::simple("hide", "Hide the tray icon", true),
            AgentAction::simple("poll_event", "Check for tray events", false),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Configuration
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "backend": "null",
            "note": "Headless mode — tray actions are no-ops",
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "show" => {
                let tooltip = params["tooltip"].as_str().ok_or("missing tooltip")?;
                let config = TrayConfig::new(tooltip);
                self.show(&config).map_err(|e| e.to_string())?;
                Ok(serde_json::json!({ "shown": true }))
            }
            "set_tooltip" => {
                let tooltip = params["tooltip"].as_str().ok_or("missing tooltip")?;
                self.set_tooltip(tooltip).map_err(|e| e.to_string())?;
                Ok(serde_json::json!({ "tooltip": tooltip }))
            }
            "hide" => {
                self.hide().map_err(|e| e.to_string())?;
                Ok(serde_json::json!({ "hidden": true }))
            }
            "poll_event" => {
                let event = self.poll_event();
                Ok(serde_json::json!({ "event": event.map(|e| format!("{:?}", e)) }))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_tray_backend() {
        let mut tray = NullTrayBackend;
        let config = TrayConfig::new("Test App").with_menu(vec![
            TrayMenuItem::new("quit", "Quit"),
            TrayMenuItem::separator(),
            TrayMenuItem::check("dark", "Dark Mode", true),
        ]);
        tray.show(&config).unwrap();
        tray.set_tooltip("Updated").unwrap();
        assert!(tray.poll_event().is_none());
        tray.hide().unwrap();
    }
}
