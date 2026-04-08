//! GPU resource wrappers — first-class agpu types for every GPU object.
//!
//! Each wrapper holds the underlying wgpu object plus metadata for agent
//! discoverability (label, id, resource kind).  Users interact with these
//! types instead of raw wgpu handles.  The `.raw()` escape hatch exposes
//! the inner wgpu type for advanced use.

use crate::ontology::*;
use crate::types::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global resource ID counter.
static NEXT_RESOURCE_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_RESOURCE_ID.fetch_add(1, Ordering::Relaxed)
}

// ── GpuBuffer ───────────────────────────────────────────────────────

/// A GPU buffer — vertex, index, uniform, storage, or staging memory.
pub struct GpuBuffer {
    inner: wgpu::Buffer,
    id: u64,
    label: Option<String>,
    size: u64,
    usage: BufferUsages,
}

impl GpuBuffer {
    pub(crate) fn from_raw(inner: wgpu::Buffer, desc: &GpuBufferDescriptor) -> Self {
        Self {
            inner,
            id: next_id(),
            label: desc.label.clone(),
            size: desc.size,
            usage: desc.usage,
        }
    }

    /// Unique resource identifier.
    pub fn id(&self) -> u64 {
        self.id
    }
    /// Optional label.
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    /// Buffer size in bytes.
    pub fn size(&self) -> u64 {
        self.size
    }
    /// Buffer usage flags.
    pub fn usage(&self) -> BufferUsages {
        self.usage
    }
    /// Access the underlying wgpu buffer.
    pub fn raw(&self) -> &wgpu::Buffer {
        &self.inner
    }

    /// Get a slice of the entire buffer.
    pub fn slice(
        &self,
        bounds: impl std::ops::RangeBounds<BufferAddress>,
    ) -> wgpu::BufferSlice<'_> {
        self.inner.slice(bounds)
    }

    /// Helper: get the entire buffer as a binding resource.
    pub fn as_entire_binding(&self) -> wgpu::BindingResource<'_> {
        self.inner.as_entire_binding()
    }
}

// ── GpuTexture ──────────────────────────────────────────────────────

/// A GPU texture — 2D, 3D, or cube-map image data on the GPU.
pub struct GpuTexture {
    inner: wgpu::Texture,
    id: u64,
    label: Option<String>,
    size: Extent3d,
    format: TextureFormat,
    dimension: TextureDimension,
    mip_level_count: u32,
    sample_count: u32,
    usage: TextureUsages,
}

impl GpuTexture {
    pub(crate) fn from_raw(inner: wgpu::Texture, desc: &GpuTextureDescriptor) -> Self {
        Self {
            inner,
            id: next_id(),
            label: desc.label.clone(),
            size: desc.size,
            format: desc.format,
            dimension: desc.dimension,
            mip_level_count: desc.mip_level_count,
            sample_count: desc.sample_count,
            usage: desc.usage,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn size(&self) -> Extent3d {
        self.size
    }
    pub fn format(&self) -> TextureFormat {
        self.format
    }
    pub fn dimension(&self) -> TextureDimension {
        self.dimension
    }
    pub fn mip_level_count(&self) -> u32 {
        self.mip_level_count
    }
    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }
    pub fn usage(&self) -> TextureUsages {
        self.usage
    }
    pub fn raw(&self) -> &wgpu::Texture {
        &self.inner
    }

    /// Create a texture view with default settings.
    pub fn create_view_default(&self) -> GpuTextureView {
        let view = self
            .inner
            .create_view(&wgpu::TextureViewDescriptor::default());
        GpuTextureView {
            inner: view,
            id: next_id(),
            label: self.label.clone().map(|l| format!("{l}_view")),
            format: self.format,
        }
    }

    /// Create a texture view with custom settings.
    pub fn create_view(&self, desc: &wgpu::TextureViewDescriptor<'_>) -> GpuTextureView {
        let view = self.inner.create_view(desc);
        GpuTextureView {
            inner: view,
            id: next_id(),
            label: desc.label.map(String::from),
            format: desc.format.unwrap_or(self.format),
        }
    }
}

impl Discoverable for GpuTexture {
    fn schema(&self) -> WidgetSchema {
        let mut s = WidgetSchema::new(
            "GpuTexture",
            "GPU texture — image data stored on the GPU for sampling or rendering",
            SemanticRole::System,
        );
        s.tags = vec!["gpu".into(), "texture".into(), "image".into()];
        s
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::Custom("gpu_texture".into())]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::simple(
            "get_info",
            "Query texture dimensions, format, and usage",
            false,
        )]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::System
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "label": self.label,
            "width": self.size.width,
            "height": self.size.height,
            "depth_or_layers": self.size.depth_or_array_layers,
            "format": format!("{:?}", self.format),
            "dimension": format!("{:?}", self.dimension),
            "mip_levels": self.mip_level_count,
            "sample_count": self.sample_count,
            "usage": format!("{:?}", self.usage),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "get_info" => Ok(self.agent_state()),
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        None
    }
}

