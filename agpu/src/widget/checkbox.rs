//! Checkbox and Radio button widgets.

use crate::core::{Color, Position, Rect, TextStyle};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, SemanticRole, UiNode, WidgetSchema,
};
use crate::paint::Painter;
use crate::widget::Widget;

/// A toggle checkbox with a label.
pub struct Checkbox {
    pub id: String,
    pub label: String,
    pub checked: bool,
    bg_color: Option<Color>,
    fg_color: Option<Color>,
    corner_radius: Option<f32>,
    font_size: Option<f32>,
    is_bold: bool,
}

impl Checkbox {
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, checked: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            checked,
            bg_color: None,
            fg_color: None,
            corner_radius: None,
            font_size: None,
            is_bold: false,
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

    #[must_use]
    pub fn bold(mut self) -> Self {
        self.is_bold = true;
        self
    }
}

impl Widget for Checkbox {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        let box_size = 16.0;
        let box_rect = Rect::new(
            area.x,
            area.y + (area.height - box_size) * 0.5,
            box_size,
            box_size,
        );

        let border = self.fg_color.unwrap_or(Color::rgba(0.5, 0.5, 0.6, 1.0));
        let radius = self.corner_radius.unwrap_or(2.0);
        painter.stroke_rect(box_rect, border, 1.0, radius);

        if self.checked {
            let inner = Rect::new(
                box_rect.x + 3.0,
                box_rect.y + 3.0,
                box_size - 6.0,
                box_size - 6.0,
            );
            let check_color = self.bg_color.unwrap_or(Color::rgba(0.2, 0.6, 1.0, 1.0));
            painter.fill_rect(inner, check_color, 1.0);
        }

        let text_color = self.fg_color.unwrap_or(Color::WHITE);
        let style = TextStyle {
            font_size: self.font_size.unwrap_or(14.0),
            color: text_color,
            ..TextStyle::default()
        };
        painter.text(
            Position::new(
                area.x + box_size + 8.0,
                area.y + (area.height - style.font_size) * 0.5,
            ),
            &self.label,
            &style,
        );
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("Checkbox", SemanticRole::Input).with_id(&self.id)
    }
}

impl Discoverable for Checkbox {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new("Checkbox", "A toggle checkbox", SemanticRole::Input)
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::Toggleable {
                state: self.checked,
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::simple("toggle", "Toggle the checkbox", true)]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Input
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({ "checked": self.checked, "label": self.label })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "toggle" => {
                self.checked = !self.checked;
                Ok(serde_json::json!({ "checked": self.checked }))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

/// An exclusive-choice radio button.
pub struct Radio {
    pub id: String,
    pub label: String,
    pub selected: bool,
    bg_color: Option<Color>,
    fg_color: Option<Color>,
    font_size: Option<f32>,
    is_bold: bool,
}

impl Radio {
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, selected: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected,
            bg_color: None,
            fg_color: None,
            font_size: None,
            is_bold: false,
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

impl Widget for Radio {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        let radius = 8.0;
        let center = Position::new(area.x + radius, area.y + area.height * 0.5);

        let border = self.fg_color.unwrap_or(Color::rgba(0.5, 0.5, 0.6, 1.0));
        painter.stroke_circle(center, radius, border, 1.0);

        if self.selected {
            let fill = self.bg_color.unwrap_or(Color::rgba(0.2, 0.6, 1.0, 1.0));
            painter.fill_circle(center, radius - 3.0, fill);
        }

        let text_color = self.fg_color.unwrap_or(Color::WHITE);
        let style = TextStyle {
            font_size: self.font_size.unwrap_or(14.0),
            color: text_color,
            ..TextStyle::default()
        };
        painter.text(
            Position::new(
                area.x + radius * 2.0 + 8.0,
                area.y + (area.height - style.font_size) * 0.5,
            ),
            &self.label,
            &style,
        );
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("Radio", SemanticRole::Input).with_id(&self.id)
    }
}

impl Discoverable for Radio {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new(
            "Radio",
            "An exclusive-choice radio button",
            SemanticRole::Input,
        )
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::Selectable {
                multi_select: false,
                item_count: 1,
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::simple(
            "select",
            "Select this radio option",
            true,
        )]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Input
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({ "selected": self.selected, "label": self.label })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "select" => {
                self.selected = true;
                Ok(serde_json::json!({ "selected": true }))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}
