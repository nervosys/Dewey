//! GPU context — Vulkan-first device initialisation and resource factory.
//!
//! `GpuContext` owns the GPU device stack and provides creation methods
//! that return agpu resource types.  agpu **prefers Vulkan** by default,
//! falling back to the platform backend when Vulkan is unavailable.

use crate::ontology::*;
use crate::resource::*;
use crate::types::*;

/// Core GPU context — instance, adapter, device, and queue.
///
/// All resource creation goes through this context so that every
/// allocated object is an agpu type with tracking metadata.
pub struct GpuContext {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    backend_preference: BackendPreference,
}

impl GpuContext {
    /// Create a GPU context for the given surface.
    ///
    /// Uses Vulkan-first backend selection: tries `Backends::VULKAN`,
    /// and if no suitable adapter is found, falls back to the platform
    /// default (`Backends::PRIMARY`).
    pub async fn new(
        surface: &wgpu::Surface<'_>,
        preference: BackendPreference,
    ) -> Result<Self, GpuError> {
        let backends = preference.to_backends();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });

        // Try preferred backend first
        let (instance, adapter) = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await
        {
            Some(a) => (instance, a),
            None if matches!(preference, BackendPreference::VulkanPreferred) => {
                // Vulkan unavailable — fall back to platform default
                log::info!("agpu: Vulkan adapter not found, falling back to platform default");
                let fallback_instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::PRIMARY,
                    ..Default::default()
                });
                let adapter = fallback_instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::HighPerformance,
                        compatible_surface: Some(surface),
                        force_fallback_adapter: false,
                    })
                    .await
                    .ok_or(GpuError::NoAdapter)?;
                (fallback_instance, adapter)
            }
            None if matches!(preference, BackendPreference::OpenGLPreferred) => {
                // OpenGL unavailable — fall back to platform default
                log::info!("agpu: OpenGL adapter not found, falling back to platform default");
                let fallback_instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::PRIMARY,
                    ..Default::default()
                });
                let adapter = fallback_instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::HighPerformance,
                        compatible_surface: Some(surface),
                        force_fallback_adapter: false,
                    })
                    .await
                    .ok_or(GpuError::NoAdapter)?;
                (fallback_instance, adapter)
            }
            None => return Err(GpuError::NoAdapter),
        };

        // Request all features the adapter supports (Vulkan 1.3+ features
        // are enabled automatically when the adapter exposes them).
        let supported_features = adapter.features();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("agpu_device"),
                    required_features: supported_features,
                    required_limits: adapter.limits(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|e| GpuError::DeviceRequest(e.to_string()))?;

        log::info!(
            "agpu: GPU initialised — {} ({:?}, {:?})",
            adapter.get_info().name,
            adapter.get_info().backend,
            adapter.get_info().device_type,
        );

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            backend_preference: preference,
        })
    }

    /// Create a GPU context reusing an existing `wgpu::Instance` and surface.
    ///
    /// Use this when the caller already owns the instance that created
    /// the surface — avoids the "Surface does not exist" panic that
    /// occurs when a *second* instance tries to look up the surface.
    pub async fn from_instance(
        instance: wgpu::Instance,
        surface: &wgpu::Surface<'_>,
        preference: BackendPreference,
    ) -> Result<Self, GpuError> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await;

        // Fallback: if preferred backend yielded nothing, try PRIMARY
        let (instance, adapter) = match adapter {
            Some(a) => (instance, a),
            None if matches!(
                preference,
                BackendPreference::VulkanPreferred | BackendPreference::OpenGLPreferred
            ) =>
            {
                log::info!("agpu: preferred adapter not found, falling back to platform default");
                let fallback = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::PRIMARY,
                    ..Default::default()
                });
                let a = fallback
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::HighPerformance,
                        compatible_surface: None, // surface belongs to original instance
                        force_fallback_adapter: false,
                    })
                    .await
                    .ok_or(GpuError::NoAdapter)?;
                (fallback, a)
            }
            None => return Err(GpuError::NoAdapter),
        };

        let supported_features = adapter.features();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("agpu_device"),
                    required_features: supported_features,
                    required_limits: adapter.limits(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|e| GpuError::DeviceRequest(e.to_string()))?;

        log::info!(
            "agpu: GPU initialised — {} ({:?}, {:?})",
            adapter.get_info().name,
            adapter.get_info().backend,
            adapter.get_info().device_type,
        );

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            backend_preference: preference,
        })
    }

    // ── Accessors ───────────────────────────────────────────────────

    /// The wgpu device (needed internally and for advanced escape-hatch).
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    /// The wgpu queue.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    /// The wgpu adapter.
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }
    /// The wgpu instance.
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    /// Adapter name (e.g. "NVIDIA GeForce RTX 4090").
    pub fn adapter_name(&self) -> String {
        self.adapter.get_info().name
    }
    /// Active backend (e.g. "Vulkan", "DX12", "Metal").
    pub fn backend(&self) -> String {
        format!("{:?}", self.adapter.get_info().backend)
    }
    /// Device type (discrete, integrated, software).
    pub fn device_type(&self) -> String {
        format!("{:?}", self.adapter.get_info().device_type)
    }
    /// Driver name.
    pub fn driver(&self) -> String {
        self.adapter.get_info().driver
    }
    /// Driver info string.
    pub fn driver_info(&self) -> String {
        self.adapter.get_info().driver_info
    }
    /// Maximum 2D texture dimension.
    pub fn max_texture_dimension(&self) -> u32 {
        self.device.limits().max_texture_dimension_2d
    }
    /// Maximum buffer size in bytes.
    pub fn max_buffer_size(&self) -> u64 {
        self.device.limits().max_buffer_size
    }
    /// Enabled GPU features.
    pub fn features(&self) -> Features {
        self.device.features()
    }
    /// Device limits.
    pub fn limits(&self) -> Limits {
        self.device.limits()
    }
    /// Backend preference that was requested.
    pub fn backend_preference(&self) -> BackendPreference {
        self.backend_preference
    }

    // ── Resource creation ───────────────────────────────────────────

    /// Create a GPU buffer.
    pub fn create_buffer(&self, desc: &GpuBufferDescriptor) -> GpuBuffer {
        let inner = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: desc.label.as_deref(),
            size: desc.size,
            usage: desc.usage,
            mapped_at_creation: desc.mapped_at_creation,
        });
        GpuBuffer::from_raw(inner, desc)
    }

    /// Create a GPU texture.
    pub fn create_texture(&self, desc: &GpuTextureDescriptor) -> GpuTexture {
        let inner = self.device.create_texture(&wgpu::TextureDescriptor {
            label: desc.label.as_deref(),
            size: desc.size,
            mip_level_count: desc.mip_level_count,
            sample_count: desc.sample_count,
            dimension: desc.dimension,
            format: desc.format,
            usage: desc.usage,
            view_formats: &desc.view_formats,
        });
        GpuTexture::from_raw(inner, desc)
    }

    /// Create a GPU sampler.
    pub fn create_sampler(&self, desc: &GpuSamplerDescriptor) -> GpuSampler {
        let inner = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: desc.label.as_deref(),
            address_mode_u: desc.address_mode_u,
            address_mode_v: desc.address_mode_v,
            address_mode_w: desc.address_mode_w,
            mag_filter: desc.mag_filter,
            min_filter: desc.min_filter,
            mipmap_filter: desc.mipmap_filter,
            compare: desc.compare,
            anisotropy_clamp: desc.anisotropy_clamp,
            lod_min_clamp: desc.lod_min_clamp,
            lod_max_clamp: desc.lod_max_clamp,
            ..Default::default()
        });
        GpuSampler::from_raw(inner, desc.label.clone())
    }

    /// Create a shader module from WGSL source.
    pub fn create_shader_wgsl(&self, label: &str, source: &str) -> GpuShaderModule {
        let inner = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
        GpuShaderModule::from_raw(inner, Some(label.into()), ShaderSourceKind::Wgsl)
    }

    /// Create a bind group layout.
    pub fn create_bind_group_layout(
        &self,
        label: &str,
        entries: &[wgpu::BindGroupLayoutEntry],
    ) -> GpuBindGroupLayout {
        let inner = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(label),
                entries,
            });
        GpuBindGroupLayout::from_raw(inner, Some(label.into()))
    }

    /// Create a bind group.
    pub fn create_bind_group(
        &self,
        label: &str,
        layout: &GpuBindGroupLayout,
        entries: &[wgpu::BindGroupEntry<'_>],
    ) -> GpuBindGroup {
        let inner = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: layout.raw(),
            entries,
        });
        GpuBindGroup::from_raw(inner, Some(label.into()))
    }

    /// Create a pipeline layout.
    pub fn create_pipeline_layout(
        &self,
        label: &str,
        bind_group_layouts: &[&GpuBindGroupLayout],
        push_constant_ranges: &[wgpu::PushConstantRange],
    ) -> GpuPipelineLayout {
        let raw_layouts: Vec<&wgpu::BindGroupLayout> =
            bind_group_layouts.iter().map(|l| l.raw()).collect();
        let inner = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &raw_layouts,
                push_constant_ranges,
            });
        GpuPipelineLayout::from_raw(inner, Some(label.into()))
    }

    /// Create a render pipeline.
    pub fn create_render_pipeline(
        &self,
        desc: &wgpu::RenderPipelineDescriptor<'_>,
    ) -> GpuRenderPipeline {
        let label = desc.label.map(String::from);
        let inner = self.device.create_render_pipeline(desc);
        GpuRenderPipeline::from_raw(inner, label)
    }

    /// Create a compute pipeline.
    pub fn create_compute_pipeline(
        &self,
        desc: &wgpu::ComputePipelineDescriptor<'_>,
    ) -> GpuComputePipeline {
        let label = desc.label.map(String::from);
        let inner = self.device.create_compute_pipeline(desc);
        GpuComputePipeline::from_raw(inner, label)
    }

    /// Create a command encoder.
    pub fn create_command_encoder(&self, label: &str) -> crate::command::GpuCommandEncoder {
        let inner = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
        crate::command::GpuCommandEncoder::from_raw(inner)
    }

    /// Create a query set for timestamp or occlusion queries.
    pub fn create_query_set(&self, label: &str, ty: wgpu::QueryType, count: u32) -> GpuQuerySet {
        let inner = self.device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some(label),
            ty,
            count,
        });
        GpuQuerySet::from_raw(inner, Some(label.into()), count)
    }

    /// Write data to a buffer.
    pub fn write_buffer(&self, buffer: &GpuBuffer, offset: u64, data: &[u8]) {
        self.queue.write_buffer(buffer.raw(), offset, data);
    }

    /// Submit finished command buffers to the GPU.
    pub fn submit(&self, commands: impl IntoIterator<Item = crate::command::GpuCommandBuffer>) {
        self.queue
            .submit(commands.into_iter().map(|c| c.into_inner()));
    }

    /// Write data directly to a texture.
    pub fn write_texture(
        &self,
        texture: wgpu::TexelCopyTextureInfo<'_>,
        data: &[u8],
        data_layout: wgpu::TexelCopyBufferLayout,
        size: Extent3d,
    ) {
        self.queue.write_texture(texture, data, data_layout, size);
    }
}

