//! Select / Dropdown widget.

use crate::core::{Color, Position, Rect, TextStyle};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, SemanticRole, UiNode, WidgetSchema,
};
use crate::paint::Painter;
use crate::widget::Widget;

/// A dropdown select widget for single-item selection.
pub struct Select {
    pub id: String,
    pub options: Vec<String>,
    pub selected: Option<usize>,
    pub open: bool,
    pub searchable: bool,
    pub search_query: String,
}

impl Select {
    pub fn new(id: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            id: id.into(),
            options,
            selected: None,
            open: false,
            searchable: false,
            search_query: String::new(),
        }
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    pub fn selected_label(&self) -> &str {
        self.selected
            .and_then(|i| self.options.get(i))
            .map(|s| s.as_str())
            .unwrap_or("Select...")
    }
}

impl Widget for Select {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        painter.fill_rect(area, Color::rgba(0.18, 0.18, 0.22, 1.0), 3.0);
        painter.stroke_rect(area, Color::rgba(0.4, 0.4, 0.5, 1.0), 1.0, 3.0);

        let style = TextStyle {
            font_size: 14.0,
            color: Color::WHITE,
            ..TextStyle::default()
        };

        let padding = 8.0;
        painter.text(
            Position::new(
                area.x + padding,
                area.y + (area.height - style.font_size) * 0.5,
            ),
            self.selected_label(),
            &style,
        );

        // Dropdown arrow
        let arrow_x = area.x + area.width - 20.0;
        let arrow_y = area.y + area.height * 0.5;
        painter.line(
            Position::new(arrow_x, arrow_y - 3.0),
            Position::new(arrow_x + 6.0, arrow_y + 3.0),
            Color::WHITE,
            1.5,
        );
        painter.line(
            Position::new(arrow_x + 6.0, arrow_y + 3.0),
            Position::new(arrow_x + 12.0, arrow_y - 3.0),
            Color::WHITE,
            1.5,
        );
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("Select", SemanticRole::Selection).with_id(&self.id)
    }
}

impl Discoverable for Select {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new("Select", "A dropdown selection", SemanticRole::Selection)
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::Selectable {
                multi_select: false,
                item_count: self.options.len(),
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::simple("open", "Open the dropdown", true),
            AgentAction::simple("close", "Close the dropdown", true),
            AgentAction::simple("select", "Select an option by index", true),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Selection
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "selected": self.selected,
            "selected_label": self.selected_label(),
            "option_count": self.options.len(),
            "open": self.open,
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "open" => {
                self.open = true;
                Ok(serde_json::json!({ "open": true }))
            }
            "close" => {
                self.open = false;
                Ok(serde_json::json!({ "open": false }))
            }
            "select" => {
                if let Some(idx) = params.get("index").and_then(|v| v.as_u64()) {
                    let idx = idx as usize;
                    if idx < self.options.len() {
                        self.selected = Some(idx);
                        self.open = false;
                        Ok(serde_json::json!({ "selected": idx, "label": self.options[idx] }))
                    } else {
                        Err("Index out of range".into())
                    }
                } else {
                    Err("Missing 'index' parameter".into())
                }
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}
