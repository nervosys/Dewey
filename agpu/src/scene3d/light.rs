//! Lighting — point and directional lights for 3D scenes.

use crate::core::Color;

/// A point light with position, color, and intensity.
#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: [f32; 3],
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
}

impl PointLight {
    pub fn new(position: [f32; 3]) -> Self {
        Self {
            position,
            color: Color::WHITE,
            intensity: 1.0,
            range: 50.0,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    pub fn range(mut self, range: f32) -> Self {
        self.range = range;
        self
    }
}

/// A directional light (sun-like) with direction and color.
#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub direction: [f32; 3],
    pub color: Color,
    pub intensity: f32,
}

impl DirectionalLight {
    pub fn new(direction: [f32; 3]) -> Self {
        Self {
            direction,
            color: Color::WHITE,
            intensity: 1.0,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }
}

/// Unified light enum.
#[derive(Debug, Clone, Copy)]
pub enum Light {
    Point(PointLight),
    Directional(DirectionalLight),
}
