//! Backend abstraction for Dewey.
//!
//! Provides the egui/eframe backend for GPU-accelerated rendering via wgpu,
//! the agpu backend for Vulkan-first GPU rendering, and a test backend for
//! headless rendering and snapshot testing.

#[cfg(feature = "agpu-backend")]
pub mod agpu_backend;
#[cfg(feature = "egui-backend")]
pub mod egui_backend;
pub mod image_buffer;
pub mod test;
pub mod web;

#[cfg(feature = "agpu-backend")]
pub use agpu_backend::AgpuProgram;
#[cfg(feature = "egui-backend")]
pub use egui_backend::EguiBackend;
pub use image_buffer::ImagePainter;
pub use test::TestBackend;
pub use web::WebPainter;
