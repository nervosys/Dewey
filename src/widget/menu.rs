//! Menu widget — menus and menu items.

use crate::core::style::TextStyle;
use crate::core::{Color, Position, Rect};
use crate::ontology::*;
use crate::runtime::Frame;
use crate::widget::Widget;

/// A single menu item.
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            enabled: true,
        }
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// A menu containing items.
pub struct Menu {
    title: String,
    items: Vec<MenuItem>,
    agent_id: String,
}

impl Menu {
    pub fn new(title: impl Into<String>, items: Vec<MenuItem>) -> Self {
        Self {
            title: title.into(),
            items,
            agent_id: String::new(),
        }
    }

    pub fn agent_id(mut self, id: impl Into<String>) -> Self {
        self.agent_id = id.into();
        self
    }
}

impl Discoverable for Menu {
    fn schema(&self) -> WidgetSchema {
        let mut schema = WidgetSchema::new("Menu", "A dropdown menu", SemanticRole::Menu);
        schema.usage_hint = Some("Menu::new(\"File\", vec![MenuItem::new(\"Open\")])".into());
        schema.tags = vec!["menu".into(), "dropdown".into(), "navigation".into()];
        schema
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Selectable { multi_select: false, item_count: self.items.len() },
            AgentCapability::Focusable,
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::with_params(
            "select_item",
            "Select a menu item",
            vec![ActionParam::required("label", "Menu item label", ActionParamType::String)],
            true,
        )]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Menu
    }

    fn agent_state(&self) -> serde_json::Value {
        let items: Vec<_> = self.items.iter().map(|i| &i.label).collect();
        serde_json::json!({ "title": self.title, "items": items })
    }

    fn execute_action(&mut self, _action: &str, _params: &serde_json::Value) -> Result<serde_json::Value, String> {
        Err("Menu actions should be handled via messages".to_string())
    }

    fn agent_id(&self) -> Option<&str> {
        if self.agent_id.is_empty() { None } else { Some(&self.agent_id) }
    }

    fn accessibility_label(&self) -> Option<String> {
        Some(self.title.clone())
    }
}

impl Widget for Menu {
    fn render(self, area: Rect, frame: &mut Frame<'_>) {
        if !self.agent_id.is_empty() {
            let node = UiNode::new("Menu", SemanticRole::Menu)
                .with_id(&self.agent_id)
                .with_bounds(area.into())
                .with_property("title", serde_json::json!(self.title));
            frame.register_widget(node);
        }

        // Menu bar background
        let bar_h = 28.0;
        let bar = Rect::new(area.x, area.y, area.width, bar_h);
        frame.painter().fill_rect(bar, Color::DARK_GRAY, 0.0);
        let ts = TextStyle { font_size: 14.0, color: Color::WHITE, ..Default::default() };
        frame.painter().text(Position::new(area.x + 8.0, area.y + 6.0), &self.title, &ts);
    }
}
