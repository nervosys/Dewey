//! Button widget.

use crate::core::{Color, Position, Rect, TextStyle};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, SemanticRole, UiNode, WidgetSchema,
};
use crate::paint::Painter;
use crate::widget::Widget;

/// A clickable button with a text label.
pub struct Button {
    pub id: String,
    pub label: String,
    pub disabled: bool,
    pub icon: Option<String>,
    bg_color: Option<Color>,
    fg_color: Option<Color>,
    corner_radius: Option<f32>,
    font_size: Option<f32>,
    is_bold: bool,
}

impl Button {
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            icon: None,
            bg_color: None,
            fg_color: None,
            corner_radius: None,
            font_size: None,
            is_bold: false,
        }
    }

    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
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

    #[must_use]
    pub fn bold(mut self) -> Self {
        self.is_bold = true;
        self
    }
}

impl Widget for Button {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        let bg = self.bg_color.unwrap_or(if self.disabled {
            Color::rgba(0.3, 0.3, 0.3, 1.0)
        } else {
            Color::rgba(0.2, 0.4, 0.8, 1.0)
        });
        let fg = self.fg_color.unwrap_or(if self.disabled {
            Color::rgba(0.6, 0.6, 0.6, 1.0)
        } else {
            Color::WHITE
        });
        let radius = self.corner_radius.unwrap_or(4.0);

        painter.fill_rect(area, bg, radius);
        let style = TextStyle {
            font_size: self.font_size.unwrap_or(14.0),
            color: fg,
            ..TextStyle::default()
        };
        let text_size = painter.measure_text(&self.label, &style);
        let tx = area.x + (area.width - text_size.width) * 0.5;
        let ty = area.y + (area.height - text_size.height) * 0.5;
        painter.text(Position::new(tx, ty), &self.label, &style);
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("Button", SemanticRole::Action).with_id(&self.id)
    }
}

impl Discoverable for Button {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new("Button", "A clickable button", SemanticRole::Action)
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        if self.disabled {
            vec![]
        } else {
            vec![AgentCapability::Clickable, AgentCapability::Focusable]
        }
    }

    fn actions(&self) -> Vec<AgentAction> {
        if self.disabled {
            vec![]
        } else {
            vec![AgentAction::simple("click", "Click the button", false)]
        }
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Action
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({ "label": self.label, "disabled": self.disabled })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "click" if !self.disabled => Ok(serde_json::json!({ "clicked": true })),
            "click" => Err("Button is disabled".into()),
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}
