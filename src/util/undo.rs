//! Undo/redo history.

/// A simple undo/redo stack.
pub struct UndoStack<T: Clone> {
    states: Vec<T>,
    current: usize,
    max_depth: usize,
}

impl<T: Clone> UndoStack<T> {
    /// Create an undo stack with the given initial state and max depth.
    pub fn new(initial: T, max_depth: usize) -> Self {
        Self {
            states: vec![initial],
            current: 0,
            max_depth,
        }
    }

    /// Push a new state, discarding any redo history.
    pub fn push(&mut self, state: T) {
        self.states.truncate(self.current + 1);
        self.states.push(state);
        self.current += 1;

        // Enforce max depth
        if self.states.len() > self.max_depth {
            let excess = self.states.len() - self.max_depth;
            self.states.drain(0..excess);
            self.current = self.current.saturating_sub(excess);
        }
    }

    /// Undo: move back one state. Returns the state if successful.
    pub fn undo(&mut self) -> Option<&T> {
        if self.current > 0 {
            self.current -= 1;
            Some(&self.states[self.current])
        } else {
            None
        }
    }

    /// Redo: move forward one state. Returns the state if successful.
    pub fn redo(&mut self) -> Option<&T> {
        if self.current + 1 < self.states.len() {
            self.current += 1;
            Some(&self.states[self.current])
        } else {
            None
        }
    }

    /// The current state.
    pub fn current(&self) -> &T {
        &self.states[self.current]
    }

    /// Whether undo is available.
    pub fn can_undo(&self) -> bool {
        self.current > 0
    }

    /// Whether redo is available.
    pub fn can_redo(&self) -> bool {
        self.current + 1 < self.states.len()
    }

    /// Number of states in the stack.
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Whether the stack has only one state.
    pub fn is_empty(&self) -> bool {
        self.states.len() <= 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_redo() {
        let mut stack = UndoStack::new("a".to_string(), 100);
        stack.push("b".to_string());
        stack.push("c".to_string());
        assert_eq!(stack.current(), "c");

        assert_eq!(stack.undo().unwrap(), "b");
        assert_eq!(stack.undo().unwrap(), "a");
        assert!(stack.undo().is_none());

        assert_eq!(stack.redo().unwrap(), "b");
        assert_eq!(stack.current(), "b");

        // Push discards redo
        stack.push("d".to_string());
        assert!(!stack.can_redo());
        assert_eq!(stack.current(), "d");
    }

    #[test]
    fn max_depth() {
        let mut stack = UndoStack::new(0, 3);
        stack.push(1);
        stack.push(2);
        stack.push(3); // Should drop state 0
        assert_eq!(stack.len(), 3);
    }
}
