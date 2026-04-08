//! Command encoding — wrappers for command encoder, render pass, and
//! compute pass that keep the API within the agpu type system.

use crate::resource::*;
use crate::types::*;

// ── GpuCommandEncoder ───────────────────────────────────────────────

/// A command encoder — records GPU commands into a command buffer.
pub struct GpuCommandEncoder {
    inner: wgpu::CommandEncoder,
}

impl GpuCommandEncoder {
    pub(crate) fn from_raw(inner: wgpu::CommandEncoder) -> Self {
        Self { inner }
    }

    /// Begin a render pass.
    pub fn begin_render_pass<'a>(
        &'a mut self,
        desc: &wgpu::RenderPassDescriptor<'a>,
    ) -> GpuRenderPass<'a> {
        let inner = self.inner.begin_render_pass(desc);
        GpuRenderPass::from_raw(inner)
    }

    /// Begin a compute pass.
    pub fn begin_compute_pass(
        &mut self,
        desc: &wgpu::ComputePassDescriptor<'_>,
    ) -> GpuComputePass<'_> {
        let inner = self.inner.begin_compute_pass(desc);
        GpuComputePass::from_raw(inner)
    }

    /// Copy data between buffers.
    pub fn copy_buffer_to_buffer(
        &mut self,
        source: &GpuBuffer,
        source_offset: BufferAddress,
        destination: &GpuBuffer,
        destination_offset: BufferAddress,
        size: BufferAddress,
    ) {
        self.inner.copy_buffer_to_buffer(
            source.raw(),
            source_offset,
            destination.raw(),
            destination_offset,
            size,
        );
    }

    /// Copy buffer data to a texture.
    pub fn copy_buffer_to_texture(
        &mut self,
        source: wgpu::TexelCopyBufferInfo<'_>,
        destination: wgpu::TexelCopyTextureInfo<'_>,
        copy_size: Extent3d,
    ) {
        self.inner
            .copy_buffer_to_texture(source, destination, copy_size);
    }

    /// Copy texture data to a buffer.
    pub fn copy_texture_to_buffer(
        &mut self,
        source: wgpu::TexelCopyTextureInfo<'_>,
        destination: wgpu::TexelCopyBufferInfo<'_>,
        copy_size: Extent3d,
    ) {
        self.inner
            .copy_texture_to_buffer(source, destination, copy_size);
    }

    /// Copy between textures.
    pub fn copy_texture_to_texture(
        &mut self,
        source: wgpu::TexelCopyTextureInfo<'_>,
        destination: wgpu::TexelCopyTextureInfo<'_>,
        copy_size: Extent3d,
    ) {
        self.inner
            .copy_texture_to_texture(source, destination, copy_size);
    }

    /// Finish recording and produce a command buffer for submission.
    pub fn finish(self) -> GpuCommandBuffer {
        GpuCommandBuffer {
            inner: self.inner.finish(),
        }
    }

    /// Access the underlying wgpu encoder.
    pub fn raw(&self) -> &wgpu::CommandEncoder {
        &self.inner
    }

    /// Access the underlying wgpu encoder mutably.
    pub fn raw_mut(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.inner
    }
}

// ── GpuCommandBuffer ────────────────────────────────────────────────

/// A finished command buffer ready for queue submission.
pub struct GpuCommandBuffer {
    pub(crate) inner: wgpu::CommandBuffer,
}

impl GpuCommandBuffer {
    /// Consume and return the inner wgpu command buffer.
    pub fn into_inner(self) -> wgpu::CommandBuffer {
        self.inner
    }
    /// Consume and return the inner wgpu command buffer (alias).
    pub fn raw(self) -> wgpu::CommandBuffer {
        self.inner
    }
}

// ── GpuRenderPass ───────────────────────────────────────────────────

/// A render pass — encodes draw commands for the GPU.
pub struct GpuRenderPass<'a> {
    inner: wgpu::RenderPass<'a>,
}

impl<'a> GpuRenderPass<'a> {
    pub(crate) fn from_raw(inner: wgpu::RenderPass<'a>) -> Self {
        Self { inner }
    }

    /// Set the active render pipeline.
    pub fn set_pipeline(&mut self, pipeline: &'a GpuRenderPipeline) {
        self.inner.set_pipeline(pipeline.raw());
    }

    /// Set a bind group.
    pub fn set_bind_group(&mut self, index: u32, bind_group: &'a GpuBindGroup, offsets: &[u32]) {
        self.inner.set_bind_group(index, bind_group.raw(), offsets);
    }

    /// Set a vertex buffer.
    pub fn set_vertex_buffer(&mut self, slot: u32, buffer: &'a GpuBuffer) {
        self.inner.set_vertex_buffer(slot, buffer.raw().slice(..));
    }

