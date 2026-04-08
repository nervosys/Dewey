//! Label widget — displays static or dynamic text.

use crate::core::style::TextStyle;
use crate::core::{Position, Rect, Style};
use crate::ontology::*;
use crate::runtime::Frame;
use crate::widget::Widget;

/// A text label.
pub struct Label {
    text: String,
    style: Style,
    agent_id: String,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
            agent_id: String::new(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn agent_id(mut self, id: impl Into<String>) -> Self {
        self.agent_id = id.into();
        self
    }
}

impl Discoverable for Label {
    fn schema(&self) -> WidgetSchema {
        let mut schema = WidgetSchema::new("Label", "A text label", SemanticRole::Display);
        schema.usage_hint = Some("Label::new(\"Hello world\")".into());
        schema.tags = vec!["label".into(), "text".into(), "display".into()];
        schema
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::Focusable]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Display
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({ "text": self.text })
    }

    fn execute_action(&mut self, _action: &str, _params: &serde_json::Value) -> Result<serde_json::Value, String> {
        Err("Label has no actions".to_string())
    }

    fn agent_id(&self) -> Option<&str> {
        if self.agent_id.is_empty() { None } else { Some(&self.agent_id) }
    }

    fn accessibility_label(&self) -> Option<String> {
        Some(self.text.clone())
    }
}

impl Widget for Label {
    fn render(self, area: Rect, frame: &mut Frame<'_>) {
        if !self.agent_id.is_empty() {
            let node = UiNode::new("Label", SemanticRole::Display)
                .with_id(&self.agent_id)
                .with_bounds(area.into())
                .with_property("text", serde_json::json!(self.text));
            frame.register_widget(node);
        }

        let ts = TextStyle {
            font_size: 14.0,
            color: self.style.resolved_fg(),
            ..Default::default()
        };
        frame.painter().text(Position::new(area.x, area.y), &self.text, &ts);
    }
}
