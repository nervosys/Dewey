//! Slider widget for continuous/discrete value selection.

use crate::core::{Color, Position, Rect};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, SemanticRole, UiNode, WidgetSchema,
};
use crate::paint::Painter;
use crate::widget::Widget;

/// A horizontal slider for selecting a value within a range.
pub struct Slider {
    pub id: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: Option<f32>,
}

impl Slider {
    pub fn new(id: impl Into<String>, value: f32, min: f32, max: f32) -> Self {
        Self {
            id: id.into(),
            value: value.clamp(min, max),
            min,
            max,
            step: None,
        }
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Normalized value (0.0–1.0).
    pub fn normalized(&self) -> f32 {
        if (self.max - self.min).abs() < f32::EPSILON {
            0.0
        } else {
            (self.value - self.min) / (self.max - self.min)
        }
    }
}

impl Widget for Slider {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        let track_height = 4.0;
        let track_y = area.y + (area.height - track_height) * 0.5;
        let track = Rect::new(area.x, track_y, area.width, track_height);

        painter.fill_rect(track, Color::rgba(0.3, 0.3, 0.35, 1.0), 2.0);

        let t = self.normalized();
        let filled = Rect::new(area.x, track_y, area.width * t, track_height);
        painter.fill_rect(filled, Color::rgba(0.2, 0.5, 1.0, 1.0), 2.0);

        let thumb_radius = 8.0;
        let thumb_x = area.x + area.width * t;
        let thumb_center = Position::new(thumb_x, area.y + area.height * 0.5);
        painter.fill_circle(thumb_center, thumb_radius, Color::WHITE);
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("Slider", SemanticRole::Input).with_id(&self.id)
    }
}

impl Discoverable for Slider {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new(
            "Slider",
            "A range slider for value selection",
            SemanticRole::Input,
        )
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::RangeEditable {
                min: self.min as f64,
                max: self.max as f64,
                step: self.step.map(|s| s as f64),
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::simple(
            "set_value",
            "Set the slider value",
            true,
        )]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Input
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "value": self.value,
            "min": self.min,
            "max": self.max,
            "normalized": self.normalized(),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "set_value" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_f64()) {
                    self.value = (v as f32).clamp(self.min, self.max);
                    Ok(serde_json::json!({ "value": self.value }))
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
