//! Tabs and Panel widgets.

use crate::core::{Color, Position, Rect, TextStyle};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, SemanticRole, UiNode, WidgetSchema,
};
use crate::paint::Painter;
use crate::widget::Widget;

/// A tabbed container — shows one tab's content at a time.
pub struct Tabs {
    pub id: String,
    pub labels: Vec<String>,
    pub active: usize,
}

impl Tabs {
    pub fn new(id: impl Into<String>, labels: Vec<String>, active: usize) -> Self {
        Self {
            id: id.into(),
            labels,
            active,
        }
    }
}

impl Widget for Tabs {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        let tab_height = 32.0;
        let tab_width = if self.labels.is_empty() {
            area.width
        } else {
            area.width / self.labels.len() as f32
        };

        // Tab bar background
        let tab_bar = Rect::new(area.x, area.y, area.width, tab_height);
        painter.fill_rect(tab_bar, Color::rgba(0.15, 0.15, 0.18, 1.0), 0.0);

        let style = TextStyle {
            font_size: 13.0,
            color: Color::WHITE,
            ..TextStyle::default()
        };

        for (i, label) in self.labels.iter().enumerate() {
            let tab_rect = Rect::new(area.x + i as f32 * tab_width, area.y, tab_width, tab_height);

            if i == self.active {
                painter.fill_rect(tab_rect, Color::rgba(0.2, 0.2, 0.25, 1.0), 0.0);
                // Active indicator
                let indicator =
                    Rect::new(tab_rect.x, tab_rect.y + tab_height - 2.0, tab_width, 2.0);
                painter.fill_rect(indicator, Color::rgba(0.3, 0.6, 1.0, 1.0), 0.0);
            }

            let text_color = if i == self.active {
                Color::WHITE
            } else {
                Color::rgba(0.6, 0.6, 0.7, 1.0)
            };

            let tab_style = TextStyle {
                color: text_color,
                ..style.clone()
            };

            let text_size = painter.measure_text(label, &tab_style);
            painter.text(
                Position::new(
                    tab_rect.x + (tab_width - text_size.width) * 0.5,
                    tab_rect.y + (tab_height - text_size.height) * 0.5,
                ),
                label,
                &tab_style,
            );
        }

        // Content area
        let content = Rect::new(
            area.x,
            area.y + tab_height,
            area.width,
            area.height - tab_height,
        );
        painter.fill_rect(content, Color::rgba(0.1, 0.1, 0.13, 1.0), 0.0);
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("Tabs", SemanticRole::Tab).with_id(&self.id)
    }
}

impl Discoverable for Tabs {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new("Tabs", "A tabbed container", SemanticRole::Tab)
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::Selectable {
                multi_select: false,
                item_count: self.labels.len(),
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::simple(
            "select_tab",
            "Switch to a tab by index",
            true,
        )]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Tab
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "active": self.active,
            "tab_count": self.labels.len(),
            "labels": self.labels,
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "select_tab" => {
                if let Some(idx) = params.get("index").and_then(|v| v.as_u64()) {
                    let idx = idx as usize;
                    if idx < self.labels.len() {
                        self.active = idx;
                        Ok(serde_json::json!({ "active": idx }))
                    } else {
                        Err("Tab index out of range".into())
                    }
                } else {
                    Err("Missing 'index' parameter".into())
                }
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

/// A collapsible panel container.
pub struct Panel {
    pub id: String,
    pub title: String,
    pub collapsed: bool,
}

impl Panel {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            collapsed: false,
        }
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }
}

impl Widget for Panel {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        let header_height = 28.0;
        let header = Rect::new(area.x, area.y, area.width, header_height);
        painter.fill_rect(header, Color::rgba(0.18, 0.18, 0.22, 1.0), 3.0);

        let style = TextStyle {
            font_size: 13.0,
            color: Color::WHITE,
            ..TextStyle::default()
        };

        // Collapse indicator
        let arrow = if self.collapsed { "▸" } else { "▾" };
        painter.text(
            Position::new(
                area.x + 8.0,
                area.y + (header_height - style.font_size) * 0.5,
            ),
            arrow,
            &style,
        );

        painter.text(
            Position::new(
                area.x + 24.0,
                area.y + (header_height - style.font_size) * 0.5,
            ),
            &self.title,
            &style,
        );

        if !self.collapsed {
            let content = Rect::new(
                area.x,
                area.y + header_height,
                area.width,
                area.height - header_height,
            );
            painter.fill_rect(content, Color::rgba(0.1, 0.1, 0.13, 1.0), 0.0);
        }
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("Panel", SemanticRole::Container).with_id(&self.id)
    }
}

impl Discoverable for Panel {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new(
            "Panel",
            "A collapsible panel container",
            SemanticRole::Container,
        )
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::Expandable {
                expanded: !self.collapsed,
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::simple("toggle", "Toggle panel collapse", true),
            AgentAction::simple("expand", "Expand the panel", true),
            AgentAction::simple("collapse", "Collapse the panel", true),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Container
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "title": self.title,
            "collapsed": self.collapsed,
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "toggle" => {
                self.collapsed = !self.collapsed;
                Ok(serde_json::json!({ "collapsed": self.collapsed }))
            }
            "expand" => {
                self.collapsed = false;
                Ok(serde_json::json!({ "collapsed": false }))
            }
            "collapse" => {
                self.collapsed = true;
                Ok(serde_json::json!({ "collapsed": true }))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}
