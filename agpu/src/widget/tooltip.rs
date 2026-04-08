//! Tooltip widget that renders a positioned text popup.

use crate::core::{Color, Position, Rect, TextStyle};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, SemanticRole, UiNode, WidgetSchema,
};
use crate::paint::Painter;
use crate::widget::Widget;

/// A tooltip popup displaying helper text.
pub struct Tooltip {
    pub id: String,
    pub text: String,
}

impl Tooltip {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
        }
    }
}

impl Widget for Tooltip {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        let bg = Color::rgba(0.12, 0.12, 0.16, 0.95);
        let border = Color::rgba(0.35, 0.35, 0.45, 1.0);
        painter.fill_rect(area, bg, 4.0);
        painter.stroke_rect(area, border, 1.0, 4.0);

        let style = TextStyle {
            font_size: 12.0,
            color: Color::rgba(0.9, 0.9, 0.9, 1.0),
            ..TextStyle::default()
        };
        let padding = 6.0;
        painter.text(
            Position::new(area.x + padding, area.y + padding),
            &self.text,
            &style,
        );
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("Tooltip", SemanticRole::Display).with_id(&self.id)
    }
}

impl Discoverable for Tooltip {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new("Tooltip", "A tooltip popup showing help text", SemanticRole::Display)
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::HasTooltip]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::simple("set_text", "Set tooltip text content", true),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Display
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({ "text": self.text })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "set_text" => {
                if let Some(text) = params.get("text").and_then(|v| v.as_str()) {
                    self.text = text.to_string();
                    Ok(serde_json::json!({ "text": self.text }))
                } else {
                    Err("Missing 'text' parameter".into())
                }
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}
