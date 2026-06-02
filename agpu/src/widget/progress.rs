//! Progress bar widget.

use crate::core::{Color, Rect};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, SemanticRole, UiNode, WidgetSchema,
};
use crate::paint::Painter;
use crate::widget::Widget;

/// A horizontal progress bar (determinate or indeterminate).
pub struct ProgressBar {
    pub id: String,
    /// Progress fraction 0.0–1.0.  `None` = indeterminate.
    pub progress: Option<f32>,
    bg_color: Option<Color>,
    fg_color: Option<Color>,
    corner_radius: Option<f32>,
}

impl ProgressBar {
    /// Create a determinate progress bar.
    #[must_use]
    pub fn new(id: impl Into<String>, progress: f32) -> Self {
        Self {
            id: id.into(),
            progress: Some(progress.clamp(0.0, 1.0)),
            bg_color: None,
            fg_color: None,
            corner_radius: None,
        }
    }

    /// Create an indeterminate (spinning/pulsing) progress bar.
    #[must_use]
    pub fn indeterminate(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            progress: None,
            bg_color: None,
            fg_color: None,
            corner_radius: None,
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
}

impl Widget for ProgressBar {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        let radius = self.corner_radius.unwrap_or(3.0);
        // Track
        let track_bg = self.bg_color.unwrap_or(Color::rgba(0.2, 0.2, 0.25, 1.0));
        painter.fill_rect(area, track_bg, radius);

        // Fill
        let fill_width = match self.progress {
            Some(p) => area.width * p,
            None => area.width * 0.3, // simple placeholder for indeterminate
        };
        let fill = Rect::new(area.x, area.y, fill_width, area.height);
        let fill_color = self.fg_color.unwrap_or(Color::rgba(0.2, 0.6, 1.0, 1.0));
        painter.fill_rect(fill, fill_color, radius);
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("ProgressBar", SemanticRole::Progress).with_id(&self.id)
    }
}

impl Discoverable for ProgressBar {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new(
            "ProgressBar",
            "A progress indicator",
            SemanticRole::Progress,
        )
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::simple(
            "set_progress",
            "Set progress 0.0–1.0",
            true,
        )]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Progress
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "progress": self.progress,
            "determinate": self.progress.is_some(),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "set_progress" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_f64()) {
                    let p = (v as f32).clamp(0.0, 1.0);
                    self.progress = Some(p);
                    Ok(serde_json::json!({ "progress": p }))
                } else {
                    Err("Missing 'value' parameter".into())
                }
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}
