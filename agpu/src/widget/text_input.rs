//! Single-line text input widget.

use crate::core::{Color, Position, Rect, TextStyle};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, SemanticRole, UiNode, WidgetSchema,
};
use crate::paint::Painter;
use crate::widget::Widget;

/// A single-line editable text field.
pub struct TextInput {
    pub id: String,
    pub value: String,
    pub placeholder: String,
    pub cursor: usize,
    pub selection: Option<(usize, usize)>,
    bg_color: Option<Color>,
    fg_color: Option<Color>,
    corner_radius: Option<f32>,
    font_size: Option<f32>,
    is_bold: bool,
}

impl TextInput {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            value: String::new(),
            placeholder: String::new(),
            cursor: 0,
            selection: None,
            bg_color: None,
            fg_color: None,
            corner_radius: None,
            font_size: None,
            is_bold: false,
        }
    }

    #[must_use]
    pub fn value(mut self, value: impl Into<String>) -> Self {
        let v = value.into();
        self.cursor = v.len();
        self.value = v;
        self
    }

    #[must_use]
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn insert_char(&mut self, ch: char) {
        self.value.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    pub fn delete_back(&mut self) {
        if self.cursor > 0 {
            let prev = self.value[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.value.drain(prev..self.cursor);
            self.cursor = prev;
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

impl Widget for TextInput {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        let bg = self.bg_color.unwrap_or(Color::rgba(0.15, 0.15, 0.18, 1.0));
        let radius = self.corner_radius.unwrap_or(3.0);
        painter.fill_rect(area, bg, radius);
        painter.stroke_rect(area, Color::rgba(0.4, 0.4, 0.5, 1.0), 1.0, radius);

        let style = TextStyle {
            font_size: self.font_size.unwrap_or(14.0),
            color: self.fg_color.unwrap_or(Color::WHITE),
            ..TextStyle::default()
        };

        let display_text = if self.value.is_empty() {
            &self.placeholder
        } else {
            &self.value
        };
        let text_color = if self.value.is_empty() {
            Color::rgba(0.5, 0.5, 0.5, 1.0)
        } else {
            Color::WHITE
        };

        let text_style = TextStyle {
            color: text_color,
            ..style
        };

        let padding = 6.0;
        painter.text(
            Position::new(
                area.x + padding,
                area.y + (area.height - text_style.font_size) * 0.5,
            ),
            display_text,
            &text_style,
        );
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("TextInput", SemanticRole::Input).with_id(&self.id)
    }
}

impl Discoverable for TextInput {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new("TextInput", "A single-line text field", SemanticRole::Input)
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::TextInput {
                multiline: false,
                max_length: None,
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::simple("set_value", "Set the text value", true),
            AgentAction::simple("clear", "Clear the text value", true),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Input
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "value": self.value,
            "placeholder": self.placeholder,
            "cursor": self.cursor,
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
                    self.cursor = self.value.len();
                    Ok(serde_json::json!({ "value": self.value }))
                } else {
                    Err("Missing 'value' parameter".into())
                }
            }
            "clear" => {
                self.value.clear();
                self.cursor = 0;
                Ok(serde_json::json!({ "value": "" }))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}
