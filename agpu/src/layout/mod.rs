//! Layout engine for agpu — composable layout algorithms.
//!
//! Provides [`Layout`] trait and concrete implementations for flexbox,
//! grid, constraint-based, scroll container, and responsive layouts.

mod constraint;
mod flex;
mod grid;
mod responsive;
mod scroll;

pub use constraint::{ConstraintLayout, ConstraintRule, MinMax};
pub use flex::{Align, CrossAlign, FlexItem, FlexLayout, FlexWrap, JustifyContent};
pub use grid::{GridItem, GridLayout, GridSpan};
pub use responsive::{Breakpoint, ResponsiveLayout};
pub use scroll::{ScrollContainer, ScrollDirection};

use crate::core::Rect;

/// Size constraints passed to layout algorithms.
#[derive(Debug, Clone, Copy)]
pub struct SizeConstraints {
    pub min_width: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub max_height: f32,
}

impl SizeConstraints {
    /// Unconstrained — no minimum, infinite maximum.
    pub fn unconstrained() -> Self {
        Self {
            min_width: 0.0,
            min_height: 0.0,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
        }
    }

    /// Fixed-size constraint.
    pub fn fixed(width: f32, height: f32) -> Self {
        Self {
            min_width: width,
            min_height: height,
            max_width: width,
            max_height: height,
        }
    }

    /// Constrain to the given available area.
    pub fn from_rect(area: Rect) -> Self {
        Self {
            min_width: 0.0,
            min_height: 0.0,
            max_width: area.width,
            max_height: area.height,
        }
    }

    /// Clamp a size to these constraints.
    pub fn clamp(&self, width: f32, height: f32) -> (f32, f32) {
        (
            width.clamp(self.min_width, self.max_width),
            height.clamp(self.min_height, self.max_height),
        )
    }
}

impl Default for SizeConstraints {
    fn default() -> Self {
        Self::unconstrained()
    }
}

/// A node's desired size as reported by [`Layout::measure`].
#[derive(Debug, Clone, Copy, Default)]
pub struct DesiredSize {
    pub width: f32,
    pub height: f32,
}

impl DesiredSize {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
}

/// Trait implemented by all layout algorithms.
///
/// A layout receives a set of child sizes and an available area, then
/// computes the final [`Rect`] for each child.
pub trait Layout {
    /// Compute the desired size of this layout given child sizes and constraints.
    fn measure(&self, children: &[DesiredSize], constraints: SizeConstraints) -> DesiredSize;

    /// Arrange children within the given area, returning a positioned rect per child.
    fn arrange(&self, children: &[DesiredSize], area: Rect) -> Vec<Rect>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_constraints_unconstrained() {
        let c = SizeConstraints::unconstrained();
        assert_eq!(c.min_width, 0.0);
        assert!(c.max_width.is_infinite());
    }

    #[test]
    fn size_constraints_fixed() {
        let c = SizeConstraints::fixed(100.0, 50.0);
        assert_eq!(c.min_width, 100.0);
        assert_eq!(c.max_width, 100.0);
        assert_eq!(c.min_height, 50.0);
        assert_eq!(c.max_height, 50.0);
    }

    #[test]
    fn size_constraints_clamp() {
        let c = SizeConstraints {
            min_width: 10.0,
            min_height: 10.0,
            max_width: 100.0,
            max_height: 100.0,
        };
        assert_eq!(c.clamp(5.0, 200.0), (10.0, 100.0));
        assert_eq!(c.clamp(50.0, 50.0), (50.0, 50.0));
    }

    #[test]
    fn desired_size_zero() {
        let s = DesiredSize::zero();
        assert_eq!(s.width, 0.0);
        assert_eq!(s.height, 0.0);
    }

    #[test]
    fn size_constraints_from_rect() {
        let r = Rect::new(10.0, 20.0, 300.0, 400.0);
        let c = SizeConstraints::from_rect(r);
        assert_eq!(c.max_width, 300.0);
        assert_eq!(c.max_height, 400.0);
    }

    // ── Flex layout tests ───────────────────────────────────────────

