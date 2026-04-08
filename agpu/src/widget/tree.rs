//! TreeView widget with hierarchical expand/collapse nodes.

use crate::core::{Color, Position, Rect, TextStyle};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, SemanticRole, UiNode, WidgetSchema,
};
use crate::paint::Painter;
use crate::widget::Widget;

/// A node in a tree hierarchy.
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
}

impl TreeNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children: Vec::new(),
            expanded: false,
        }
    }

    pub fn child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Count the total visible (expanded) nodes starting from this node.
    fn visible_count(&self) -> usize {
        let mut count = 1; // self
        if self.expanded {
            for child in &self.children {
                count += child.visible_count();
            }
        }
        count
    }

    /// Recursively draw this node and its visible children.
    fn draw_recursive(
        &self,
        painter: &mut dyn Painter,
        x: f32,
        y: &mut f32,
        row_height: f32,
        area: &Rect,
        indent: f32,
    ) {
        if *y + row_height < area.y || *y > area.y + area.height {
            *y += row_height;
            if self.expanded {
                for child in &self.children {
                    child.draw_recursive(painter, x + indent, y, row_height, area, indent);
                }
            }
            return;
        }

        let style = TextStyle {
            font_size: 13.0,
            color: Color::WHITE,
            ..TextStyle::default()
        };

        // Expand/collapse arrow if has children
        if !self.children.is_empty() {
            let arrow = if self.expanded { "▾" } else { "▸" };
            painter.text(
                Position::new(x, *y + (row_height - style.font_size) * 0.5),
                arrow,
                &style,
            );
        }

        painter.text(
            Position::new(x + 16.0, *y + (row_height - style.font_size) * 0.5),
            &self.label,
            &style,
        );

        *y += row_height;

        if self.expanded {
            for child in &self.children {
                child.draw_recursive(painter, x + indent, y, row_height, area, indent);
            }
        }
    }
}

/// A hierarchical tree view widget.
pub struct TreeView {
    pub id: String,
    pub root: TreeNode,
}

impl TreeView {
    pub fn new(id: impl Into<String>, root: TreeNode) -> Self {
        Self {
            id: id.into(),
            root,
        }
    }
}

impl Widget for TreeView {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        painter.fill_rect(area, Color::rgba(0.1, 0.1, 0.13, 1.0), 3.0);

        let row_height = 24.0;
        let indent = 18.0;
        let mut y = area.y;

        self.root
            .draw_recursive(painter, area.x + 4.0, &mut y, row_height, &area, indent);
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("TreeView", SemanticRole::TreeNode).with_id(&self.id)
    }
}

impl Discoverable for TreeView {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new(
            "TreeView",
            "A hierarchical tree view",
            SemanticRole::TreeNode,
        )
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::Expandable {
                expanded: self.root.expanded,
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::simple("expand", "Expand a node", true),
            AgentAction::simple("collapse", "Collapse a node", true),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::TreeNode
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "root_label": self.root.label,
            "root_expanded": self.root.expanded,
            "visible_nodes": self.root.visible_count(),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "expand" => {
                self.root.expanded = true;
                Ok(serde_json::json!({ "expanded": true }))
            }
            "collapse" => {
                self.root.expanded = false;
                Ok(serde_json::json!({ "expanded": false }))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}
