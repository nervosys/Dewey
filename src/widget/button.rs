//! Button widget — a clickable element.

use crate::core::style::TextStyle;
use crate::core::{Color, Rect, Style};
use crate::ontology::*;
use crate::runtime::Frame;
use crate::widget::Widget;

/// A clickable button.
pub struct Button {
    label: String,
    style: Style,
    enabled: bool,
    agent_id: String,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            style: Style::default(),
            enabled: true,
            agent_id: String::new(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn agent_id(mut self, id: impl Into<String>) -> Self {
        self.agent_id = id.into();
        self
    }
}

impl Discoverable for Button {
    fn schema(&self) -> WidgetSchema {
        let mut schema = WidgetSchema::new("Button", "A clickable button", SemanticRole::Action);
        schema.usage_hint = Some("Button::new(\"Save\").enabled(true)".into());
        schema.tags = vec!["button".into(), "click".into(), "action".into()];
        schema
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::Clickable, AgentCapability::Focusable]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::simple("click", "Click the button", false)]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Action
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({ "label": self.label, "enabled": self.enabled })
    }

    fn execute_action(&mut self, action: &str, _params: &serde_json::Value) -> Result<serde_json::Value, String> {
        match action {
            "click" if self.enabled => Ok(serde_json::json!({ "clicked": true })),
            "click" => Err("Button is disabled".to_string()),
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        if self.agent_id.is_empty() { None } else { Some(&self.agent_id) }
    }

    fn accessibility_label(&self) -> Option<String> {
        Some(self.label.clone())
    }
}

impl Widget for Button {
    fn render(self, area: Rect, frame: &mut Frame<'_>) {
        if !self.agent_id.is_empty() {
            let node = UiNode::new("Button", SemanticRole::Action)
                .with_id(&self.agent_id)
                .with_bounds(area.into())
                .with_property("label", serde_json::json!(self.label))
                .with_property("enabled", serde_json::json!(self.enabled));
            frame.register_widget(node);
            frame.register_hitbox(&self.agent_id, area, 1);
        }

        let bg = if self.enabled {
            self.style.background.unwrap_or(Color::DARK_GRAY)
        } else {
            Color::GRAY
        };
        let fg = self.style.resolved_fg();
        let radius = self.style.border_radius.unwrap_or(4.0);
        frame.painter().fill_rect(area, bg, radius);
        frame.painter().stroke_rect(area, fg.with_alpha(0.3), 1.0, radius);
        let ts = TextStyle { font_size: 14.0, color: fg, ..Default::default() };
        let text_size = frame.painter().measure_text(&self.label, &ts);
        let tx = area.x + (area.width - text_size.width) * 0.5;
        let ty = area.y + (area.height - text_size.height) * 0.5;
        frame.painter().text(crate::core::Position::new(tx, ty), &self.label, &ts);
    }
}
