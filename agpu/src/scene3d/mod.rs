//! Basic 3D scene support — camera, meshes, and lighting.
//!
//! Provides a retained-mode 3D scene graph with perspective/orthographic
//! cameras, triangle-mesh geometry, and point/directional lighting.

mod camera;
mod light;
mod mesh;
mod scene;

pub use camera::{Camera, Projection};
pub use light::{DirectionalLight, Light, PointLight};
pub use mesh::{Mesh, Vertex3D};
pub use scene::{Scene3D, SceneNode};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex3d_default() {
        let v = Vertex3D::default();
        assert_eq!(v.position, [0.0, 0.0, 0.0]);
        assert_eq!(v.normal, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn camera_perspective() {
        let cam = Camera::perspective(60.0, 16.0 / 9.0, 0.1, 100.0);
        assert!((cam.fov_y() - 60.0).abs() < 0.01);
    }

    #[test]
    fn camera_orthographic() {
        let cam = Camera::orthographic(10.0, 10.0, 0.1, 100.0);
        let proj = cam.projection();
        assert!(matches!(proj, Projection::Orthographic { .. }));
    }

    #[test]
    fn camera_look_at() {
        let cam = Camera::perspective(60.0, 1.0, 0.1, 100.0)
            .position([0.0, 0.0, 5.0])
            .look_at([0.0, 0.0, 0.0]);
        assert!((cam.eye()[2] - 5.0).abs() < 0.01);
    }

    #[test]
    fn camera_view_matrix() {
        let cam = Camera::perspective(60.0, 1.0, 0.1, 100.0)
            .position([0.0, 0.0, 5.0])
            .look_at([0.0, 0.0, 0.0]);
        let view = cam.view_matrix();
        assert_eq!(view.len(), 16);
        // Identity-like except translation
    }

    #[test]
    fn camera_projection_matrix() {
        let cam = Camera::perspective(60.0, 1.0, 0.1, 100.0);
        let proj = cam.projection_matrix();
        assert_eq!(proj.len(), 16);
    }

    #[test]
    fn mesh_cube() {
        let cube = Mesh::cube(1.0);
        assert_eq!(cube.vertices().len(), 24); // 6 faces × 4 vertices
        assert_eq!(cube.indices().len(), 36); // 6 faces × 2 triangles × 3 indices
    }

    #[test]
    fn mesh_plane() {
        let plane = Mesh::plane(2.0, 2.0);
        assert_eq!(plane.vertices().len(), 4);
        assert_eq!(plane.indices().len(), 6);
    }

    #[test]
    fn mesh_custom() {
        let mesh = Mesh::new(
            vec![Vertex3D::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0])],
            vec![0],
        );
        assert_eq!(mesh.vertices().len(), 1);
    }

    #[test]
    fn point_light_default() {
        let light = PointLight::new([0.0, 5.0, 0.0]);
        assert!((light.intensity - 1.0).abs() < 0.01);
    }

    #[test]
    fn directional_light() {
        let light = DirectionalLight::new([0.0, -1.0, 0.0]);
        assert!((light.direction[1] - (-1.0)).abs() < 0.01);
    }

    #[test]
    fn scene_empty() {
        let scene = Scene3D::new();
        assert_eq!(scene.nodes().len(), 0);
    }

    #[test]
    fn scene_add_node() {
        let mut scene = Scene3D::new();
        scene.add(SceneNode::mesh(Mesh::cube(1.0)));
        assert_eq!(scene.nodes().len(), 1);
    }

    #[test]
    fn scene_with_camera_and_light() {
        let mut scene = Scene3D::new();
        scene.set_camera(Camera::perspective(60.0, 1.0, 0.1, 100.0));
        scene.add_light(Light::Point(PointLight::new([0.0, 5.0, 0.0])));
        assert_eq!(scene.lights().len(), 1);
    }

    #[test]
    fn scene_node_transform() {
        let node = SceneNode::mesh(Mesh::cube(1.0))
            .translate([1.0, 2.0, 3.0])
            .scale([2.0, 2.0, 2.0]);
        assert!((node.translation[0] - 1.0).abs() < 0.01);
        assert!((node.scale_factor[0] - 2.0).abs() < 0.01);
    }
}
