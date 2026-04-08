//! Select / combo box widget.

use crate::core::style::TextStyle;
use crate::core::{Color, Position, Rect};
use crate::ontology::*;
use crate::runtime::Frame;
use crate::widget::StatefulWidget;

/// Select state.
pub struct SelectState {
    pub selected: usize,
}

impl SelectState {
    pub fn new() -> Self {
        Self { selected: 0 }
    }
}

impl Default for SelectState {
    fn default() -> Self {
        Self::new()
    }
}

/// A dropdown select / combo box.
pub struct Select {
    options: Vec<String>,
    label: String,
    agent_id: String,
}

impl Select {
    pub fn new(label: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            options,
            label: label.into(),
            agent_id: String::new(),
        }
    }

    pub fn agent_id(mut self, id: impl Into<String>) -> Self {
        self.agent_id = id.into();
        self
    }
}

impl Discoverable for Select {
    fn schema(&self) -> WidgetSchema {
        let mut schema = WidgetSchema::new("Select", "A dropdown select", SemanticRole::Selection);
        schema.usage_hint = Some("Select::new(\"Color\", vec![\"Red\".into(), \"Blue\".into()])".into());
        schema.tags = vec!["select".into(), "dropdown".into(), "combo".into()];
        schema
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Selectable { multi_select: false, item_count: self.options.len() },
            AgentCapability::Focusable,
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::with_params(
            "select",
            "Select an option by index",
            vec![ActionParam::required("index", "Option index", ActionParamType::Index)],
            true,
        )]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Selection
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({ "options": self.options, "label": self.label })
    }

    fn execute_action(&mut self, _action: &str, _params: &serde_json::Value) -> Result<serde_json::Value, String> {
        Err("Use StatefulWidget for state mutations".to_string())
    }

    fn agent_id(&self) -> Option<&str> {
        if self.agent_id.is_empty() { None } else { Some(&self.agent_id) }
    }

    fn accessibility_label(&self) -> Option<String> {
        Some(self.label.clone())
    }
}

impl StatefulWidget for Select {
    type State = SelectState;

    fn render(self, area: Rect, frame: &mut Frame<'_>, state: &mut SelectState) {
        if !self.agent_id.is_empty() {
            let selected_text = self.options.get(state.selected).cloned().unwrap_or_default();
            let node = UiNode::new("Select", SemanticRole::Selection)
                .with_id(&self.agent_id)
                .with_bounds(area.into())
                .with_property("options", serde_json::json!(self.options))
                .with_property("selected", serde_json::json!(state.selected))
                .with_property("selected_text", serde_json::json!(selected_text));
            frame.register_widget(node);
            frame.register_hitbox(&self.agent_id, area, 1);
        }

        // Draw select box
        frame.painter().fill_rect(area, Color::DARK_GRAY, 4.0);
        frame.painter().stroke_rect(area, Color::GRAY, 1.0, 4.0);
        let current = self.options.get(state.selected).cloned().unwrap_or_default();
        let ts = TextStyle { font_size: 14.0, color: Color::WHITE, ..Default::default() };
        // Label
        if !self.label.is_empty() {
            let label_ts = TextStyle { font_size: 12.0, color: Color::GRAY, ..Default::default() };
            frame.painter().text(Position::new(area.x, area.y - 16.0), &self.label, &label_ts);
        }
        frame.painter().text(Position::new(area.x + 4.0, area.y + 4.0), &current, &ts);
        // Dropdown arrow
        let arrow_x = area.x + area.width - 16.0;
        let arrow_y = area.y + area.height * 0.5;
        frame.painter().text(Position::new(arrow_x, arrow_y - 7.0), "\u{25BC}", &ts);
    }
}