// ── GpuTextureView ──────────────────────────────────────────────────

/// A view into a GPU texture.
pub struct GpuTextureView {
    inner: wgpu::TextureView,
    id: u64,
    label: Option<String>,
    format: TextureFormat,
}

impl GpuTextureView {
    pub(crate) fn from_surface(inner: wgpu::TextureView, format: TextureFormat) -> Self {
        Self {
            inner,
            id: next_id(),
            label: Some("surface_view".into()),
            format,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn format(&self) -> TextureFormat {
        self.format
    }
    pub fn raw(&self) -> &wgpu::TextureView {
        &self.inner
    }
}

// ── GpuSampler ──────────────────────────────────────────────────────

/// A GPU sampler — controls how textures are filtered and addressed.
pub struct GpuSampler {
    inner: wgpu::Sampler,
    id: u64,
    label: Option<String>,
}

impl GpuSampler {
    pub(crate) fn from_raw(inner: wgpu::Sampler, label: Option<String>) -> Self {
        Self {
            inner,
            id: next_id(),
            label,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn raw(&self) -> &wgpu::Sampler {
        &self.inner
    }
}

// ── GpuShaderModule ─────────────────────────────────────────────────

/// A compiled GPU shader module (WGSL or SPIR-V).
pub struct GpuShaderModule {
    inner: wgpu::ShaderModule,
    id: u64,
    label: Option<String>,
    source_kind: ShaderSourceKind,
}

/// How the shader was provided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ShaderSourceKind {
    Wgsl,
    SpirV,
    Naga,
}

impl GpuShaderModule {
    pub(crate) fn from_raw(
        inner: wgpu::ShaderModule,
        label: Option<String>,
        source_kind: ShaderSourceKind,
    ) -> Self {
        Self {
            inner,
            id: next_id(),
            label,
            source_kind,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn source_kind(&self) -> ShaderSourceKind {
        self.source_kind
    }
    pub fn raw(&self) -> &wgpu::ShaderModule {
        &self.inner
    }
}

// ── GpuBindGroupLayout ──────────────────────────────────────────────

/// A bind group layout describing the shape of a bind group.
pub struct GpuBindGroupLayout {
    inner: wgpu::BindGroupLayout,
    id: u64,
    label: Option<String>,
}

impl GpuBindGroupLayout {
    pub(crate) fn from_raw(inner: wgpu::BindGroupLayout, label: Option<String>) -> Self {
        Self {
            inner,
            id: next_id(),
            label,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn raw(&self) -> &wgpu::BindGroupLayout {
        &self.inner
    }
}

// ── GpuBindGroup ────────────────────────────────────────────────────

/// A bind group — a set of resources bound to a shader.
pub struct GpuBindGroup {
    inner: wgpu::BindGroup,
    id: u64,
    label: Option<String>,
}

impl GpuBindGroup {
    pub(crate) fn from_raw(inner: wgpu::BindGroup, label: Option<String>) -> Self {
        Self {
            inner,
            id: next_id(),
            label,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn raw(&self) -> &wgpu::BindGroup {
        &self.inner
    }
}

// ── GpuPipelineLayout ───────────────────────────────────────────────

/// A pipeline layout describing bind group layouts and push constants.
pub struct GpuPipelineLayout {
    inner: wgpu::PipelineLayout,
    id: u64,
    label: Option<String>,
}

impl GpuPipelineLayout {
    pub(crate) fn from_raw(inner: wgpu::PipelineLayout, label: Option<String>) -> Self {
        Self {
            inner,
            id: next_id(),
            label,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn raw(&self) -> &wgpu::PipelineLayout {
        &self.inner
    }
}

// ── GpuRenderPipeline ───────────────────────────────────────────────

/// A GPU render pipeline — vertex + fragment shaders, primitive state,
/// blend mode, and depth/stencil configuration.
pub struct GpuRenderPipeline {
    inner: wgpu::RenderPipeline,
    id: u64,
    label: Option<String>,
}

impl GpuRenderPipeline {
    pub(crate) fn from_raw(inner: wgpu::RenderPipeline, label: Option<String>) -> Self {
        Self {
            inner,
            id: next_id(),
            label,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn raw(&self) -> &wgpu::RenderPipeline {
        &self.inner
    }
}

impl Discoverable for GpuRenderPipeline {
    fn schema(&self) -> WidgetSchema {
        let mut s = WidgetSchema::new(
            "GpuRenderPipeline",
            "GPU render pipeline — compiled vertex/fragment shader program",
            SemanticRole::System,
        );
        s.tags = vec![
            "gpu".into(),
            "pipeline".into(),
            "render".into(),
            "shader".into(),
        ];
        s
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::Custom("gpu_render_pipeline".into())]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::simple(
            "get_info",
            "Query pipeline metadata",
            false,
        )]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::System
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "label": self.label,
            "kind": "RenderPipeline",
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "get_info" => Ok(self.agent_state()),
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        None
    }
}

// ── GpuComputePipeline ──────────────────────────────────────────────

/// A GPU compute pipeline — a compiled compute shader program.
pub struct GpuComputePipeline {
    inner: wgpu::ComputePipeline,
    id: u64,
    label: Option<String>,
}

impl GpuComputePipeline {
    pub(crate) fn from_raw(inner: wgpu::ComputePipeline, label: Option<String>) -> Self {
        Self {
            inner,
            id: next_id(),
            label,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn raw(&self) -> &wgpu::ComputePipeline {
        &self.inner
    }
}

impl Discoverable for GpuComputePipeline {
    fn schema(&self) -> WidgetSchema {
        let mut s = WidgetSchema::new(
            "GpuComputePipeline",
            "GPU compute pipeline — compiled compute shader program for GPGPU workloads",
            SemanticRole::System,
        );
        s.tags = vec![
            "gpu".into(),
            "pipeline".into(),
            "compute".into(),
            "shader".into(),
        ];
        s
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::Custom("gpu_compute_pipeline".into())]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::simple(
            "get_info",
            "Query compute pipeline metadata",
            false,
        )]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::System
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "label": self.label,
            "kind": "ComputePipeline",
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "get_info" => Ok(self.agent_state()),
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        None
    }
}

// ── GpuQuerySet ─────────────────────────────────────────────────────

/// A GPU query set — for occlusion queries or timestamp queries.
pub struct GpuQuerySet {
    inner: wgpu::QuerySet,
    id: u64,
    label: Option<String>,
    count: u32,
}

impl GpuQuerySet {
    pub(crate) fn from_raw(inner: wgpu::QuerySet, label: Option<String>, count: u32) -> Self {
        Self {
            inner,
            id: next_id(),
            label,
            count,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn count(&self) -> u32 {
        self.count
    }
    pub fn raw(&self) -> &wgpu::QuerySet {
        &self.inner
    }
}