    #[test]
    fn flex_row_even_distribution() {
        let flex = FlexLayout::row();
        let children = vec![
            DesiredSize::new(50.0, 30.0),
            DesiredSize::new(50.0, 30.0),
            DesiredSize::new(50.0, 30.0),
        ];
        let area = Rect::new(0.0, 0.0, 300.0, 100.0);
        let rects = flex.arrange(&children, area);
        assert_eq!(rects.len(), 3);
        assert!((rects[0].x - 0.0).abs() < 0.01);
        assert!((rects[1].x - 50.0).abs() < 0.01);
        assert!((rects[2].x - 100.0).abs() < 0.01);
    }

    #[test]
    fn flex_column_stacks_vertically() {
        let flex = FlexLayout::column();
        let children = vec![DesiredSize::new(100.0, 40.0), DesiredSize::new(100.0, 60.0)];
        let area = Rect::new(0.0, 0.0, 200.0, 200.0);
        let rects = flex.arrange(&children, area);
        assert_eq!(rects.len(), 2);
        assert!((rects[0].y - 0.0).abs() < 0.01);
        assert!((rects[1].y - 40.0).abs() < 0.01);
    }

    #[test]
    fn flex_spacing() {
        let flex = FlexLayout::row().spacing(10.0);
        let children = vec![DesiredSize::new(50.0, 30.0), DesiredSize::new(50.0, 30.0)];
        let area = Rect::new(0.0, 0.0, 200.0, 100.0);
        let rects = flex.arrange(&children, area);
        assert!((rects[1].x - 60.0).abs() < 0.01); // 50 + 10 spacing
    }

    #[test]
    fn flex_measure_row() {
        let flex = FlexLayout::row().spacing(5.0);
        let children = vec![DesiredSize::new(40.0, 20.0), DesiredSize::new(60.0, 30.0)];
        let size = flex.measure(&children, SizeConstraints::unconstrained());
        assert!((size.width - 105.0).abs() < 0.01); // 40 + 5 + 60
        assert!((size.height - 30.0).abs() < 0.01); // max height
    }

    #[test]
    fn flex_wrap_wraps_to_next_line() {
        let flex = FlexLayout::row().wrap(FlexWrap::Wrap);
        let children = vec![
            DesiredSize::new(60.0, 30.0),
            DesiredSize::new(60.0, 30.0),
            DesiredSize::new(60.0, 30.0),
        ];
        let area = Rect::new(0.0, 0.0, 130.0, 200.0);
        let rects = flex.arrange(&children, area);
        // First two fit in row (60+60=120 <= 130), third wraps
        assert!((rects[2].y - 30.0).abs() < 0.01);
    }

    // ── Grid layout tests ───────────────────────────────────────────

    #[test]
    fn grid_basic_layout() {
        let grid = GridLayout::new(2, 2);
        let children = vec![
            DesiredSize::new(50.0, 50.0),
            DesiredSize::new(50.0, 50.0),
            DesiredSize::new(50.0, 50.0),
            DesiredSize::new(50.0, 50.0),
        ];
        let area = Rect::new(0.0, 0.0, 200.0, 200.0);
        let rects = grid.arrange(&children, area);
        assert_eq!(rects.len(), 4);
        assert!((rects[0].width - 100.0).abs() < 0.01);
        assert!((rects[0].height - 100.0).abs() < 0.01);
    }

    #[test]
    fn grid_measure() {
        let grid = GridLayout::new(3, 2).gap(4.0);
        let children = vec![
            DesiredSize::new(30.0, 20.0),
            DesiredSize::new(30.0, 20.0),
            DesiredSize::new(30.0, 20.0),
            DesiredSize::new(30.0, 20.0),
            DesiredSize::new(30.0, 20.0),
            DesiredSize::new(30.0, 20.0),
        ];
        let size = grid.measure(&children, SizeConstraints::unconstrained());
        assert!((size.width - 98.0).abs() < 0.01); // 3*30 + 2*4
        assert!((size.height - 44.0).abs() < 0.01); // 2*20 + 1*4
    }

