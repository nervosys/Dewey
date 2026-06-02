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
    bg_color: Option<Color>,
    fg_color: Option<Color>,
    corner_radius: Option<f32>,
    font_size: Option<f32>,
}

impl Tooltip {
    #[must_use]
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            bg_color: None,
            fg_color: None,
            corner_radius: None,
            font_size: None,
        }
    }

    #[must_use]
    pub fn bg(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    #[must_use]
    pub fn fg(mut self, color: Color) -> Self {
        self.fg_color = Some(color);
        self
    }

    #[must_use]
    pub fn rounded(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    #[must_use]
    pub fn text_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }
}

impl Widget for Tooltip {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        let bg = self.bg_color.unwrap_or(Color::rgba(0.12, 0.12, 0.16, 0.95));
        let border = Color::rgba(0.35, 0.35, 0.45, 1.0);
        let radius = self.corner_radius.unwrap_or(4.0);
        painter.fill_rect(area, bg, radius);
        painter.stroke_rect(area, border, 1.0, radius);

        let style = TextStyle {
            font_size: self.font_size.unwrap_or(12.0),
            color: self.fg_color.unwrap_or(Color::rgba(0.9, 0.9, 0.9, 1.0)),
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
        WidgetSchema::new(
            "Tooltip",
            "A tooltip popup showing help text",
            SemanticRole::Display,
        )
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::HasTooltip]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::simple(
            "set_text",
            "Set tooltip text content",
            true,
        )]
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
