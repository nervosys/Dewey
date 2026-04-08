//! Property-based tests for core types and layout engine.

use dewey::core::{Position, Rect};
use dewey::layout::{Constraint, Direction, Layout};
use proptest::prelude::*;

// ── Rect property tests ─────────────────────────────────────────────

proptest! {
    #[test]
    fn rect_contains_its_own_center(
        x in -1000.0f32..1000.0,
        y in -1000.0f32..1000.0,
        w in 0.1f32..2000.0,
        h in 0.1f32..2000.0,
    ) {
        let r = Rect::new(x, y, w, h);
        let center = Position::new(x + w / 2.0, y + h / 2.0);
        prop_assert!(r.contains(center));
    }

    #[test]
    fn rect_dimensions_non_negative(
        x in -1000.0f32..1000.0,
        y in -1000.0f32..1000.0,
        w in 0.0f32..2000.0,
        h in 0.0f32..2000.0,
    ) {
        let r = Rect::new(x, y, w, h);
        prop_assert!(r.width >= 0.0);
        prop_assert!(r.height >= 0.0);
        prop_assert!(r.width * r.height >= 0.0);
    }

    #[test]
    fn rect_inner_margin_shrinks(
        x in 0.0f32..500.0,
        y in 0.0f32..500.0,
        w in 20.0f32..1000.0,
        h in 20.0f32..1000.0,
        m in 0.0f32..10.0,
    ) {
        let r = Rect::new(x, y, w, h);
        let margin = dewey::core::Margin::uniform(m);
        let inner = r.inner(&margin);
        prop_assert!(inner.width <= r.width, "inner width {} > original {}", inner.width, r.width);
        prop_assert!(inner.height <= r.height, "inner height {} > original {}", inner.height, r.height);
    }
}

// ── Layout property tests ───────────────────────────────────────────

proptest! {
    #[test]
    fn layout_split_produces_correct_count(
        n in 1usize..10,
        w in 100.0f32..2000.0,
        h in 100.0f32..2000.0,
    ) {
        let constraints: Vec<Constraint> = (0..n)
            .map(|_| Constraint::Percentage(100.0 / n as f32))
            .collect();
        let layout = Layout::new(Direction::Horizontal, constraints);
        let area = Rect::new(0.0, 0.0, w, h);
        let chunks = layout.split(area);
        prop_assert_eq!(chunks.len(), n);
    }

    #[test]
    fn layout_chunks_fit_within_area(
        n in 1usize..8,
        w in 100.0f32..2000.0,
        h in 100.0f32..2000.0,
    ) {
        let constraints: Vec<Constraint> = (0..n)
            .map(|_| Constraint::Length(w / n as f32))
            .collect();
        let layout = Layout::new(Direction::Horizontal, constraints);
        let area = Rect::new(0.0, 0.0, w, h);
        let chunks = layout.split(area);
        for chunk in &chunks {
            // Each chunk should not exceed the total area
            prop_assert!(chunk.x >= area.x - 0.01);
            prop_assert!(chunk.y >= area.y - 0.01);
            prop_assert!(chunk.width >= 0.0);
            prop_assert!(chunk.height >= 0.0);
        }
    }

    #[test]
    fn layout_fill_distributes_space(
        w in 100.0f32..2000.0,
        h in 100.0f32..2000.0,
    ) {
        let layout = Layout::new(
            Direction::Horizontal,
            vec![Constraint::Fill(1.0), Constraint::Fill(1.0)],
        );
        let area = Rect::new(0.0, 0.0, w, h);
        let chunks = layout.split(area);
        prop_assert_eq!(chunks.len(), 2);
        // Equal-weight fills should produce approximately equal widths
        let diff = (chunks[0].width - chunks[1].width).abs();
        prop_assert!(diff < 1.0, "Fill widths too different: {} vs {}", chunks[0].width, chunks[1].width);
    }
}
