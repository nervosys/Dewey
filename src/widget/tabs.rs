//! Tabs widget — a tabbed panel selector.

use crate::core::style::TextStyle;
use crate::core::{Color, Position, Rect};
use crate::ontology::*;
use crate::runtime::Frame;
use crate::widget::StatefulWidget;

/// Tab state.
pub struct TabState {
    pub selected: usize,
}

impl TabState {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn with_selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }
}

impl Default for TabState {
    fn default() -> Self {
        Self::new()
    }
}

/// A tab bar.
pub struct Tabs {
    labels: Vec<String>,
    agent_id: String,
}

impl Tabs {
    pub fn new(labels: Vec<String>) -> Self {
        Self {
            labels,
            agent_id: String::new(),
        }
    }

    pub fn agent_id(mut self, id: impl Into<String>) -> Self {
        self.agent_id = id.into();
        self
    }
}

impl Discoverable for Tabs {
    fn schema(&self) -> WidgetSchema {
        let mut schema = WidgetSchema::new("Tabs", "A tab bar for switching views", SemanticRole::Tab);
        schema.usage_hint = Some("Tabs::new(vec![\"Home\".into(), \"Settings\".into()])".into());
        schema.tags = vec!["tabs".into(), "navigation".into(), "switch".into()];
        schema
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Selectable { multi_select: false, item_count: self.labels.len() },
            AgentCapability::Focusable,
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::with_params(
            "select_tab",
            "Select a tab by index",
            vec![ActionParam::required("index", "Tab index", ActionParamType::Index)],
            true,
        )]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Tab
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({ "labels": self.labels })
    }

    fn execute_action(&mut self, _action: &str, _params: &serde_json::Value) -> Result<serde_json::Value, String> {
        Err("Use StatefulWidget for state mutations".to_string())
    }

    fn agent_id(&self) -> Option<&str> {
        if self.agent_id.is_empty() { None } else { Some(&self.agent_id) }
    }

    fn accessibility_label(&self) -> Option<String> {
        Some(self.labels.join(", "))
    }
}

impl StatefulWidget for Tabs {
    type State = TabState;

    fn render(self, area: Rect, frame: &mut Frame<'_>, state: &mut TabState) {
        if !self.agent_id.is_empty() {
            let node = UiNode::new("Tabs", SemanticRole::Tab)
                .with_id(&self.agent_id)
                .with_bounds(area.into())
                .with_property("labels", serde_json::json!(self.labels))
                .with_property("selected", serde_json::json!(state.selected));
            frame.register_widget(node);
            frame.register_hitbox(&self.agent_id, area, 1);
        }

        let tab_h = 28.0;
        let ts = TextStyle { font_size: 14.0, color: Color::WHITE, ..Default::default() };
        let mut x = area.x;
        for (i, label) in self.labels.iter().enumerate() {
            let text_w = frame.painter().measure_text(label, &ts).width + 16.0;
            let tab_rect = Rect::new(x, area.y, text_w, tab_h);
            if state.selected == i {
                frame.painter().fill_rect(tab_rect, Color::DARK_GRAY, 4.0);
            }
            frame.painter().text(Position::new(x + 8.0, area.y + 6.0), label, &ts);
            x += text_w;
        }
        // Bottom border
        frame.painter().line(
            Position::new(area.x, area.y + tab_h),
            Position::new(area.x + area.width, area.y + tab_h),
            Color::GRAY,
            1.0,
        );
    }
}
