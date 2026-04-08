//! Animation framework — declarative animations with easing functions.
//!
//! Provides [`Animation`], [`Easing`], and [`AnimationManager`] for
//! smooth property transitions and timed sequences.

use std::time::Duration;

/// Easing function for animation interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    #[default]
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    Spring { stiffness: f32, damping: f32 },
}

impl Easing {
    /// Apply the easing function to a linear progress value in [0.0, 1.0].
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => t * (2.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Self::EaseInQuad => t * t,
            Self::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Self::EaseInCubic => t * t * t,
            Self::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Self::Spring {
                stiffness,
                damping,
            } => {
                let omega = stiffness.sqrt();
                let zeta = damping / (2.0 * omega);
                if zeta < 1.0 {
                    let wd = omega * (1.0 - zeta * zeta).sqrt();
                    1.0 - (-zeta * omega * t).exp() * ((wd * t).cos() + zeta / (1.0 - zeta * zeta).sqrt() * (wd * t).sin())
                } else {
                    1.0 - (1.0 + omega * t) * (-omega * t).exp()
                }
            }
        }
    }
}



/// State of an animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Pending,
    Running,
    Paused,
    Completed,
}

/// A single animation that interpolates a value over time.
#[derive(Debug, Clone)]
pub struct Animation {
    id: String,
    from: f64,
    to: f64,
    duration: Duration,
    delay: Duration,
    easing: Easing,
    repeat: RepeatMode,
    state: AnimationState,
    elapsed: Duration,
}

/// How an animation repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Once,
    Loop,
    PingPong,
    Count(u32),
}

