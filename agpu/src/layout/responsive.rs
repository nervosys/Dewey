//! Responsive layout — adapt layout to window size via breakpoints.

use crate::core::Rect;
use crate::layout::flex::FlexLayout;
use crate::layout::{DesiredSize, Layout, SizeConstraints};

/// A breakpoint defines a minimum width at which a layout activates.
pub struct Breakpoint {
    pub min_width: f32,
    pub layout: FlexLayout,
}

impl Breakpoint {
    pub fn new(min_width: f32, layout: FlexLayout) -> Self {
        Self { min_width, layout }
    }
}

/// Selects a layout based on the available width compared to breakpoints.
///
/// Breakpoints are checked from largest `min_width` to smallest.
/// The first breakpoint whose `min_width <= area.width` is used.
pub struct ResponsiveLayout {
    breakpoints: Vec<Breakpoint>,
}

impl ResponsiveLayout {
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
        }
    }

    pub fn breakpoint(mut self, bp: Breakpoint) -> Self {
        self.breakpoints.push(bp);
        // Keep sorted descending by min_width for matching
        self.breakpoints
            .sort_by(|a, b| b.min_width.partial_cmp(&a.min_width).unwrap());
        self
    }

    fn active_layout(&self, width: f32) -> Option<&FlexLayout> {
        self.breakpoints
            .iter()
            .find(|bp| width >= bp.min_width)
            .map(|bp| &bp.layout)
    }
}

impl Default for ResponsiveLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl Layout for ResponsiveLayout {
    fn measure(&self, children: &[DesiredSize], constraints: SizeConstraints) -> DesiredSize {
        // Use the smallest breakpoint's layout for measurement
        if let Some(bp) = self.breakpoints.last() {
            bp.layout.measure(children, constraints)
        } else {
            DesiredSize::zero()
        }
    }

    fn arrange(&self, children: &[DesiredSize], area: Rect) -> Vec<Rect> {
        if let Some(layout) = self.active_layout(area.width) {
            layout.arrange(children, area)
        } else {
            // Fallback: stack vertically
            let mut rects = Vec::with_capacity(children.len());
            let mut y = area.y;
            for child in children {
                rects.push(Rect::new(area.x, y, child.width, child.height));
                y += child.height;
            }
            rects
        }
    }
}
