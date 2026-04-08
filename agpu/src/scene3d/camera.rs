//! Camera — perspective and orthographic projection.

/// Projection type.
#[derive(Debug, Clone, Copy)]
pub enum Projection {
    Perspective {
        fov_y: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        width: f32,
        height: f32,
        near: f32,
        far: f32,
    },
}

/// A 3D camera with position, target, and projection.
#[derive(Debug, Clone)]
pub struct Camera {
    eye: [f32; 3],
    target: [f32; 3],
    up: [f32; 3],
    projection: Projection,
}

impl Camera {
    pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            eye: [0.0, 0.0, 5.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            projection: Projection::Perspective {
                fov_y,
                aspect,
                near,
                far,
            },
        }
    }

    pub fn orthographic(width: f32, height: f32, near: f32, far: f32) -> Self {
        Self {
            eye: [0.0, 0.0, 5.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            projection: Projection::Orthographic {
                width,
                height,
                near,
                far,
            },
        }
    }

    pub fn position(mut self, eye: [f32; 3]) -> Self {
        self.eye = eye;
        self
    }

    pub fn look_at(mut self, target: [f32; 3]) -> Self {
        self.target = target;
        self
    }

    pub fn up(mut self, up: [f32; 3]) -> Self {
        self.up = up;
        self
    }

    pub fn eye(&self) -> [f32; 3] {
        self.eye
    }

    pub fn target(&self) -> [f32; 3] {
        self.target
    }

    pub fn projection(&self) -> Projection {
        self.projection
    }

    pub fn fov_y(&self) -> f32 {
        match self.projection {
            Projection::Perspective { fov_y, .. } => fov_y,
            Projection::Orthographic { .. } => 0.0,
        }
    }

    /// Compute a 4×4 view matrix (column-major) using a look-at transform.
    pub fn view_matrix(&self) -> [f32; 16] {
        let f = normalize(sub(self.target, self.eye));
        let s = normalize(cross(f, self.up));
        let u = cross(s, f);

        [
            s[0],
            u[0],
            -f[0],
            0.0,
            s[1],
            u[1],
            -f[1],
            0.0,
            s[2],
            u[2],
            -f[2],
            0.0,
            -dot(s, self.eye),
            -dot(u, self.eye),
            dot(f, self.eye),
            1.0,
        ]
    }

    /// Compute a 4×4 projection matrix (column-major).
    pub fn projection_matrix(&self) -> [f32; 16] {
        match self.projection {
            Projection::Perspective {
                fov_y,
                aspect,
                near,
                far,
            } => {
                let f = 1.0 / (fov_y.to_radians() / 2.0).tan();
                let nf = 1.0 / (near - far);
                [
                    f / aspect,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    f,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    (far + near) * nf,
                    -1.0,
                    0.0,
                    0.0,
                    2.0 * far * near * nf,
                    0.0,
                ]
            }
            Projection::Orthographic {
                width,
                height,
                near,
                far,
            } => {
                let nf = 1.0 / (near - far);
                [
                    2.0 / width,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    2.0 / height,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    2.0 * nf,
                    0.0,
                    0.0,
                    0.0,
                    (far + near) * nf,
                    1.0,
                ]
            }
        }
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}
