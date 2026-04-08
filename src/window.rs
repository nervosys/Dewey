//! Multi-window support for Dewey.
//!
//! Allows applications to manage multiple windows, each with its own
//! rendering surface and event loop connection.

use crate::core::{Rect, Size};
use crate::ontology::*;

/// Unique identifier for a window.
pub type WindowId = u64;

/// Configuration for creating a new window.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Window title.
    pub title: String,
    /// Initial size in logical pixels.
    pub size: Size,
    /// Whether the window is resizable.
    pub resizable: bool,
    /// Whether the window should be always on top.
    pub always_on_top: bool,
    /// Whether the window has decorations (title bar, borders).
    pub decorated: bool,
    /// Whether the window is transparent.
    pub transparent: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Dewey Window".to_string(),
            size: Size::new(800.0, 600.0),
            resizable: true,
            always_on_top: false,
            decorated: true,
            transparent: false,
        }
    }
}

impl WindowConfig {
    /// Create a new window config with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set the initial size.
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Size::new(width, height);
        self
    }

    /// Set resizable.
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set always on top.
    pub fn with_always_on_top(mut self, on_top: bool) -> Self {
        self.always_on_top = on_top;
        self
    }

    /// Set decorated.
    pub fn with_decorated(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }
}

/// State of a tracked window.
#[derive(Debug)]
pub struct WindowState {
    /// Unique window ID.
    pub id: WindowId,
    /// Window configuration.
    pub config: WindowConfig,
    /// Current window bounds.
    pub bounds: Rect,
    /// Whether the window is currently visible.
    pub visible: bool,
    /// Whether the window is focused.
    pub focused: bool,
}

/// Manages multiple windows.
pub struct WindowManager {
    windows: Vec<WindowState>,
    next_id: WindowId,
}

impl WindowManager {
    /// Create a new window manager.
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            next_id: 1,
        }
    }

    /// Open a new window with the given config. Returns its ID.
    pub fn open(&mut self, config: WindowConfig) -> WindowId {
        let id = self.next_id;
        self.next_id += 1;
        let bounds = Rect::new(0.0, 0.0, config.size.width, config.size.height);
        self.windows.push(WindowState {
            id,
            config,
            bounds,
            visible: true,
            focused: true,
        });
        id
    }

    /// Close a window by ID.
    pub fn close(&mut self, id: WindowId) {
        self.windows.retain(|w| w.id != id);
    }

    /// Get a window state by ID.
    pub fn get(&self, id: WindowId) -> Option<&WindowState> {
        self.windows.iter().find(|w| w.id == id)
    }

    /// Get a mutable window state by ID.
    pub fn get_mut(&mut self, id: WindowId) -> Option<&mut WindowState> {
        self.windows.iter_mut().find(|w| w.id == id)
    }

    /// List all open windows.
    pub fn windows(&self) -> &[WindowState] {
        &self.windows
    }

    /// Get the number of open windows.
    pub fn count(&self) -> usize {
        self.windows.len()
    }

    /// Set focus to a window.
    pub fn focus(&mut self, id: WindowId) {
        for w in &mut self.windows {
            w.focused = w.id == id;
        }
    }

    /// Get the focused window ID.
    pub fn focused(&self) -> Option<WindowId> {
        self.windows.iter().find(|w| w.focused).map(|w| w.id)
    }

    /// Show or hide a window.
    pub fn set_visible(&mut self, id: WindowId, visible: bool) {
        if let Some(w) = self.get_mut(id) {
            w.visible = visible;
        }
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Discoverable for WindowManager {
    fn schema(&self) -> WidgetSchema {
        let mut schema = WidgetSchema::new(
            "WindowManager",
            "Manages multiple application windows with focus tracking",
            SemanticRole::System,
        );
        schema.usage_hint = Some("WindowManager::new().open(WindowConfig::new(\"Title\"))".into());
        schema.tags = vec!["window".into(), "multi-window".into(), "manager".into()];
        schema
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::Closable,
            AgentCapability::Minimizable,
            AgentCapability::Maximizable,
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::with_params(
                "open",
                "Open a new window",
                vec![ActionParam::required("title", "Window title", ActionParamType::String)],
                true,
            ),
            AgentAction::with_params(
                "close",
                "Close a window by ID",
                vec![ActionParam::required("id", "Window ID", ActionParamType::Index)],
                true,
            ),
            AgentAction::with_params(
                "focus",
                "Set focus to a window",
                vec![ActionParam::required("id", "Window ID", ActionParamType::Index)],
                true,
            ),
            AgentAction::with_params(
                "set_visible",
                "Show or hide a window",
                vec![
                    ActionParam::required("id", "Window ID", ActionParamType::Index),
                    ActionParam::required("visible", "Visibility", ActionParamType::Boolean),
                ],
                true,
            ),
            AgentAction::simple("list", "List all windows", false),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::System
    }

    fn agent_state(&self) -> serde_json::Value {
        let windows: Vec<serde_json::Value> = self
            .windows
            .iter()
            .map(|w| {
                serde_json::json!({
                    "id": w.id,
                    "title": w.config.title,
                    "width": w.bounds.width,
                    "height": w.bounds.height,
                    "visible": w.visible,
                    "focused": w.focused,
                    "resizable": w.config.resizable,
                    "decorated": w.config.decorated,
                })
            })
            .collect();
        serde_json::json!({
            "window_count": self.windows.len(),
            "windows": windows,
            "focused_id": self.focused(),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "open" => {
                let title = params["title"]
                    .as_str()
                    .ok_or("missing title")?
                    .to_string();
                let id = self.open(WindowConfig::new(title));
                Ok(serde_json::json!({ "id": id }))
            }
            "close" => {
                let id = params["id"].as_u64().ok_or("missing id")?;
                self.close(id);
                Ok(serde_json::json!({ "closed": id }))
            }
            "focus" => {
                let id = params["id"].as_u64().ok_or("missing id")?;
                self.focus(id);
                Ok(serde_json::json!({ "focused": id }))
            }
            "set_visible" => {
                let id = params["id"].as_u64().ok_or("missing id")?;
                let visible = params["visible"].as_bool().ok_or("missing visible")?;
                self.set_visible(id, visible);
                Ok(serde_json::json!({ "id": id, "visible": visible }))
            }
            "list" => Ok(self.agent_state()),
            _ => Err(format!("Unknown action: {action}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_and_close() {
        let mut wm = WindowManager::new();
        let id1 = wm.open(WindowConfig::new("Window 1"));
        let id2 = wm.open(WindowConfig::new("Window 2"));
        assert_eq!(wm.count(), 2);
        assert_eq!(wm.get(id1).unwrap().config.title, "Window 1");
        wm.close(id1);
        assert_eq!(wm.count(), 1);
        assert!(wm.get(id1).is_none());
        assert!(wm.get(id2).is_some());
    }

    #[test]
    fn focus_management() {
        let mut wm = WindowManager::new();
        let id1 = wm.open(WindowConfig::new("A"));
        let id2 = wm.open(WindowConfig::new("B"));
        // Last opened window gets focus
        assert!(wm.get(id2).unwrap().focused);
        wm.focus(id1);
        assert!(wm.get(id1).unwrap().focused);
        assert!(!wm.get(id2).unwrap().focused);
        assert_eq!(wm.focused(), Some(id1));
    }
}