/// Errors from GPU initialisation.
#[derive(Debug)]
pub enum GpuError {
    NoAdapter,
    DeviceRequest(String),
    SurfaceConfig(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAdapter => write!(f, "No suitable GPU adapter found"),
            Self::DeviceRequest(e) => write!(f, "GPU device request failed: {e}"),
            Self::SurfaceConfig(e) => write!(f, "Surface configuration failed: {e}"),
        }
    }
}

impl std::error::Error for GpuError {}

// ── Ontology ────────────────────────────────────────────────────────

impl Discoverable for GpuContext {
    fn schema(&self) -> WidgetSchema {
        let mut schema = WidgetSchema::new(
            "GpuContext",
            "Vulkan-first GPU device context — adapter, device, queue, and hardware capabilities",
            SemanticRole::System,
        );
        schema.usage_hint =
            Some("GpuContext::new(&surface, BackendPreference::default()).await".into());
        schema.tags = vec![
            "gpu".into(),
            "vulkan".into(),
            "device".into(),
            "adapter".into(),
            "hardware".into(),
        ];
        schema
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Custom("gpu_compute".into()),
            AgentCapability::Custom("gpu_render".into()),
            AgentCapability::Custom("vulkan_preferred".into()),
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::simple(
                "get_adapter_info",
                "Query GPU adapter name, backend, and device type",
                false,
            ),
            AgentAction::simple(
                "get_limits",
                "Query device limits (max texture size, buffer size, etc.)",
                false,
            ),
            AgentAction::simple("get_features", "List enabled GPU features", false),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::System
    }

