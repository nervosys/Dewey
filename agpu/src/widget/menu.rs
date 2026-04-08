//! Menu widget with menu items, separators, and sub-menus.

use crate::core::{Color, Position, Rect, TextStyle};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, SemanticRole, UiNode, WidgetSchema,
};
use crate::paint::Painter;
use crate::widget::Widget;

/// A single item in a menu.
#[derive(Debug, Clone)]
pub enum MenuItem {
    /// A clickable item with an id and display label.
    Item { id: String, label: String },
    /// A visual separator line.
    Separator,
}

impl MenuItem {
    /// Create a clickable menu item.
    pub fn item(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Item {
            id: id.into(),
            label: label.into(),
        }
    }

    /// Create a separator.
    pub fn separator() -> Self {
        Self::Separator
    }
}

/// A vertical context/popup menu.
pub struct Menu {
    pub id: String,
    pub items: Vec<MenuItem>,
}

impl Menu {
    pub fn new(id: impl Into<String>, items: Vec<MenuItem>) -> Self {
        Self {
            id: id.into(),
            items,
        }
    }
}

impl Widget for Menu {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        painter.fill_rect(area, Color::rgba(0.16, 0.16, 0.2, 1.0), 4.0);
        painter.stroke_rect(area, Color::rgba(0.35, 0.35, 0.45, 1.0), 1.0, 4.0);

        let item_height = 28.0;
        let sep_height = 8.0;
        let padding = 12.0;

        let style = TextStyle {
            font_size: 13.0,
            color: Color::WHITE,
            ..TextStyle::default()
        };

        let mut y = area.y + 4.0;
        for item in &self.items {
            match item {
                MenuItem::Item { label, .. } => {
                    painter.text(
                        Position::new(area.x + padding, y + (item_height - style.font_size) * 0.5),
                        label,
                        &style,
                    );
                    y += item_height;
                }
                MenuItem::Separator => {
                    let sep_y = y + sep_height * 0.5;
                    painter.line(
                        Position::new(area.x + 4.0, sep_y),
                        Position::new(area.x + area.width - 4.0, sep_y),
                        Color::rgba(0.3, 0.3, 0.4, 1.0),
                        1.0,
                    );
                    y += sep_height;
                }
            }
        }
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("Menu", SemanticRole::Menu).with_id(&self.id)
    }
}

impl Discoverable for Menu {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new("Menu", "A context or popup menu", SemanticRole::Menu)
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::Focusable]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::simple("select_item", "Activate a menu item by id", false)]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Menu
    }

    fn agent_state(&self) -> serde_json::Value {
        let items: Vec<serde_json::Value> = self
            .items
            .iter()
            .filter_map(|item| match item {
                MenuItem::Item { id, label } => {
                    Some(serde_json::json!({ "id": id, "label": label }))
                }
                MenuItem::Separator => None,
            })
            .collect();
        serde_json::json!({ "items": items })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "select_item" => {
                if let Some(id) = params.get("id").and_then(|v| v.as_str()) {
                    let found = self.items.iter().any(|item| matches!(item, MenuItem::Item { id: iid, .. } if iid == id));
                    if found {
                        Ok(serde_json::json!({ "selected": id }))
                    } else {
                        Err(format!("Unknown menu item: {id}"))
                    }
                } else {
                    Err("Missing 'id' parameter".into())
                }
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}
