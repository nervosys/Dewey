//! Modal dialog widget.

use crate::core::{Color, Position, Rect, TextStyle};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, SemanticRole, UiNode, WidgetSchema,
};
use crate::paint::Painter;
use crate::widget::Widget;

/// A modal overlay dialog.
pub struct Modal {
    pub id: String,
    pub title: String,
    pub visible: bool,
}

impl Modal {
    pub fn new(id: impl Into<String>, title: impl Into<String>, visible: bool) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            visible,
        }
    }
}

impl Widget for Modal {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        if !self.visible {
            return;
        }

        // Overlay backdrop
        painter.fill_rect(area, Color::rgba(0.0, 0.0, 0.0, 0.5), 0.0);

        // Dialog box centred in the area
        let dialog_w = (area.width * 0.5).min(400.0);
        let dialog_h = (area.height * 0.4).min(240.0);
        let dialog_x = area.x + (area.width - dialog_w) * 0.5;
        let dialog_y = area.y + (area.height - dialog_h) * 0.5;
        let dialog = Rect::new(dialog_x, dialog_y, dialog_w, dialog_h);

        painter.fill_rect(dialog, Color::rgba(0.18, 0.18, 0.22, 1.0), 6.0);
        painter.stroke_rect(dialog, Color::rgba(0.35, 0.35, 0.45, 1.0), 1.0, 6.0);

        // Title
        let style = TextStyle {
            font_size: 16.0,
            color: Color::WHITE,
            ..TextStyle::default()
        };
        let text_size = painter.measure_text(&self.title, &style);
        painter.text(
            Position::new(
                dialog_x + (dialog_w - text_size.width) * 0.5,
                dialog_y + 16.0,
            ),
            &self.title,
            &style,
        );
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("Modal", SemanticRole::Modal).with_id(&self.id)
    }
}

impl Discoverable for Modal {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new("Modal", "A modal dialog overlay", SemanticRole::Modal)
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::Focusable, AgentCapability::Closable]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::simple("open", "Show the modal", true),
            AgentAction::simple("close", "Hide the modal", true),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Modal
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "title": self.title,
            "visible": self.visible,
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "open" => {
                self.visible = true;
                Ok(serde_json::json!({ "visible": true }))
            }
            "close" => {
                self.visible = false;
                Ok(serde_json::json!({ "visible": false }))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}