    fn agent_state(&self) -> serde_json::Value {
        let info = self.adapter.get_info();
        let limits = self.device.limits();
        serde_json::json!({
            "adapter_name": info.name,
            "backend": format!("{:?}", info.backend),
            "device_type": format!("{:?}", info.device_type),
            "driver": info.driver,
            "driver_info": info.driver_info,
            "backend_preference": format!("{:?}", self.backend_preference),
            "limits": {
                "max_texture_dimension_2d": limits.max_texture_dimension_2d,
                "max_buffer_size": limits.max_buffer_size,
                "max_bind_groups": limits.max_bind_groups,
                "max_vertex_buffers": limits.max_vertex_buffers,
                "max_vertex_attributes": limits.max_vertex_attributes,
                "max_compute_workgroup_size_x": limits.max_compute_workgroup_size_x,
                "max_compute_workgroups_per_dimension": limits.max_compute_workgroups_per_dimension,
            }
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "get_adapter_info" => Ok(serde_json::json!({
                "name": self.adapter.get_info().name,
                "backend": format!("{:?}", self.adapter.get_info().backend),
                "device_type": format!("{:?}", self.adapter.get_info().device_type),
                "driver": self.adapter.get_info().driver,
            })),
            "get_limits" => {
                let l = self.device.limits();
                Ok(serde_json::json!({
                    "max_texture_dimension_2d": l.max_texture_dimension_2d,
                    "max_buffer_size": l.max_buffer_size,
                    "max_bind_groups": l.max_bind_groups,
                }))
            }
            "get_features" => {
                let features = self.adapter.features();
                Ok(serde_json::json!({
                    "features": format!("{features:?}"),
                }))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some("gpu_context")
    }

    fn accessibility_label(&self) -> Option<String> {
        Some(format!("GPU: {} ({})", self.adapter_name(), self.backend()))
    }
}
