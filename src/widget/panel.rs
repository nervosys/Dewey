//! Panel widget — a framed section of the UI.

use crate::core::style::TextStyle;
use crate::core::{Color, Position, Rect};
use crate::ontology::*;
use crate::runtime::Frame;
use crate::widget::Widget;

/// Panel position within a layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelSide {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

/// A panel that occupies a specific region.
pub struct Panel {
    side: PanelSide,
    title: Option<String>,
    agent_id: String,
}

impl Panel {
    pub fn new(side: PanelSide) -> Self {
        Self {
            side,
            title: None,
            agent_id: String::new(),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn agent_id(mut self, id: impl Into<String>) -> Self {
        self.agent_id = id.into();
        self
    }
}

impl Discoverable for Panel {
    fn schema(&self) -> WidgetSchema {
        let mut schema = WidgetSchema::new("Panel", "A framed panel region", SemanticRole::Container);
        schema.usage_hint = Some("Panel::new(PanelSide::Left).title(\"Explorer\")".into());
        schema.tags = vec!["panel".into(), "region".into(), "layout".into()];
        schema
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::Resizable {
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Container
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({ "side": format!("{:?}", self.side), "title": self.title })
    }

    fn execute_action(&mut self, _action: &str, _params: &serde_json::Value) -> Result<serde_json::Value, String> {
        Err("Panel has no actions".to_string())
    }

    fn agent_id(&self) -> Option<&str> {
        if self.agent_id.is_empty() { None } else { Some(&self.agent_id) }
    }

    fn accessibility_label(&self) -> Option<String> {
        self.title.clone()
    }
}

impl Widget for Panel {
    fn render(self, area: Rect, frame: &mut Frame<'_>) {
        if !self.agent_id.is_empty() {
            let node = UiNode::new("Panel", SemanticRole::Container)
                .with_id(&self.agent_id)
                .with_bounds(area.into())
                .with_property("side", serde_json::json!(format!("{:?}", self.side)));
            frame.register_widget(node);
        }

        frame.painter().fill_rect(area, Color::DARK_GRAY, 0.0);
        frame.painter().stroke_rect(area, Color::GRAY, 1.0, 0.0);
        if let Some(title) = &self.title {
            let ts = TextStyle { font_size: 18.0, color: Color::WHITE, ..Default::default() };
            frame.painter().text(Position::new(area.x + 4.0, area.y + 4.0), title, &ts);
        }
    }
}
