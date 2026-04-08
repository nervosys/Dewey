//! Scene graph — retained-mode 3D scene with nodes, camera, and lights.

use super::camera::Camera;
use super::light::Light;
use super::mesh::Mesh;

/// A node in the 3D scene graph.
pub struct SceneNode {
    pub mesh: Mesh,
    pub translation: [f32; 3],
    pub rotation: [f32; 3], // Euler angles in radians
    pub scale_factor: [f32; 3],
    pub visible: bool,
}

impl SceneNode {
    pub fn mesh(mesh: Mesh) -> Self {
        Self {
            mesh,
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale_factor: [1.0, 1.0, 1.0],
            visible: true,
        }
    }

    pub fn translate(mut self, translation: [f32; 3]) -> Self {
        self.translation = translation;
        self
    }

    pub fn rotate(mut self, rotation: [f32; 3]) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn scale(mut self, scale: [f32; 3]) -> Self {
        self.scale_factor = scale;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Compute a 4×4 model matrix (column-major) from translation, rotation, scale.
    pub fn model_matrix(&self) -> [f32; 16] {
        let [tx, ty, tz] = self.translation;
        let [rx, ry, rz] = self.rotation;
        let [sx, sy, sz] = self.scale_factor;

        // Rotation: Rz * Ry * Rx
        let (cx, sx_r) = (rx.cos(), rx.sin());
        let (cy, sy_r) = (ry.cos(), ry.sin());
        let (cz, sz_r) = (rz.cos(), rz.sin());

        [
            sx * cy * cz,
            sx * cy * sz_r,
            sx * (-sy_r),
            0.0,
            sy * (sx_r * sy_r * cz - cx * sz_r),
            sy * (sx_r * sy_r * sz_r + cx * cz),
            sy * sx_r * cy,
            0.0,
            sz * (cx * sy_r * cz + sx_r * sz_r),
            sz * (cx * sy_r * sz_r - sx_r * cz),
            sz * cx * cy,
            0.0,
            tx,
            ty,
            tz,
            1.0,
        ]
    }
}

/// A complete 3D scene with camera, lights, and nodes.
pub struct Scene3D {
    camera: Camera,
    lights: Vec<Light>,
    nodes: Vec<SceneNode>,
}

impl Scene3D {
    pub fn new() -> Self {
        Self {
            camera: Camera::perspective(60.0, 16.0 / 9.0, 0.1, 100.0),
            lights: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn lights(&self) -> &[Light] {
        &self.lights
    }

    pub fn add(&mut self, node: SceneNode) {
        self.nodes.push(node);
    }

    pub fn nodes(&self) -> &[SceneNode] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut [SceneNode] {
        &mut self.nodes
    }
}

impl Default for Scene3D {
    fn default() -> Self {
        Self::new()
    }
}
