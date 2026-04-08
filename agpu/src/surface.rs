//! GPU surface — the swapchain / presentation target wrapping the
//! platform window surface.

use crate::context::GpuContext;
use crate::resource::GpuTextureView;
use crate::types::*;
use std::sync::atomic::{AtomicU64, Ordering};

#[allow(dead_code)]
static NEXT_SURFACE_ID: AtomicU64 = AtomicU64::new(1);

/// A GPU surface — the render target presented to a window.
pub struct GpuSurface {
    inner: wgpu::Surface<'static>,
    id: u64,
    config: wgpu::SurfaceConfiguration,
}

impl GpuSurface {
    /// Wrap an existing wgpu surface with its configuration.
    #[allow(dead_code)]
    pub(crate) fn from_raw(
        inner: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        Self {
            inner,
            id: NEXT_SURFACE_ID.fetch_add(1, Ordering::Relaxed),
            config,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    /// Current texture format.
    pub fn format(&self) -> TextureFormat {
        self.config.format
    }

    /// Current present mode.
    pub fn present_mode(&self) -> PresentMode {
        self.config.present_mode
    }

    /// Current width.
    pub fn width(&self) -> u32 {
        self.config.width
    }

    /// Current height.
    pub fn height(&self) -> u32 {
        self.config.height
    }

    /// Reconfigure the surface (e.g. after a resize).
    pub fn configure(&mut self, gpu: &GpuContext) {
        self.inner.configure(gpu.device(), &self.config);
    }

    /// Update dimensions and reconfigure.
    pub fn resize(&mut self, gpu: &GpuContext, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.configure(gpu);
        }
    }

    /// Update present mode and reconfigure.
    pub fn set_present_mode(&mut self, gpu: &GpuContext, mode: PresentMode) {
        self.config.present_mode = mode;
        self.configure(gpu);
    }

    /// Get the next frame to render into.
    pub fn get_current_texture(&self) -> Result<GpuSurfaceTexture, SurfaceError> {
        let frame = self.inner.get_current_texture()?;
        Ok(GpuSurfaceTexture { inner: frame })
    }

    /// Access the underlying wgpu surface.
    pub fn raw(&self) -> &wgpu::Surface<'static> {
        &self.inner
    }

    /// Access the surface configuration.
    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    /// Get surface capabilities for an adapter.
    pub fn get_capabilities(&self, adapter: &wgpu::Adapter) -> wgpu::SurfaceCapabilities {
        self.inner.get_capabilities(adapter)
    }
}

/// A surface texture frame — the current backbuffer to render into.
pub struct GpuSurfaceTexture {
    inner: wgpu::SurfaceTexture,
}

impl GpuSurfaceTexture {
    /// Create a default view of this frame.
    pub fn create_view(&self, format: TextureFormat) -> GpuTextureView {
        let view = self
            .inner
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        GpuTextureView::from_surface(view, format)
    }

    /// Present the frame to the screen.
    pub fn present(self) {
        self.inner.present();
    }

    /// Access the underlying wgpu surface texture.
    pub fn raw(&self) -> &wgpu::SurfaceTexture {
        &self.inner
    }
}
