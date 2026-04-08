//! Vertex types for the agpu shape renderer.

use crate::types::{
    BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
};

/// A 2D vertex with position and RGBA colour.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &[
            // position
            VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x2,
            },
            // color
            VertexAttribute {
                offset: std::mem::size_of::<[f32; 2]>() as BufferAddress,
                shader_location: 1,
                format: VertexFormat::Float32x4,
            },
        ],
    };

    #[inline]
    pub fn new(x: f32, y: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            position: [x, y],
            color: [r, g, b, a],
        }
    }
}
