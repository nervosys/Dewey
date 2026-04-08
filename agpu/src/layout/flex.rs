//! Flexbox layout — row/column with alignment, wrapping, and spacing.

use crate::core::Rect;
use crate::layout::{DesiredSize, Layout, SizeConstraints};

/// Main axis direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    /// Lay out children in a horizontal row.
    Row,
    /// Lay out children in a vertical column.
    Column,
}

/// How to distribute remaining space on the main axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Cross-axis alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossAlign {
    Start,
    End,
    Center,
    Stretch,
}

/// Wrapping behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
}

/// Per-child flex configuration.
#[derive(Debug, Clone, Copy)]
pub struct FlexItem {
    pub grow: f32,
    pub shrink: f32,
    pub basis: Option<f32>,
}

impl Default for FlexItem {
    fn default() -> Self {
        Self {
            grow: 0.0,
            shrink: 1.0,
            basis: None,
        }
    }
}

impl FlexItem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grow(mut self, grow: f32) -> Self {
        self.grow = grow;
        self
    }
}

/// Flexbox layout container.
pub struct FlexLayout {
    direction: Align,
    justify: JustifyContent,
    cross_align: CrossAlign,
    wrap: FlexWrap,
    spacing: f32,
}

impl FlexLayout {
    pub fn row() -> Self {
        Self {
            direction: Align::Row,
            justify: JustifyContent::Start,
            cross_align: CrossAlign::Start,
            wrap: FlexWrap::NoWrap,
            spacing: 0.0,
        }
    }

    pub fn column() -> Self {
        Self {
            direction: Align::Column,
            justify: JustifyContent::Start,
            cross_align: CrossAlign::Start,
            wrap: FlexWrap::NoWrap,
            spacing: 0.0,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn justify(mut self, justify: JustifyContent) -> Self {
        self.justify = justify;
        self
    }

    pub fn cross_align(mut self, cross_align: CrossAlign) -> Self {
        self.cross_align = cross_align;
        self
    }

    pub fn wrap(mut self, wrap: FlexWrap) -> Self {
        self.wrap = wrap;
        self
    }
}

impl Layout for FlexLayout {
    fn measure(&self, children: &[DesiredSize], constraints: SizeConstraints) -> DesiredSize {
        if children.is_empty() {
            return DesiredSize::zero();
        }
        let total_spacing = self.spacing * (children.len() as f32 - 1.0).max(0.0);
        match self.direction {
            Align::Row => {
                let width: f32 = children.iter().map(|c| c.width).sum::<f32>() + total_spacing;
                let height = children.iter().map(|c| c.height).fold(0.0f32, f32::max);
                let (w, h) = constraints.clamp(width, height);
                DesiredSize::new(w, h)
            }
            Align::Column => {
                let width = children.iter().map(|c| c.width).fold(0.0f32, f32::max);
                let height: f32 = children.iter().map(|c| c.height).sum::<f32>() + total_spacing;
                let (w, h) = constraints.clamp(width, height);
                DesiredSize::new(w, h)
            }
        }
    }

    fn arrange(&self, children: &[DesiredSize], area: Rect) -> Vec<Rect> {
        if children.is_empty() {
            return vec![];
        }

        if self.wrap == FlexWrap::Wrap {
            return self.arrange_wrapped(children, area);
        }

        let mut rects = Vec::with_capacity(children.len());
        match self.direction {
            Align::Row => {
                let total_child_width: f32 = children.iter().map(|c| c.width).sum();
                let total_spacing = self.spacing * (children.len() as f32 - 1.0).max(0.0);
                let remaining = (area.width - total_child_width - total_spacing).max(0.0);
                let (start_offset, inter_spacing) =
                    self.justify_offsets(remaining, children.len());

                let mut x = area.x + start_offset;
                for child in children {
                    let y = self.cross_offset(area.y, area.height, child.height);
                    let h = if self.cross_align == CrossAlign::Stretch {
                        area.height
                    } else {
                        child.height
                    };
                    rects.push(Rect::new(x, y, child.width, h));
                    x += child.width + self.spacing + inter_spacing;
                }
            }
            Align::Column => {
                let total_child_height: f32 = children.iter().map(|c| c.height).sum();
                let total_spacing = self.spacing * (children.len() as f32 - 1.0).max(0.0);
                let remaining = (area.height - total_child_height - total_spacing).max(0.0);
                let (start_offset, inter_spacing) =
                    self.justify_offsets(remaining, children.len());

                let mut y = area.y + start_offset;
                for child in children {
                    let x = self.cross_offset(area.x, area.width, child.width);
                    let w = if self.cross_align == CrossAlign::Stretch {
                        area.width
                    } else {
                        child.width
                    };
                    rects.push(Rect::new(x, y, w, child.height));
                    y += child.height + self.spacing + inter_spacing;
                }
            }
        }
        rects
    }
}

impl FlexLayout {
    fn justify_offsets(&self, remaining: f32, count: usize) -> (f32, f32) {
        match self.justify {
            JustifyContent::Start => (0.0, 0.0),
            JustifyContent::End => (remaining, 0.0),
            JustifyContent::Center => (remaining / 2.0, 0.0),
            JustifyContent::SpaceBetween => {
                if count <= 1 {
                    (0.0, 0.0)
                } else {
                    (0.0, remaining / (count as f32 - 1.0))
                }
            }
            JustifyContent::SpaceAround => {
                let gap = remaining / count as f32;
                (gap / 2.0, gap)
            }
            JustifyContent::SpaceEvenly => {
                let gap = remaining / (count as f32 + 1.0);
                (gap, gap)
            }
        }
    }

    fn cross_offset(&self, area_start: f32, area_size: f32, child_size: f32) -> f32 {
        match self.cross_align {
            CrossAlign::Start | CrossAlign::Stretch => area_start,
            CrossAlign::End => area_start + area_size - child_size,
            CrossAlign::Center => area_start + (area_size - child_size) / 2.0,
        }
    }

    fn arrange_wrapped(&self, children: &[DesiredSize], area: Rect) -> Vec<Rect> {
        let mut rects = vec![Rect::new(0.0, 0.0, 0.0, 0.0); children.len()];

        match self.direction {
            Align::Row => {
                let mut x = area.x;
                let mut y = area.y;
                let mut line_height: f32 = 0.0;

                for (i, child) in children.iter().enumerate() {
                    if x + child.width > area.x + area.width && i > 0 {
                        x = area.x;
                        y += line_height + self.spacing;
                        line_height = 0.0;
                    }
                    rects[i] = Rect::new(x, y, child.width, child.height);
                    x += child.width + self.spacing;
                    line_height = line_height.max(child.height);
                }
            }
            Align::Column => {
                let mut x = area.x;
                let mut y = area.y;
                let mut col_width: f32 = 0.0;

                for (i, child) in children.iter().enumerate() {
                    if y + child.height > area.y + area.height && i > 0 {
                        y = area.y;
                        x += col_width + self.spacing;
                        col_width = 0.0;
                    }
                    rects[i] = Rect::new(x, y, child.width, child.height);
                    y += child.height + self.spacing;
                    col_width = col_width.max(child.width);
                }
            }
        }
        rects
    }
}
