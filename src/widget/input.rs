//! Text input widget — single-line text entry.

use crate::core::style::TextStyle;
use crate::core::{Color, Position, Rect};
use crate::ontology::*;
use crate::runtime::Frame;
use crate::widget::StatefulWidget;

/// State for a text input.
pub struct TextInputState {
    pub text: String,
    pub cursor: usize,
    pub focused: bool,
}

impl TextInputState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            focused: false,
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.cursor = self.text.len();
        self
    }
}

impl Default for TextInputState {
    fn default() -> Self {
        Self::new()
    }
}

/// A single-line text input.
pub struct TextInput {
    placeholder: String,
    agent_id: String,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            placeholder: String::new(),
            agent_id: String::new(),
        }
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn agent_id(mut self, id: impl Into<String>) -> Self {
        self.agent_id = id.into();
        self
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Discoverable for TextInput {
    fn schema(&self) -> WidgetSchema {
        let mut schema =
            WidgetSchema::new("TextInput", "A single-line text input", SemanticRole::Input);
        schema.usage_hint = Some("TextInput::new().placeholder(\"Enter name...\")".into());
        schema.tags = vec!["input".into(), "text".into(), "field".into()];
        schema
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::TextInput {
                multiline: false,
                max_length: None,
            },
            AgentCapability::Focusable,
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::with_params(
                "set_text",
                "Set the input text",
                vec![ActionParam::required(
                    "text",
                    "The text to set",
                    ActionParamType::String,
                )],
                true,
            ),
            AgentAction::simple("clear", "Clear the input", true),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Input
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({ "placeholder": self.placeholder })
    }

    fn execute_action(
        &mut self,
        _action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err("Use StatefulWidget for state mutations".to_string())
    }

    fn agent_id(&self) -> Option<&str> {
        if self.agent_id.is_empty() {
            None
        } else {
            Some(&self.agent_id)
        }
    }

    fn accessibility_label(&self) -> Option<String> {
        if self.placeholder.is_empty() {
            None
        } else {
            Some(self.placeholder.clone())
        }
    }
}

impl StatefulWidget for TextInput {
    type State = TextInputState;

    fn render(self, area: Rect, frame: &mut Frame<'_>, state: &mut TextInputState) {
        if !self.agent_id.is_empty() {
            let node = UiNode::new("TextInput", SemanticRole::Input)
                .with_id(&self.agent_id)
                .with_bounds(area.into())
                .with_property("text", serde_json::json!(state.text))
                .with_property("placeholder", serde_json::json!(self.placeholder))
                .with_property("focused", serde_json::json!(state.focused));
            frame.register_widget(node);
            frame.register_hitbox(&self.agent_id, area, 1);
        }

        // Border
        frame.painter().stroke_rect(area, Color::GRAY, 1.0, 2.0);
        // Text or placeholder
        let ts = TextStyle {
            font_size: 14.0,
            color: Color::WHITE,
            ..Default::default()
        };
        if state.text.is_empty() {
            let mut pts = ts.clone();
            pts.color = Color::GRAY;
            frame.painter().text(
                Position::new(area.x + 4.0, area.y + 4.0),
                &self.placeholder,
                &pts,
            );
        } else {
            frame
                .painter()
                .text(Position::new(area.x + 4.0, area.y + 4.0), &state.text, &ts);
        }
        // Cursor
        if state.focused {
            let before = &state.text[..state.cursor.min(state.text.len())];
            let cursor_x = frame.painter().measure_text(before, &ts).width;
            frame.painter().line(
                Position::new(area.x + 4.0 + cursor_x, area.y + 3.0),
                Position::new(area.x + 4.0 + cursor_x, area.y + area.height - 3.0),
                Color::WHITE,
                1.0,
            );
        }
    }
}
