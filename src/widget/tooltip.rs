//! Tooltip widget — displays a hover tip that wraps inner content.

use crate::core::style::TextStyle;
use crate::core::{Color, Position, Rect};
use crate::ontology::*;
use crate::runtime::Frame;
use crate::widget::Widget;

/// A tooltip that wraps a label and shows hover text.
///
/// Renders the label text. The tooltip text is exposed via the ontology
/// for agent discovery. Visual tooltip popups are handled by the backend.
pub struct Tooltip {
    /// The visible label text.
    label: String,
    /// The tooltip text shown on hover.
    text: String,
    agent_id: String,
}

impl Tooltip {
    /// Create a new tooltip with text that appears on hover.
    /// `label` is shown inline; `text` appears when the user hovers.
    pub fn new(label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            text: text.into(),
            agent_id: String::new(),
        }
    }

    /// Create a tooltip with only hover text (empty label).
    pub fn hover_only(text: impl Into<String>) -> Self {
        Self {
            label: String::new(),
            text: text.into(),
            agent_id: String::new(),
        }
    }

    pub fn agent_id(mut self, id: impl Into<String>) -> Self {
        self.agent_id = id.into();
        self
    }
}

impl Discoverable for Tooltip {
    fn schema(&self) -> WidgetSchema {
        let mut schema =
            WidgetSchema::new("Tooltip", "A tooltip shown on hover", SemanticRole::Display);
        schema.usage_hint = Some("Tooltip::new(\"Hover me\", \"Extra info\")".into());
        schema.tags = vec!["tooltip".into(), "hover".into(), "hint".into()];
        schema
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::HasTooltip]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Display
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({ "label": self.label, "text": self.text })
    }

    fn execute_action(
        &mut self,
        _action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err("Tooltip has no actions".to_string())
    }

    fn agent_id(&self) -> Option<&str> {
        if self.agent_id.is_empty() {
            None
        } else {
            Some(&self.agent_id)
        }
    }

    fn accessibility_label(&self) -> Option<String> {
        if self.label.is_empty() {
            None
        } else {
            Some(self.label.clone())
        }
    }
}

impl Widget for Tooltip {
    fn render(self, area: Rect, frame: &mut Frame<'_>) {
        if !self.agent_id.is_empty() {
            let node = UiNode::new("Tooltip", SemanticRole::Display)
                .with_id(&self.agent_id)
                .with_bounds(area.into())
                .with_property("label", serde_json::json!(self.label))
                .with_property("text", serde_json::json!(self.text));
            frame.register_widget(node);
        }

        let label = if self.label.is_empty() {
            "(?)"
        } else {
            &self.label
        };
        let ts = TextStyle {
            font_size: 14.0,
            color: Color::WHITE,
            ..Default::default()
        };
        frame
            .painter()
            .text(Position::new(area.x, area.y), label, &ts);
    }
}
