//! Progress bar widget.

use crate::core::style::TextStyle;
use crate::core::{Color, Position, Rect};
use crate::ontology::*;
use crate::runtime::Frame;
use crate::widget::Widget;

/// A progress bar showing completion percentage.
pub struct ProgressBar {
    progress: f32,
    label: Option<String>,
    agent_id: String,
}

impl ProgressBar {
    /// Create a progress bar. `progress` is clamped to `[0.0, 1.0]`.
    pub fn new(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            label: None,
            agent_id: String::new(),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn agent_id(mut self, id: impl Into<String>) -> Self {
        self.agent_id = id.into();
        self
    }
}

impl Discoverable for ProgressBar {
    fn schema(&self) -> WidgetSchema {
        let mut schema = WidgetSchema::new("ProgressBar", "A progress indicator", SemanticRole::Progress);
        schema.usage_hint = Some("ProgressBar::new(0.75).label(\"Loading...\")".into());
        schema.tags = vec!["progress".into(), "loading".into(), "bar".into()];
        schema
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Progress
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({ "progress": self.progress })
    }

    fn execute_action(&mut self, _action: &str, _params: &serde_json::Value) -> Result<serde_json::Value, String> {
        Err("ProgressBar has no actions".to_string())
    }

    fn agent_id(&self) -> Option<&str> {
        if self.agent_id.is_empty() { None } else { Some(&self.agent_id) }
    }

    fn accessibility_label(&self) -> Option<String> {
        self.label.clone()
    }
}

impl Widget for ProgressBar {
    fn render(self, area: Rect, frame: &mut Frame<'_>) {
        if !self.agent_id.is_empty() {
            let node = UiNode::new("ProgressBar", SemanticRole::Progress)
                .with_id(&self.agent_id)
                .with_bounds(area.into())
                .with_property("progress", serde_json::json!(self.progress));
            frame.register_widget(node);
        }

        // Background track
        frame.painter().fill_rect(area, Color::DARK_GRAY, 4.0);
        // Filled portion
        let fill_w = area.width * self.progress;
        if fill_w > 0.0 {
            let fill = Rect::new(area.x, area.y, fill_w, area.height);
            frame.painter().fill_rect(fill, Color::BLUE, 4.0);
        }
        // Label
        if let Some(label) = &self.label {
            let ts = TextStyle { font_size: 12.0, color: Color::WHITE, ..Default::default() };
            let text_size = frame.painter().measure_text(label, &ts);
            let tx = area.x + (area.width - text_size.width) * 0.5;
            let ty = area.y + (area.height - text_size.height) * 0.5;
            frame.painter().text(Position::new(tx, ty), label, &ts);
        }
    }
}