impl Animation {
    pub fn new(id: impl Into<String>, from: f64, to: f64, duration: Duration) -> Self {
        Self {
            id: id.into(),
            from,
            to,
            duration,
            delay: Duration::ZERO,
            easing: Easing::default(),
            repeat: RepeatMode::Once,
            state: AnimationState::Pending,
            elapsed: Duration::ZERO,
        }
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    pub fn repeat(mut self, mode: RepeatMode) -> Self {
        self.repeat = mode;
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn state(&self) -> AnimationState {
        self.state
    }

    /// Start or resume the animation.
    pub fn start(&mut self) {
        self.state = AnimationState::Running;
    }

    /// Pause the animation.
    pub fn pause(&mut self) {
        if self.state == AnimationState::Running {
            self.state = AnimationState::Paused;
        }
    }

    /// Reset to the beginning.
    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.state = AnimationState::Pending;
    }

    /// Advance the animation by the given delta time. Returns the current value.
    pub fn tick(&mut self, dt: Duration) -> f64 {
        if self.state != AnimationState::Running {
            return self.current_value();
        }

        self.elapsed += dt;

        // Handle delay
        if self.elapsed < self.delay {
            return self.from;
        }

        let active_elapsed = self.elapsed - self.delay;
        let raw_t = active_elapsed.as_secs_f64() / self.duration.as_secs_f64();

        let t = match self.repeat {
            RepeatMode::Once => {
                if raw_t >= 1.0 {
                    self.state = AnimationState::Completed;
                    1.0
                } else {
                    raw_t
                }
            }
            RepeatMode::Loop => raw_t.fract(),
            RepeatMode::PingPong => {
                let cycle = raw_t % 2.0;
                if cycle > 1.0 {
                    2.0 - cycle
                } else {
                    cycle
                }
            }
            RepeatMode::Count(n) => {
                if raw_t >= n as f64 {
                    self.state = AnimationState::Completed;
                    1.0
                } else {
                    raw_t.fract()
                }
            }
        };

        let eased = self.easing.apply(t as f32) as f64;
        self.from + (self.to - self.from) * eased
    }

    fn current_value(&self) -> f64 {
        match self.state {
            AnimationState::Pending => self.from,
            AnimationState::Completed => self.to,
            _ => {
                if self.elapsed < self.delay {
                    return self.from;
                }
                let active = self.elapsed - self.delay;
                let t = (active.as_secs_f64() / self.duration.as_secs_f64()).clamp(0.0, 1.0);
                let eased = self.easing.apply(t as f32) as f64;
                self.from + (self.to - self.from) * eased
            }
        }
    }
}

/// Manages a collection of active animations.
pub struct AnimationManager {
    animations: Vec<Animation>,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
        }
    }

    pub fn add(&mut self, mut anim: Animation) {
        anim.start();
        self.animations.push(anim);
    }

    /// Tick all animations by dt, remove completed ones. Returns values keyed by id.
    pub fn tick(&mut self, dt: Duration) -> Vec<(String, f64)> {
        let values: Vec<(String, f64)> = self
            .animations
            .iter_mut()
            .map(|a| {
                let val = a.tick(dt);
                (a.id().to_string(), val)
            })
            .collect();

        // Remove completed non-repeating animations
        self.animations
            .retain(|a| a.state != AnimationState::Completed);

        values
    }

    pub fn get(&self, id: &str) -> Option<&Animation> {
        self.animations.iter().find(|a| a.id() == id)
    }

    pub fn cancel(&mut self, id: &str) {
        self.animations.retain(|a| a.id() != id);
    }

    pub fn active_count(&self) -> usize {
        self.animations.len()
    }
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easing_linear() {
        assert!((Easing::Linear.apply(0.5) - 0.5).abs() < 0.001);
    }

    #[test]
    fn easing_endpoints() {
        for easing in [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::EaseInCubic,
            Easing::EaseOutCubic,
        ] {
            assert!((easing.apply(0.0)).abs() < 0.001, "{easing:?} at 0");
            assert!((easing.apply(1.0) - 1.0).abs() < 0.001, "{easing:?} at 1");
        }
    }

    #[test]
    fn easing_clamps() {
        assert!((Easing::Linear.apply(-0.5)).abs() < 0.001);
        assert!((Easing::Linear.apply(1.5) - 1.0).abs() < 0.001);
    }

    #[test]
    fn animation_basic() {
        let mut anim = Animation::new("x", 0.0, 100.0, Duration::from_millis(1000));
        anim.start();
        let val = anim.tick(Duration::from_millis(500));
        assert!(val > 0.0 && val < 100.0);
    }

    #[test]
    fn animation_completes() {
        let mut anim = Animation::new("x", 0.0, 100.0, Duration::from_millis(100));
        anim.start();
        let val = anim.tick(Duration::from_millis(200));
        assert!((val - 100.0).abs() < 0.01);
        assert_eq!(anim.state(), AnimationState::Completed);
    }

    #[test]
    fn animation_delay() {
        let mut anim = Animation::new("x", 0.0, 1.0, Duration::from_millis(100))
            .delay(Duration::from_millis(50));
        anim.start();
        let val = anim.tick(Duration::from_millis(25));
        assert!((val - 0.0).abs() < 0.001); // Still in delay
    }

    #[test]
    fn animation_pause_resume() {
        let mut anim = Animation::new("x", 0.0, 100.0, Duration::from_millis(1000));
        anim.start();
        anim.tick(Duration::from_millis(300));
        anim.pause();
        let val_paused = anim.tick(Duration::from_millis(500));
        anim.start();
        let val_resumed = anim.tick(Duration::from_millis(100));
        assert!(val_resumed > val_paused || (val_resumed - val_paused).abs() < 0.01);
    }

    #[test]
    fn animation_reset() {
        let mut anim = Animation::new("x", 0.0, 100.0, Duration::from_millis(100));
        anim.start();
        anim.tick(Duration::from_millis(200));
        anim.reset();
        assert_eq!(anim.state(), AnimationState::Pending);
    }

    #[test]
    fn animation_loop() {
        let mut anim = Animation::new("x", 0.0, 1.0, Duration::from_millis(100))
            .repeat(RepeatMode::Loop);
        anim.start();
        anim.tick(Duration::from_millis(150));
        assert_eq!(anim.state(), AnimationState::Running); // Still going
    }

    #[test]
    fn animation_pingpong() {
        let mut anim = Animation::new("x", 0.0, 1.0, Duration::from_millis(100))
            .repeat(RepeatMode::PingPong)
            .easing(Easing::Linear);
        anim.start();
        // At t=150ms (1.5 cycles), should be going back towards 0
        let val = anim.tick(Duration::from_millis(150));
        assert!(val < 0.6);
    }

    #[test]
    fn manager_add_and_tick() {
        let mut mgr = AnimationManager::new();
        mgr.add(Animation::new("a", 0.0, 1.0, Duration::from_millis(100)));
        mgr.add(Animation::new("b", 10.0, 20.0, Duration::from_millis(200)));
        let vals = mgr.tick(Duration::from_millis(50));
        assert_eq!(vals.len(), 2);
    }

    #[test]
    fn manager_removes_completed() {
        let mut mgr = AnimationManager::new();
        mgr.add(Animation::new("short", 0.0, 1.0, Duration::from_millis(50)));
        mgr.add(Animation::new("long", 0.0, 1.0, Duration::from_millis(500)));
        mgr.tick(Duration::from_millis(100));
        assert_eq!(mgr.active_count(), 1);
    }

    #[test]
    fn manager_cancel() {
        let mut mgr = AnimationManager::new();
        mgr.add(Animation::new("a", 0.0, 1.0, Duration::from_millis(500)));
        mgr.cancel("a");
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn easing_spring() {
        let spring = Easing::Spring {
            stiffness: 100.0,
            damping: 10.0,
        };
        let val = spring.apply(0.5);
        assert!(val > 0.0);
    }
}
