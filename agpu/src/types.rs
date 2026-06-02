//! Core GPU types — agpu's own type system replacing direct wgpu dependency.
//!
//! Users depend on `agpu` instead of `wgpu`. Every type is defined or
//! re-exported here so that downstream crates never need `wgpu` in their
//! `Cargo.toml`.

// ── Re-exports from wgpu ────────────────────────────────────────────
//
// Configuration types that are pure data — no resource handles.
// Users see these as `agpu::TextureFormat`, `agpu::BufferUsages`, etc.

// Texture
pub use wgpu::Extent3d;
pub use wgpu::Origin3d;
pub use wgpu::StorageTextureAccess;
pub use wgpu::TextureAspect;
pub use wgpu::TextureDimension;
pub use wgpu::TextureFormat;
pub use wgpu::TextureSampleType;
pub use wgpu::TextureUsages;
pub use wgpu::TextureViewDimension;

// Buffer
pub use wgpu::BufferAddress;
pub use wgpu::BufferSize;
pub use wgpu::BufferUsages;

// Pipeline
pub use wgpu::DepthBiasState;
pub use wgpu::DepthStencilState;
pub use wgpu::Face;
pub use wgpu::FrontFace;
pub use wgpu::IndexFormat;
pub use wgpu::MultisampleState;
pub use wgpu::PipelineCompilationOptions;
pub use wgpu::PolygonMode;
pub use wgpu::PrimitiveState;
pub use wgpu::PrimitiveTopology;
pub use wgpu::StencilFaceState;
pub use wgpu::StencilOperation;
pub use wgpu::StencilState;

// Blend / Color
pub use wgpu::BlendComponent;
pub use wgpu::BlendFactor;
pub use wgpu::BlendOperation;
pub use wgpu::BlendState;
pub use wgpu::Color;
pub use wgpu::ColorTargetState;
pub use wgpu::ColorWrites;

// Shader
pub use wgpu::ShaderSource;
pub use wgpu::ShaderStages;

// Bind group
pub use wgpu::BindingType;
pub use wgpu::BufferBindingType;
pub use wgpu::SamplerBindingType;

// Vertex
pub use wgpu::VertexAttribute;
pub use wgpu::VertexBufferLayout;
pub use wgpu::VertexFormat;
pub use wgpu::VertexStepMode;

// Render pass
pub use wgpu::LoadOp;
pub use wgpu::Operations;
pub use wgpu::StoreOp;

// Sampler
pub use wgpu::AddressMode;
pub use wgpu::CompareFunction;
pub use wgpu::FilterMode;

// Surface
pub use wgpu::PresentMode;
pub use wgpu::SurfaceConfiguration;

// Device
pub use wgpu::Features;
pub use wgpu::Limits;
pub use wgpu::Maintain;
pub use wgpu::MemoryHints;
pub use wgpu::PowerPreference;

// Backend selection
pub use wgpu::Backends;

// Misc
pub use wgpu::CommandEncoderDescriptor;
pub use wgpu::MapMode;
pub use wgpu::RenderPassColorAttachment;
pub use wgpu::RenderPassDepthStencilAttachment;
pub use wgpu::RenderPassDescriptor;
pub use wgpu::RenderPassTimestampWrites;
pub use wgpu::SurfaceError;
pub use wgpu::TextureViewDescriptor;

// Raw wgpu types needed by downstream backends
pub use wgpu::Device;
pub use wgpu::Instance;
pub use wgpu::InstanceDescriptor;
pub use wgpu::Queue;
pub use wgpu::Surface;
pub use wgpu::TextureDescriptor;
pub use wgpu::TextureView;

// ── agpu-specific types ─────────────────────────────────────────────

/// Backend preference for GPU initialisation.
///
/// agpu is Vulkan-first: when `VulkanPreferred` is used (the default),
/// the engine tries the Vulkan backend before falling back to the
/// platform default.  `OpenGLPreferred` selects GL/GLES first.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BackendPreference {
    /// Try Vulkan first, then fall back to the platform default.
    VulkanPreferred,
    /// Try OpenGL/GLES first, then fall back to the platform default.
    OpenGLPreferred,
    /// Use the platform default (DX12 on Windows, Metal on macOS, Vulkan on Linux).
    #[default]
    PlatformDefault,

    /// Force a specific set of backends.
    Specific(Backends),
}

impl BackendPreference {
    /// Convert to the concrete `wgpu::Backends` bitflags.
    ///
    /// `PlatformDefault` selects the native API for the current OS:
    /// DX12 on Windows, Metal on macOS, Vulkan elsewhere.
    pub fn to_backends(self) -> Backends {
        match self {
            Self::VulkanPreferred => Backends::VULKAN,
            Self::OpenGLPreferred => Backends::GL,
            Self::PlatformDefault => {
                #[cfg(target_os = "windows")]
                {
                    Backends::DX12
                }
                #[cfg(target_os = "macos")]
                {
                    Backends::METAL
                }
                #[cfg(not(any(target_os = "windows", target_os = "macos")))]
                {
                    Backends::VULKAN
                }
            }
            Self::Specific(b) => b,
        }
    }
}

/// Describes how to create a GPU buffer.
#[derive(Debug, Clone)]
pub struct GpuBufferDescriptor {
    pub label: Option<String>,
    pub size: u64,
    pub usage: BufferUsages,
    pub mapped_at_creation: bool,
}

impl Default for GpuBufferDescriptor {
    fn default() -> Self {
        Self {
            label: None,
            size: 0,
            usage: BufferUsages::empty(),
            mapped_at_creation: false,
        }
    }
}

/// Describes how to create a GPU texture.
#[derive(Debug, Clone)]
pub struct GpuTextureDescriptor {
    pub label: Option<String>,
    pub size: Extent3d,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub dimension: TextureDimension,
    pub format: TextureFormat,
    pub usage: TextureUsages,
    pub view_formats: Vec<TextureFormat>,
}

impl Default for GpuTextureDescriptor {
    fn default() -> Self {
        Self {
            label: None,
            size: Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::empty(),
            view_formats: Vec::new(),
        }
    }
}

/// Describes how to create a GPU sampler.
#[derive(Debug, Clone)]
pub struct GpuSamplerDescriptor {
    pub label: Option<String>,
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
    pub mipmap_filter: FilterMode,
    pub compare: Option<CompareFunction>,
    pub anisotropy_clamp: u16,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
}

impl Default for GpuSamplerDescriptor {
    fn default() -> Self {
        Self {
            label: None,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: None,
            anisotropy_clamp: 1,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
        }
    }
}

/// GPU resource kind — used by the resource tracker and ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum GpuResourceKind {
    Buffer,
    Texture,
    TextureView,
    Sampler,
    ShaderModule,
    BindGroupLayout,
    BindGroup,
    PipelineLayout,
    RenderPipeline,
    ComputePipeline,
    QuerySet,
    CommandBuffer,
}

impl std::fmt::Display for GpuResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
