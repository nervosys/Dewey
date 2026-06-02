//! Multi-line text area widget.

use crate::core::{Color, Position, Rect, TextStyle};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, SemanticRole, UiNode, WidgetSchema,
};
use crate::paint::Painter;
use crate::widget::Widget;

/// A multi-line editable text area with scrolling.
pub struct TextArea {
    pub id: String,
    pub value: String,
    pub scroll_offset: f32,
    bg_color: Option<Color>,
    fg_color: Option<Color>,
    corner_radius: Option<f32>,
    font_size: Option<f32>,
    is_bold: bool,
}

impl TextArea {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            value: String::new(),
            scroll_offset: 0.0,
            bg_color: None,
            fg_color: None,
            corner_radius: None,
            font_size: None,
            is_bold: false,
        }
    }

    #[must_use]
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
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

impl Widget for TextArea {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        let bg = self.bg_color.unwrap_or(Color::rgba(0.12, 0.12, 0.15, 1.0));
        let radius = self.corner_radius.unwrap_or(3.0);
        painter.fill_rect(area, bg, radius);
        painter.stroke_rect(area, Color::rgba(0.4, 0.4, 0.5, 1.0), 1.0, radius);

        let fs = self.font_size.unwrap_or(13.0);
        let style = TextStyle {
            font_size: fs,
            color: self.fg_color.unwrap_or(Color::WHITE),
            ..TextStyle::default()
        };

        let padding = 6.0;
        let line_height = style.font_size * 1.4;
        let max_lines = ((area.height - padding * 2.0) / line_height).floor() as usize;

        for (i, line) in self.value.lines().enumerate().take(max_lines) {
            let y = area.y + padding + i as f32 * line_height - self.scroll_offset;
            if y >= area.y && y + line_height <= area.y + area.height {
                painter.text(Position::new(area.x + padding, y), line, &style);
            }
        }
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("TextArea", SemanticRole::Input).with_id(&self.id)
    }
}

impl Discoverable for TextArea {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new("TextArea", "A multi-line text editor", SemanticRole::Input)
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::TextInput {
                multiline: true,
                max_length: None,
            },
            AgentCapability::Scrollable {
                vertical: true,
                horizontal: false,
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::simple("set_value", "Set the text content", true),
            AgentAction::simple("clear", "Clear all text", true),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Input
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "value": self.value,
            "line_count": self.value.lines().count(),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "set_value" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_str()) {
                    self.value = v.to_string();
                    Ok(serde_json::json!({ "value": self.value }))
                } else {
                    Err("Missing 'value' parameter".into())
                }
            }
            "clear" => {
                self.value.clear();
                Ok(serde_json::json!({ "value": "" }))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}
