//! Layout engine with constraint-based space allocation.
//!
//! Provides flexbox-inspired layout primitives for GUI widget arrangement.

use crate::core::rect::{Margin, Rect};

/// Layout direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Direction {
    #[default]
    Vertical,
    Horizontal,
}

/// Size constraint for layout segments.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Constraint {
    /// Fixed length in logical pixels.
    Length(f32),
    /// Percentage of available space (0.0 - 100.0).
    Percentage(f32),
    /// Minimum size.
    Min(f32),
    /// Maximum size.
    Max(f32),
    /// Ratio of available space (numerator/denominator).
    Ratio(u32, u32),
    /// Fill remaining space proportionally (weight).
    Fill(f32),
}

/// How to distribute excess space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Flex {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Layout builder.
#[derive(Debug, Clone)]
pub struct Layout {
    direction: Direction,
    constraints: Vec<Constraint>,
    margin: Margin,
    flex: Flex,
    spacing: f32,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            direction: Direction::Vertical,
            constraints: Vec::new(),
            margin: Margin::ZERO,
            flex: Flex::Start,
            spacing: 0.0,
        }
    }
}

impl Layout {
    pub fn new(direction: Direction, constraints: impl Into<Vec<Constraint>>) -> Self {
        Self {
            direction,
            constraints: constraints.into(),
            ..Default::default()
        }
    }

    pub fn vertical(constraints: impl Into<Vec<Constraint>>) -> Self {
        Self::new(Direction::Vertical, constraints)
    }

    pub fn horizontal(constraints: impl Into<Vec<Constraint>>) -> Self {
        Self::new(Direction::Horizontal, constraints)
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn constraints(mut self, constraints: impl Into<Vec<Constraint>>) -> Self {
        self.constraints = constraints.into();
        self
    }

    pub fn margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Split the given area into segments according to constraints.
    pub fn split(&self, area: Rect) -> Vec<Rect> {
        let inner = area.inner(&self.margin);
        if self.constraints.is_empty() || inner.is_empty() {
            return vec![inner];
        }

        let total_space = match self.direction {
            Direction::Vertical => inner.height,
            Direction::Horizontal => inner.width,
        };

        let n = self.constraints.len();
        let total_spacing = if n > 1 {
            (n as f32 - 1.0) * self.spacing
        } else {
            0.0
        };
        let available = (total_space - total_spacing).max(0.0);

        // Phase 1: compute initial sizes from constraints
        let mut sizes: Vec<f32> = self
            .constraints
            .iter()
            .map(|c| match c {
                Constraint::Length(l) => l.min(available),
                Constraint::Percentage(p) => available * p / 100.0,
                Constraint::Min(m) => *m,
                Constraint::Max(m) => m.min(available),
                Constraint::Ratio(num, den) => {
                    if *den == 0 {
                        0.0
                    } else {
                        available * *num as f32 / *den as f32
                    }
                }
                Constraint::Fill(_) => 0.0,
            })
            .collect();

        // Phase 2: distribute remaining space to Fill constraints
        let fixed_total: f32 = sizes.iter().sum();
        let remaining = (available - fixed_total).max(0.0);

        let fill_total_weight: f32 = self
            .constraints
            .iter()
            .filter_map(|c| match c {
                Constraint::Fill(w) => Some(*w),
                _ => None,
            })
            .sum();

        if fill_total_weight > 0.0 && remaining > 0.0 {
            for (i, c) in self.constraints.iter().enumerate() {
                if let Constraint::Fill(w) = c {
                    sizes[i] = remaining * w / fill_total_weight;
                }
            }
        }

        // Phase 3: distribute remaining space to Min constraints
        let used: f32 = sizes.iter().sum();
        let leftover = (available - used).max(0.0);
        let min_count = self
            .constraints
            .iter()
            .filter(|c| matches!(c, Constraint::Min(_)))
            .count();

        if min_count > 0 && leftover > 0.0 {
            let share = leftover / min_count as f32;
            for (i, c) in self.constraints.iter().enumerate() {
                if matches!(c, Constraint::Min(_)) {
                    sizes[i] += share;
                }
            }
        }

        // Phase 4: apply Flex distribution
        let total_used: f32 = sizes.iter().sum();
        let excess = (available - total_used).max(0.0);

        let offsets = match self.flex {
            Flex::Start => vec![0.0; n],
            Flex::End => {
                let mut o = vec![0.0; n];
                if n > 0 {
                    o[0] = excess;
                }
                o
            }
            Flex::Center => {
                let mut o = vec![0.0; n];
                if n > 0 {
                    o[0] = excess / 2.0;
                }
                o
            }
            Flex::SpaceBetween => {
                let mut o = vec![0.0; n];
                if n > 1 {
                    let gap = excess / (n as f32 - 1.0);
                    for item in o.iter_mut().skip(1) {
                        *item = gap;
                    }
                }
                o
            }
            Flex::SpaceAround => {
                let mut o = vec![0.0; n];
                let gap = excess / n as f32;
                for (i, item) in o.iter_mut().enumerate() {
                    *item = if i == 0 { gap / 2.0 } else { gap };
                }
                o
            }
            Flex::SpaceEvenly => {
                let mut o = vec![0.0; n];
                let gap = excess / (n as f32 + 1.0);
                for item in o.iter_mut() {
                    *item = gap;
                }
                o
            }
        };

        // Build result rects
        let mut results = Vec::with_capacity(n);
        let mut pos = match self.direction {
            Direction::Vertical => inner.y,
            Direction::Horizontal => inner.x,
        };

        for (i, size) in sizes.iter().enumerate() {
            pos += offsets[i];

            let rect = match self.direction {
                Direction::Vertical => Rect::new(inner.x, pos, inner.width, *size),
                Direction::Horizontal => Rect::new(pos, inner.y, *size, inner.height),
            };
            results.push(rect);
            pos += size;

            if i < n - 1 {
                pos += self.spacing;
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertical_split() {
        let area = Rect::new(0.0, 0.0, 200.0, 400.0);
        let chunks = Layout::vertical(vec![Constraint::Length(100.0), Constraint::Fill(1.0)])
            .split(area);
        assert_eq!(chunks.len(), 2);
        assert!((chunks[0].height - 100.0).abs() < 0.01);
        assert!((chunks[1].height - 300.0).abs() < 0.01);
    }

    #[test]
    fn horizontal_split_with_spacing() {
        let area = Rect::new(0.0, 0.0, 300.0, 100.0);
        let chunks = Layout::horizontal(vec![
            Constraint::Fill(1.0),
            Constraint::Fill(1.0),
            Constraint::Fill(1.0),
        ])
        .spacing(10.0)
        .split(area);
        assert_eq!(chunks.len(), 3);
        // Total spacing: 2 * 10 = 20, so each fill gets (300-20)/3 ≈ 93.33
        let expected = (300.0 - 20.0) / 3.0;
        assert!((chunks[0].width - expected).abs() < 0.01);
    }
}