    /// Set vertex buffer with a custom range.
    pub fn set_vertex_buffer_range(&mut self, slot: u32, buffer_slice: wgpu::BufferSlice<'a>) {
        self.inner.set_vertex_buffer(slot, buffer_slice);
    }

    /// Set the index buffer.
    pub fn set_index_buffer(&mut self, buffer: &'a GpuBuffer, format: IndexFormat) {
        self.inner.set_index_buffer(buffer.raw().slice(..), format);
    }

    /// Set index buffer with a custom range.
    pub fn set_index_buffer_range(
        &mut self,
        buffer_slice: wgpu::BufferSlice<'a>,
        format: IndexFormat,
    ) {
        self.inner.set_index_buffer(buffer_slice, format);
    }

    /// Draw primitives.
    pub fn draw(&mut self, vertices: std::ops::Range<u32>, instances: std::ops::Range<u32>) {
        self.inner.draw(vertices, instances);
    }

    /// Draw indexed primitives.
    pub fn draw_indexed(
        &mut self,
        indices: std::ops::Range<u32>,
        base_vertex: i32,
        instances: std::ops::Range<u32>,
    ) {
        self.inner.draw_indexed(indices, base_vertex, instances);
    }

    /// Draw indirect (GPU-driven rendering).
    pub fn draw_indirect(&mut self, buffer: &'a GpuBuffer, offset: BufferAddress) {
        self.inner.draw_indirect(buffer.raw(), offset);
    }

    /// Draw indexed-indirect.
    pub fn draw_indexed_indirect(&mut self, buffer: &'a GpuBuffer, offset: BufferAddress) {
        self.inner.draw_indexed_indirect(buffer.raw(), offset);
    }

    /// Set the viewport.
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32, min_depth: f32, max_depth: f32) {
        self.inner.set_viewport(x, y, w, h, min_depth, max_depth);
    }

    /// Set the scissor rectangle.
    pub fn set_scissor_rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        self.inner.set_scissor_rect(x, y, w, h);
    }

    /// Set the stencil reference value.
    pub fn set_stencil_reference(&mut self, reference: u32) {
        self.inner.set_stencil_reference(reference);
    }

    /// Set the blend constant.
    pub fn set_blend_constant(&mut self, color: Color) {
        self.inner.set_blend_constant(color);
    }

    /// Set push constants.
    pub fn set_push_constants(&mut self, stages: ShaderStages, offset: u32, data: &[u8]) {
        self.inner.set_push_constants(stages, offset, data);
    }

    /// Access the underlying wgpu render pass.
    pub fn raw(&self) -> &wgpu::RenderPass<'a> {
        &self.inner
    }

    /// Access the underlying wgpu render pass mutably.
    pub fn raw_mut(&mut self) -> &mut wgpu::RenderPass<'a> {
        &mut self.inner
    }
}

// ── GpuComputePass ──────────────────────────────────────────────────

/// A compute pass — encodes dispatch commands for GPGPU workloads.
pub struct GpuComputePass<'a> {
    inner: wgpu::ComputePass<'a>,
}

impl<'a> GpuComputePass<'a> {
    pub(crate) fn from_raw(inner: wgpu::ComputePass<'a>) -> Self {
        Self { inner }
    }

    /// Set the active compute pipeline.
    pub fn set_pipeline(&mut self, pipeline: &'a GpuComputePipeline) {
        self.inner.set_pipeline(pipeline.raw());
    }

    /// Set a bind group for the compute pipeline.
    pub fn set_bind_group(&mut self, index: u32, bind_group: &'a GpuBindGroup, offsets: &[u32]) {
        self.inner.set_bind_group(index, bind_group.raw(), offsets);
    }

    /// Dispatch compute work groups.
    pub fn dispatch_workgroups(&mut self, x: u32, y: u32, z: u32) {
        self.inner.dispatch_workgroups(x, y, z);
    }

    /// Dispatch compute work groups indirectly from a buffer.
    pub fn dispatch_workgroups_indirect(&mut self, buffer: &'a GpuBuffer, offset: BufferAddress) {
        self.inner
            .dispatch_workgroups_indirect(buffer.raw(), offset);
    }

    /// Set push constants.
    pub fn set_push_constants(&mut self, offset: u32, data: &[u8]) {
        self.inner.set_push_constants(offset, data);
    }

    /// Access the underlying wgpu compute pass.
    pub fn raw(&self) -> &wgpu::ComputePass<'a> {
        &self.inner
    }

    /// Access the underlying wgpu compute pass mutably.
    pub fn raw_mut(&mut self) -> &mut wgpu::ComputePass<'a> {
        &mut self.inner
    }
}
