//! Mesh — triangle-mesh geometry with 3D vertices.

/// A 3D vertex with position, normal, and texture coordinates.
#[derive(Debug, Clone, Copy)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Default for Vertex3D {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
        }
    }
}

impl Vertex3D {
    pub fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            uv,
        }
    }
}

/// A triangle mesh with indexed geometry.
pub struct Mesh {
    vertices: Vec<Vertex3D>,
    indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex3D>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub fn vertices(&self) -> &[Vertex3D] {
        &self.vertices
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// Generate a unit cube centred at the origin.
    pub fn cube(size: f32) -> Self {
        let h = size / 2.0;
        let mut vertices = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);

        // Six faces, each with 4 vertices and 2 triangles
        type CubeFace = ([f32; 3], [[f32; 3]; 4], [[f32; 2]; 4]);
        let faces: [CubeFace; 6] = [
            // Front (+Z)
            (
                [0.0, 0.0, 1.0],
                [[-h, -h, h], [h, -h, h], [h, h, h], [-h, h, h]],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // Back (-Z)
            (
                [0.0, 0.0, -1.0],
                [[h, -h, -h], [-h, -h, -h], [-h, h, -h], [h, h, -h]],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // Right (+X)
            (
                [1.0, 0.0, 0.0],
                [[h, -h, h], [h, -h, -h], [h, h, -h], [h, h, h]],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // Left (-X)
            (
                [-1.0, 0.0, 0.0],
                [[-h, -h, -h], [-h, -h, h], [-h, h, h], [-h, h, -h]],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // Top (+Y)
            (
                [0.0, 1.0, 0.0],
                [[-h, h, h], [h, h, h], [h, h, -h], [-h, h, -h]],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // Bottom (-Y)
            (
                [0.0, -1.0, 0.0],
                [[-h, -h, -h], [h, -h, -h], [h, -h, h], [-h, -h, h]],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
        ];

        for (normal, positions, uvs) in &faces {
            let base = vertices.len() as u32;
            for i in 0..4 {
                vertices.push(Vertex3D::new(positions[i], *normal, uvs[i]));
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }

        Self { vertices, indices }
    }

    /// Generate a flat plane on the XZ plane, centred at the origin.
    pub fn plane(width: f32, depth: f32) -> Self {
        let hw = width / 2.0;
        let hd = depth / 2.0;
        let vertices = vec![
            Vertex3D::new([-hw, 0.0, -hd], [0.0, 1.0, 0.0], [0.0, 0.0]),
            Vertex3D::new([hw, 0.0, -hd], [0.0, 1.0, 0.0], [1.0, 0.0]),
            Vertex3D::new([hw, 0.0, hd], [0.0, 1.0, 0.0], [1.0, 1.0]),
            Vertex3D::new([-hw, 0.0, hd], [0.0, 1.0, 0.0], [0.0, 1.0]),
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];
        Self { vertices, indices }
    }
}
