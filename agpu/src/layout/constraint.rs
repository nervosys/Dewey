//! Constraint-based layout — auto-layout with min/max size constraints per child.

use crate::core::Rect;
use crate::layout::{DesiredSize, Layout, SizeConstraints};

/// A min/max range for a single dimension.
#[derive(Debug, Clone, Copy)]
pub struct MinMax {
    pub min: f32,
    pub max: f32,
}

impl MinMax {
    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub fn clamp(&self, value: f32) -> f32 {
        value.clamp(self.min, self.max)
    }
}

/// Constraint rule for a specific child by index.
#[derive(Debug, Clone)]
pub struct ConstraintRule {
    pub index: usize,
    pub width: Option<MinMax>,
    pub height: Option<MinMax>,
}

impl ConstraintRule {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            width: None,
            height: None,
        }
    }

    pub fn width(mut self, constraint: MinMax) -> Self {
        self.width = Some(constraint);
        self
    }

    pub fn height(mut self, constraint: MinMax) -> Self {
        self.height = Some(constraint);
        self
    }
}

/// Layout that applies min/max constraints to each child.
pub struct ConstraintLayout {
    rules: Vec<ConstraintRule>,
}

impl ConstraintLayout {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn rule(mut self, rule: ConstraintRule) -> Self {
        self.rules.push(rule);
        self
    }

    fn constrain(&self, index: usize, width: f32, height: f32) -> (f32, f32) {
        if let Some(rule) = self.rules.iter().find(|r| r.index == index) {
            let w = rule.width.as_ref().map_or(width, |c| c.clamp(width));
            let h = rule.height.as_ref().map_or(height, |c| c.clamp(height));
            (w, h)
        } else {
            (width, height)
        }
    }
}

impl Default for ConstraintLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl Layout for ConstraintLayout {
    fn measure(&self, children: &[DesiredSize], constraints: SizeConstraints) -> DesiredSize {
        if children.is_empty() {
            return DesiredSize::zero();
        }
        // Stack vertically, apply constraints
        let mut total_h = 0.0f32;
        let mut max_w = 0.0f32;
        for (i, child) in children.iter().enumerate() {
            let (w, h) = self.constrain(i, child.width, child.height);
            max_w = max_w.max(w);
            total_h += h;
        }
        let (w, h) = constraints.clamp(max_w, total_h);
        DesiredSize::new(w, h)
    }

    fn arrange(&self, children: &[DesiredSize], area: Rect) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(children.len());
        let mut y = area.y;
        for (i, child) in children.iter().enumerate() {
            let (w, h) = self.constrain(i, child.width, child.height);
            rects.push(Rect::new(area.x, y, w, h));
            y += h;
        }
        rects
    }
}
