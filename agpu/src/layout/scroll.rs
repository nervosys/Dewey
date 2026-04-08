//! Scroll container — overflow handling with virtual scroll.

use crate::core::Rect;
use crate::layout::{DesiredSize, Layout, SizeConstraints};

/// Which axes can scroll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Vertical,
    Horizontal,
    Both,
}

/// A scroll container that offsets content and supports virtual scrolling.
pub struct ScrollContainer {
    direction: ScrollDirection,
    offset_x: f32,
    offset_y: f32,
}

impl ScrollContainer {
    pub fn new(direction: ScrollDirection) -> Self {
        Self {
            direction,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    /// Set scroll offset (combined x/y based on direction).
    pub fn offset(mut self, offset: f32) -> Self {
        match self.direction {
            ScrollDirection::Vertical => self.offset_y = offset,
            ScrollDirection::Horizontal => self.offset_x = offset,
            ScrollDirection::Both => {
                self.offset_x = offset;
                self.offset_y = offset;
            }
        }
        self
    }

    pub fn offset_xy(mut self, x: f32, y: f32) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }

    /// The viewport rect (visible portion) given the area.
    pub fn viewport(&self, area: Rect) -> Rect {
        area
    }

    /// Maximum scroll offset given content and viewport sizes.
    pub fn max_offset(&self, content: DesiredSize, viewport: Rect) -> (f32, f32) {
        let max_x = (content.width - viewport.width).max(0.0);
        let max_y = (content.height - viewport.height).max(0.0);
        (max_x, max_y)
    }
}

impl Layout for ScrollContainer {
    fn measure(&self, children: &[DesiredSize], constraints: SizeConstraints) -> DesiredSize {
        // Content size = union of all children (stacked vertically)
        if children.is_empty() {
            return DesiredSize::zero();
        }
        let width = children.iter().map(|c| c.width).fold(0.0f32, f32::max);
        let height: f32 = children.iter().map(|c| c.height).sum();
        let (w, h) = constraints.clamp(width, height);
        DesiredSize::new(w, h)
    }

    fn arrange(&self, children: &[DesiredSize], area: Rect) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(children.len());
        let mut y = area.y - self.offset_y;
        for child in children {
            let x = area.x - self.offset_x;
            let w = match self.direction {
                ScrollDirection::Horizontal | ScrollDirection::Both => child.width,
                ScrollDirection::Vertical => child.width.max(area.width),
            };
            rects.push(Rect::new(x, y, w, child.height));
            y += child.height;
        }
        rects
    }
}
