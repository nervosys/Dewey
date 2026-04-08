//! List and Table widgets with virtualized scrolling support.

use crate::core::{Color, Position, Rect, TextStyle};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, SemanticRole, UiNode, WidgetSchema,
};
use crate::paint::Painter;
use crate::widget::Widget;

/// A scrollable list of items.
pub struct List {
    pub id: String,
    pub items: Vec<String>,
    pub selected: Option<usize>,
    pub scroll_offset: f32,
    pub item_height: f32,
}

impl List {
    pub fn new(id: impl Into<String>, items: Vec<String>) -> Self {
        Self {
            id: id.into(),
            items,
            selected: None,
            scroll_offset: 0.0,
            item_height: 28.0,
        }
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }
}

impl Widget for List {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        painter.fill_rect(area, Color::rgba(0.12, 0.12, 0.15, 1.0), 3.0);

        let style = TextStyle {
            font_size: 14.0,
            color: Color::WHITE,
            ..TextStyle::default()
        };

        let padding = 6.0;
        let visible_start = (self.scroll_offset / self.item_height).floor() as usize;
        let visible_count = (area.height / self.item_height).ceil() as usize + 1;

        for i in visible_start..self.items.len().min(visible_start + visible_count) {
            let y = area.y + i as f32 * self.item_height - self.scroll_offset;
            if y + self.item_height < area.y || y > area.y + area.height {
                continue;
            }

            let item_rect = Rect::new(area.x, y, area.width, self.item_height);

            if self.selected == Some(i) {
                painter.fill_rect(item_rect, Color::rgba(0.2, 0.4, 0.7, 0.5), 0.0);
            }

            let text_color = if self.selected == Some(i) {
                Color::WHITE
            } else {
                style.color
            };

            let text_style = TextStyle {
                color: text_color,
                ..style.clone()
            };

            painter.text(
                Position::new(
                    area.x + padding,
                    y + (self.item_height - style.font_size) * 0.5,
                ),
                &self.items[i],
                &text_style,
            );
        }
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("List", SemanticRole::Selection).with_id(&self.id)
    }
}

impl Discoverable for List {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new(
            "List",
            "A scrollable list of items",
            SemanticRole::Selection,
        )
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::Scrollable {
                vertical: true,
                horizontal: false,
            },
            AgentCapability::Selectable {
                multi_select: false,
                item_count: self.items.len(),
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::simple("select", "Select an item by index", true),
            AgentAction::simple("scroll", "Scroll the list", true),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Selection
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "item_count": self.items.len(),
            "selected": self.selected,
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "select" => {
                if let Some(idx) = params.get("index").and_then(|v| v.as_u64()) {
                    let idx = idx as usize;
                    if idx < self.items.len() {
                        self.selected = Some(idx);
                        Ok(serde_json::json!({ "selected": idx }))
                    } else {
                        Err("Index out of range".into())
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

/// A data table with column headers and rows.
pub struct Table {
    pub id: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub selected_row: Option<usize>,
    pub scroll_offset: f32,
    pub row_height: f32,
    pub header_height: f32,
}

impl Table {
    pub fn new(id: impl Into<String>, columns: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        Self {
            id: id.into(),
            columns,
            rows,
            selected_row: None,
            scroll_offset: 0.0,
            row_height: 28.0,
            header_height: 32.0,
        }
    }
}

impl Widget for Table {
    fn draw(&self, painter: &mut dyn Painter, area: Rect) {
        painter.fill_rect(area, Color::rgba(0.1, 0.1, 0.13, 1.0), 3.0);

        let col_count = self.columns.len().max(1);
        let col_width = area.width / col_count as f32;

        let header_style = TextStyle {
            font_size: 13.0,
            color: Color::rgba(0.7, 0.7, 0.8, 1.0),
            ..TextStyle::default()
        };

        // Header
        let header_rect = Rect::new(area.x, area.y, area.width, self.header_height);
        painter.fill_rect(header_rect, Color::rgba(0.15, 0.15, 0.2, 1.0), 0.0);

        for (i, col) in self.columns.iter().enumerate() {
            painter.text(
                Position::new(
                    area.x + i as f32 * col_width + 6.0,
                    area.y + (self.header_height - header_style.font_size) * 0.5,
                ),
                col,
                &header_style,
            );
        }

        // Rows
        let row_style = TextStyle {
            font_size: 13.0,
            color: Color::WHITE,
            ..TextStyle::default()
        };

        let body_y = area.y + self.header_height;
        for (ri, row) in self.rows.iter().enumerate() {
            let y = body_y + ri as f32 * self.row_height - self.scroll_offset;
            if y + self.row_height < body_y || y > area.y + area.height {
                continue;
            }

            if self.selected_row == Some(ri) {
                let row_rect = Rect::new(area.x, y, area.width, self.row_height);
                painter.fill_rect(row_rect, Color::rgba(0.2, 0.4, 0.7, 0.4), 0.0);
            }

            for (ci, cell) in row.iter().enumerate().take(col_count) {
                painter.text(
                    Position::new(
                        area.x + ci as f32 * col_width + 6.0,
                        y + (self.row_height - row_style.font_size) * 0.5,
                    ),
                    cell,
                    &row_style,
                );
            }
        }
    }

    fn ui_node(&self) -> UiNode {
        UiNode::new("Table", SemanticRole::DataVisualization).with_id(&self.id)
    }
}

impl Discoverable for Table {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new(
            "Table",
            "A data table with headers and rows",
            SemanticRole::DataVisualization,
        )
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::Scrollable {
                vertical: true,
                horizontal: false,
            },
            AgentCapability::Selectable {
                multi_select: false,
                item_count: self.rows.len(),
            },
            AgentCapability::Sortable {
                columns: self.columns.clone(),
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::simple("select_row", "Select a table row", true),
            AgentAction::simple("scroll", "Scroll the table", true),
            AgentAction::simple("sort", "Sort by column", false),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::DataVisualization
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "columns": self.columns,
            "row_count": self.rows.len(),
            "selected_row": self.selected_row,
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "select_row" => {
                if let Some(idx) = params.get("index").and_then(|v| v.as_u64()) {
                    let idx = idx as usize;
                    if idx < self.rows.len() {
                        self.selected_row = Some(idx);
                        Ok(serde_json::json!({ "selected_row": idx }))
                    } else {
                        Err("Index out of range".into())
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