    #[test]
    fn grid_with_spans() {
        let grid = GridLayout::new(3, 2)
            .item(GridItem::new(0).col_span(2))
            .item(GridItem::new(1));
        let children = vec![
            DesiredSize::new(50.0, 30.0),
            DesiredSize::new(50.0, 30.0),
        ];
        let area = Rect::new(0.0, 0.0, 300.0, 200.0);
        let rects = grid.arrange(&children, area);
        // First item spans 2 columns: width should be ~200
        assert!((rects[0].width - 200.0).abs() < 0.01);
    }

    // ── Constraint layout tests ─────────────────────────────────────

    #[test]
    fn constraint_layout_min_max() {
        let layout = ConstraintLayout::new()
            .rule(ConstraintRule::new(0).width(MinMax::new(50.0, 200.0)));
        let children = vec![DesiredSize::new(300.0, 40.0)];
        let area = Rect::new(0.0, 0.0, 400.0, 400.0);
        let rects = layout.arrange(&children, area);
        assert!((rects[0].width - 200.0).abs() < 0.01); // clamped to max
    }

    #[test]
    fn constraint_layout_unconstrained_children() {
        let layout = ConstraintLayout::new();
        let children = vec![DesiredSize::new(80.0, 60.0)];
        let area = Rect::new(10.0, 20.0, 400.0, 300.0);
        let rects = layout.arrange(&children, area);
        assert!((rects[0].x - 10.0).abs() < 0.01);
        assert!((rects[0].y - 20.0).abs() < 0.01);
        assert!((rects[0].width - 80.0).abs() < 0.01);
    }

    // ── Scroll container tests ──────────────────────────────────────

    #[test]
    fn scroll_container_clips_content() {
        let scroll = ScrollContainer::new(ScrollDirection::Vertical);
        let children = vec![DesiredSize::new(100.0, 500.0)];
        let area = Rect::new(0.0, 0.0, 100.0, 200.0);
        let rects = scroll.arrange(&children, area);
        assert!((rects[0].height - 500.0).abs() < 0.01); // content keeps full height
    }

    #[test]
    fn scroll_with_offset() {
        let scroll = ScrollContainer::new(ScrollDirection::Vertical).offset(50.0);
        let children = vec![DesiredSize::new(100.0, 500.0)];
        let area = Rect::new(0.0, 0.0, 100.0, 200.0);
        let rects = scroll.arrange(&children, area);
        assert!((rects[0].y - (-50.0)).abs() < 0.01); // shifted up
    }

    #[test]
    fn scroll_measure_passes_through() {
        let scroll = ScrollContainer::new(ScrollDirection::Both);
        let children = vec![DesiredSize::new(800.0, 600.0)];
        let size = scroll.measure(&children, SizeConstraints::unconstrained());
        assert!((size.width - 800.0).abs() < 0.01);
        assert!((size.height - 600.0).abs() < 0.01);
    }

    // ── Responsive layout tests ─────────────────────────────────────

    #[test]
    fn responsive_selects_correct_breakpoint() {
        let layout = ResponsiveLayout::new()
            .breakpoint(Breakpoint::new(0.0, FlexLayout::column()))
            .breakpoint(Breakpoint::new(600.0, FlexLayout::row()));
        let children = vec![DesiredSize::new(50.0, 30.0), DesiredSize::new(50.0, 30.0)];
        // Wide area → row layout
        let wide = Rect::new(0.0, 0.0, 800.0, 400.0);
        let rects = layout.arrange(&children, wide);
        assert!((rects[1].x - 50.0).abs() < 0.01); // row: side by side

        // Narrow area → column layout
        let narrow = Rect::new(0.0, 0.0, 400.0, 400.0);
        let rects = layout.arrange(&children, narrow);
        assert!((rects[1].y - 30.0).abs() < 0.01); // column: stacked
    }

    #[test]
    fn responsive_measure_uses_widest() {
        let layout = ResponsiveLayout::new()
            .breakpoint(Breakpoint::new(0.0, FlexLayout::column()));
        let children = vec![DesiredSize::new(50.0, 30.0)];
        let size = layout.measure(&children, SizeConstraints::unconstrained());
        assert!((size.width - 50.0).abs() < 0.01);
    }
}
