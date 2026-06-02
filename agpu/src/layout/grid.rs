//! Grid layout — 2D grid with row/column spans and gap spacing.

use crate::core::Rect;
use crate::layout::{DesiredSize, Layout, SizeConstraints};

/// Span configuration for a grid item.
#[derive(Debug, Clone, Copy)]
pub struct GridSpan {
    pub col_span: usize,
    pub row_span: usize,
}

impl Default for GridSpan {
    fn default() -> Self {
        Self {
            col_span: 1,
            row_span: 1,
        }
    }
}

/// Per-child grid placement.
#[derive(Debug, Clone)]
pub struct GridItem {
    pub index: usize,
    pub span: GridSpan,
}

impl GridItem {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            span: GridSpan::default(),
        }
    }

    pub fn col_span(mut self, span: usize) -> Self {
        self.span.col_span = span;
        self
    }

    pub fn row_span(mut self, span: usize) -> Self {
        self.span.row_span = span;
        self
    }
}

/// A 2D grid layout with configurable columns, rows, and gaps.
pub struct GridLayout {
    columns: usize,
    rows: usize,
    gap: f32,
    items: Vec<GridItem>,
}

impl GridLayout {
    pub fn new(columns: usize, rows: usize) -> Self {
        Self {
            columns: columns.max(1),
            rows: rows.max(1),
            gap: 0.0,
            items: Vec::new(),
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn item(mut self, item: GridItem) -> Self {
        self.items.push(item);
        self
    }
}

impl Layout for GridLayout {
    fn measure(&self, children: &[DesiredSize], constraints: SizeConstraints) -> DesiredSize {
        if children.is_empty() {
            return DesiredSize::zero();
        }
        let max_child_w = children.iter().map(|c| c.width).fold(0.0f32, f32::max);
        let max_child_h = children.iter().map(|c| c.height).fold(0.0f32, f32::max);

        let total_w =
            max_child_w * self.columns as f32 + self.gap * (self.columns as f32 - 1.0).max(0.0);
        let total_h = max_child_h * self.rows as f32 + self.gap * (self.rows as f32 - 1.0).max(0.0);

        let (w, h) = constraints.clamp(total_w, total_h);
        DesiredSize::new(w, h)
    }

    fn arrange(&self, children: &[DesiredSize], area: Rect) -> Vec<Rect> {
        if children.is_empty() {
            return vec![];
        }

        let col_w =
            (area.width - self.gap * (self.columns as f32 - 1.0).max(0.0)) / self.columns as f32;
        let row_h = (area.height - self.gap * (self.rows as f32 - 1.0).max(0.0)) / self.rows as f32;

        // If we have explicit items, use them for span info
        if !self.items.is_empty() {
            return self.arrange_with_items(children, area, col_w, row_h);
        }

        // Default: fill row-by-row, each child gets one cell
        let mut rects = Vec::with_capacity(children.len());
        for (i, _child) in children.iter().enumerate() {
            let col = i % self.columns;
            let row = i / self.columns;
            let x = area.x + col as f32 * (col_w + self.gap);
            let y = area.y + row as f32 * (row_h + self.gap);
            rects.push(Rect::new(x, y, col_w, row_h));
        }
        rects
    }
}

impl GridLayout {
    fn arrange_with_items(
        &self,
        children: &[DesiredSize],
        area: Rect,
        col_w: f32,
        row_h: f32,
    ) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(children.len());
        let mut col = 0usize;
        let mut row = 0usize;

        for (i, _child) in children.iter().enumerate() {
            let span = self
                .items
                .iter()
                .find(|it| it.index == i)
                .map(|it| it.span)
                .unwrap_or_default();

            let cs = span.col_span.min(self.columns - col);
            let rs = span.row_span.min(self.rows - row);

            let x = area.x + col as f32 * (col_w + self.gap);
            let y = area.y + row as f32 * (row_h + self.gap);
            let w = col_w * cs as f32 + self.gap * (cs as f32 - 1.0).max(0.0);
            let h = row_h * rs as f32 + self.gap * (rs as f32 - 1.0).max(0.0);

            rects.push(Rect::new(x, y, w, h));

            col += cs;
            if col >= self.columns {
                col = 0;
                row += 1;
            }
        }
        rects
    }
}
