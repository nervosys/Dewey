//! 2D shape renderer — batches geometry into vertex/index buffers.
//!
//! Generates triangle meshes for rects, rounded rects, circles, and
//! lines. All geometry is collected per frame and flushed in a single
//! draw call for efficiency.

use crate::context::GpuContext;
use crate::core::{Color, Position, Rect};
use crate::paint::{Gradient, GradientStop, ImageHandle, Shadow};
use crate::types::TextureFormat;
use crate::vertex::Vertex;

/// Per-frame shape geometry collector.
pub struct ShapeRenderer {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_capacity: usize,
    viewport: [f32; 2],
    sample_count: u32,
}

/// Number of segments used to approximate a circle / arc.
const CIRCLE_SEGMENTS: u32 = 32;

/// Initial buffer capacity (in elements).
const INITIAL_VERTEX_CAPACITY: usize = 4096;
const INITIAL_INDEX_CAPACITY: usize = 8192;

impl ShapeRenderer {
    /// Create a new shape renderer with the given GPU context and surface format.
    pub fn new(
        gpu: &GpuContext,
        format: TextureFormat,
        width: f32,
        height: f32,
        sample_count: u32,
    ) -> Self {
        let device = gpu.device();
        // Uniform buffer for viewport size
        let viewport = [width, height];
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agpu_uniform"),
            size: 8, // vec2<f32>
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("agpu_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("agpu_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("agpu_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("agpu_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("agpu_shape_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::LAYOUT],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agpu_vertex"),
            size: (INITIAL_VERTEX_CAPACITY * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agpu_index"),
            size: (INITIAL_INDEX_CAPACITY * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertices: Vec::with_capacity(INITIAL_VERTEX_CAPACITY),
            indices: Vec::with_capacity(INITIAL_INDEX_CAPACITY),
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            vertex_buffer,
            index_buffer,
            vertex_capacity: INITIAL_VERTEX_CAPACITY,
            index_capacity: INITIAL_INDEX_CAPACITY,
            viewport,
            sample_count,
        }
    }

    /// Clear geometry for a new frame.
    pub fn begin_frame(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    /// Update the viewport size.
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport = [width, height];
    }

    /// Add a filled rectangle.
    pub fn fill_rect(&mut self, rect: Rect, color: Color, corner_radius: f32) {
        if corner_radius <= 0.5 {
            self.push_quad(rect, color);
        } else {
            self.push_rounded_rect(rect, color, corner_radius);
        }
    }

    /// Add a stroked rectangle.
    pub fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32, corner_radius: f32) {
        if corner_radius <= 0.5 {
            // Top
            self.push_quad(Rect::new(rect.x, rect.y, rect.width, width), color);
            // Bottom
            self.push_quad(
                Rect::new(rect.x, rect.y + rect.height - width, rect.width, width),
                color,
            );
            // Left
            self.push_quad(
                Rect::new(rect.x, rect.y + width, width, rect.height - 2.0 * width),
                color,
            );
            // Right
            self.push_quad(
                Rect::new(
                    rect.x + rect.width - width,
                    rect.y + width,
                    width,
                    rect.height - 2.0 * width,
                ),
                color,
            );
        } else {
            self.stroke_rounded_rect(rect, color, width, corner_radius, Color::TRANSPARENT);
        }
    }

    /// Stroke a rounded rectangle with proper inner cutout.
    ///
    /// When `bg_color` is not transparent, the inner region is filled with that
    /// color, producing a proper stroked appearance on any background.
    pub fn stroke_rounded_rect(
        &mut self,
        rect: Rect,
        color: Color,
        width: f32,
        corner_radius: f32,
        bg_color: Color,
    ) {
        // Draw outer rounded rect
        self.push_rounded_rect(rect, color, corner_radius);
        // Draw inner rounded rect to cut out the fill
        let inner_radius = (corner_radius - width).max(0.0);
        let inner = Rect::new(
            rect.x + width,
            rect.y + width,
            rect.width - 2.0 * width,
            rect.height - 2.0 * width,
        );
        if inner.width > 0.0 && inner.height > 0.0 {
            let fill = if bg_color.a > 0.0 {
                bg_color
            } else {
                // Transparent cutout — use fully transparent to punch through
                Color::TRANSPARENT
            };
            self.push_rounded_rect(inner, fill, inner_radius);
        }
    }

    /// Add a filled circle.
    pub fn fill_circle(&mut self, center: Position, radius: f32, color: Color) {
        let base = self.vertices.len() as u32;
        let c = [color.r, color.g, color.b, color.a];

        // Center vertex
        self.vertices
            .push(Vertex::new(center.x, center.y, c[0], c[1], c[2], c[3]));

        for i in 0..CIRCLE_SEGMENTS {
            let angle = (i as f32 / CIRCLE_SEGMENTS as f32) * std::f32::consts::TAU;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            self.vertices
                .push(Vertex::new(x, y, c[0], c[1], c[2], c[3]));
        }

        for i in 0..CIRCLE_SEGMENTS {
            self.indices.push(base); // center
            self.indices.push(base + 1 + i);
            self.indices.push(base + 1 + (i + 1) % CIRCLE_SEGMENTS);
        }
    }

    /// Add a stroked circle.
    pub fn stroke_circle(&mut self, center: Position, radius: f32, color: Color, width: f32) {
        let outer = radius;
        let inner = (radius - width).max(0.0);
        let base = self.vertices.len() as u32;
        let c = [color.r, color.g, color.b, color.a];

        for i in 0..CIRCLE_SEGMENTS {
            let angle = (i as f32 / CIRCLE_SEGMENTS as f32) * std::f32::consts::TAU;
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            // outer vertex
            self.vertices.push(Vertex::new(
                center.x + outer * cos_a,
                center.y + outer * sin_a,
                c[0],
                c[1],
                c[2],
                c[3],
            ));
            // inner vertex
            self.vertices.push(Vertex::new(
                center.x + inner * cos_a,
                center.y + inner * sin_a,
                c[0],
                c[1],
                c[2],
                c[3],
            ));
        }

        for i in 0..CIRCLE_SEGMENTS {
            let i0 = base + i * 2;
            let i1 = base + i * 2 + 1;
            let i2 = base + ((i + 1) % CIRCLE_SEGMENTS) * 2;
            let i3 = base + ((i + 1) % CIRCLE_SEGMENTS) * 2 + 1;
            self.indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }
    }

    /// Fill a rectangle with a gradient (linear or radial).
    ///
    /// Gradient colors are interpolated across the rectangle's vertices.
    pub fn fill_rect_gradient(&mut self, rect: Rect, gradient: &Gradient, corner_radius: f32) {
        match gradient {
            Gradient::Linear { start, end, stops } => {
                if stops.is_empty() {
                    return;
                }
                if stops.len() == 1 {
                    self.fill_rect(rect, stops[0].color, corner_radius);
                    return;
                }
                // Compute gradient direction vector
                let dx = end.x - start.x;
                let dy = end.y - start.y;
                let len_sq = dx * dx + dy * dy;
                if len_sq < 0.0001 {
                    self.fill_rect(rect, stops[0].color, corner_radius);
                    return;
                }
                // For simplicity, render as a series of thin horizontal/vertical strips
                // with interpolated colors. Use a quad-based approach.
                let strip_count = 16u32;
                for i in 0..strip_count {
                    let t0 = i as f32 / strip_count as f32;
                    let t1 = (i + 1) as f32 / strip_count as f32;
                    let c0 = sample_gradient(stops, t0);
                    let c1 = sample_gradient(stops, t1);
                    let avg = Color::rgba(
                        (c0.r + c1.r) * 0.5,
                        (c0.g + c1.g) * 0.5,
                        (c0.b + c1.b) * 0.5,
                        (c0.a + c1.a) * 0.5,
                    );
                    // Determine strip direction from gradient direction
                    let strip_rect = if dx.abs() >= dy.abs() {
                        // Horizontal gradient
                        let x0 = rect.x + rect.width * t0;
                        let x1 = rect.x + rect.width * t1;
                        Rect::new(x0, rect.y, x1 - x0, rect.height)
                    } else {
                        // Vertical gradient
                        let y0 = rect.y + rect.height * t0;
                        let y1 = rect.y + rect.height * t1;
                        Rect::new(rect.x, y0, rect.width, y1 - y0)
                    };
                    self.push_quad(strip_rect, avg);
                }
            }
            Gradient::Radial {
                center,
                radius,
                stops,
            } => {
                if stops.is_empty() {
                    return;
                }
                if stops.len() == 1 {
                    self.fill_rect(rect, stops[0].color, corner_radius);
                    return;
                }
                // Approximate radial gradient as concentric circles
                let ring_count = 12u32;
                for i in (0..ring_count).rev() {
                    let t = (i + 1) as f32 / ring_count as f32;
                    let r = radius * t;
                    let color = sample_gradient(stops, t);
                    self.fill_circle(*center, r, color);
                }
            }
        }
    }

    /// Draw a shadow behind a rectangle.
    pub fn shadow_rect(&mut self, rect: Rect, shadow: &Shadow, corner_radius: f32) {
        let expand = shadow.blur_radius * 0.5;
        let shadow_rect = Rect::new(
            rect.x + shadow.offset_x - expand,
            rect.y + shadow.offset_y - expand,
            rect.width + expand * 2.0,
            rect.height + expand * 2.0,
        );
        self.fill_rect(shadow_rect, shadow.color, corner_radius + expand);
    }

    /// Draw an image (placeholder — textures require a separate pipeline).
    pub fn draw_image(&mut self, _handle: &ImageHandle, _rect: Rect) {
        // Image rendering requires a texture atlas and separate pipeline.
        // This is a placeholder; images are a no-op until the texture pipeline
        // is added in a future version.
    }

    /// Add a line segment.
    pub fn line(&mut self, from: Position, to: Position, color: Color, width: f32) {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        let half = width * 0.5;

        // Perpendicular direction
        let nx = -dy / len * half;
        let ny = dx / len * half;

        let base = self.vertices.len() as u32;
        let c = [color.r, color.g, color.b, color.a];

        self.vertices.push(Vertex::new(
            from.x + nx,
            from.y + ny,
            c[0],
            c[1],
            c[2],
            c[3],
        ));
        self.vertices.push(Vertex::new(
            from.x - nx,
            from.y - ny,
            c[0],
            c[1],
            c[2],
            c[3],
        ));
        self.vertices
            .push(Vertex::new(to.x - nx, to.y - ny, c[0], c[1], c[2], c[3]));
        self.vertices
            .push(Vertex::new(to.x + nx, to.y + ny, c[0], c[1], c[2], c[3]));

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Flush geometry and encode draw commands into the given render pass.
    pub fn flush(&mut self, gpu: &GpuContext, pass: &mut wgpu::RenderPass<'_>) {
        if self.indices.is_empty() {
            return;
        }

        let device = gpu.device();
        let queue = gpu.queue();

        // Upload viewport uniform
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&self.viewport));

        // Grow buffers if needed
        if self.vertices.len() > self.vertex_capacity {
            self.vertex_capacity = self.vertices.len().next_power_of_two();
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("agpu_vertex"),
                size: (self.vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if self.indices.len() > self.index_capacity {
            self.index_capacity = self.indices.len().next_power_of_two();
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("agpu_index"),
                size: (self.index_capacity * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // Upload geometry
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
    }

    /// Current vertex count for statistics.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Current index/triangle count for statistics.
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    /// MSAA sample count configured for this renderer.
    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    // ── internal helpers ────────────────────────────────────────────

    fn push_quad(&mut self, rect: Rect, color: Color) {
        let base = self.vertices.len() as u32;
        let c = [color.r, color.g, color.b, color.a];
        let x0 = rect.x;
        let y0 = rect.y;
        let x1 = rect.x + rect.width;
        let y1 = rect.y + rect.height;

        self.vertices
            .push(Vertex::new(x0, y0, c[0], c[1], c[2], c[3]));
        self.vertices
            .push(Vertex::new(x1, y0, c[0], c[1], c[2], c[3]));
        self.vertices
            .push(Vertex::new(x1, y1, c[0], c[1], c[2], c[3]));
        self.vertices
            .push(Vertex::new(x0, y1, c[0], c[1], c[2], c[3]));

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn push_rounded_rect(&mut self, rect: Rect, color: Color, radius: f32) {
        let r = radius.min(rect.width * 0.5).min(rect.height * 0.5);
        let c = [color.r, color.g, color.b, color.a];
        let base = self.vertices.len() as u32;

        // Center vertex for fan
        let cx = rect.x + rect.width * 0.5;
        let cy = rect.y + rect.height * 0.5;
        self.vertices
            .push(Vertex::new(cx, cy, c[0], c[1], c[2], c[3]));

        // Generate outline vertices: straight edges + corner arcs
        let mut outline: Vec<[f32; 2]> = Vec::with_capacity(4 * 8 + 4);

        // Corner centers
        let corners = [
            (
                rect.x + r,
                rect.y + r,
                std::f32::consts::PI,
                std::f32::consts::FRAC_PI_2 * 3.0,
            ), // top-left
            (
                rect.x + rect.width - r,
                rect.y + r,
                std::f32::consts::FRAC_PI_2 * 3.0,
                std::f32::consts::TAU,
            ), // top-right
            (
                rect.x + rect.width - r,
                rect.y + rect.height - r,
                0.0,
                std::f32::consts::FRAC_PI_2,
            ), // bottom-right
            (
                rect.x + r,
                rect.y + rect.height - r,
                std::f32::consts::FRAC_PI_2,
                std::f32::consts::PI,
            ), // bottom-left
        ];

        let arc_segments = 8u32;
        for (corner_x, corner_y, start_angle, end_angle) in &corners {
            for j in 0..=arc_segments {
                let t =
                    *start_angle + (*end_angle - *start_angle) * (j as f32 / arc_segments as f32);
                outline.push([corner_x + r * t.cos(), corner_y + r * t.sin()]);
            }
        }

        // Add outline vertices
        for pt in &outline {
            self.vertices
                .push(Vertex::new(pt[0], pt[1], c[0], c[1], c[2], c[3]));
        }

        // Triangle fan from center to outline
        let n = outline.len() as u32;
        for i in 0..n {
            self.indices.push(base); // center
            self.indices.push(base + 1 + i);
            self.indices.push(base + 1 + (i + 1) % n);
        }
    }
}

/// Linearly interpolate a gradient's color stops at parameter `t` (0.0–1.0).
fn sample_gradient(stops: &[GradientStop], t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    if stops.is_empty() {
        return Color::WHITE;
    }
    if stops.len() == 1 || t <= stops[0].offset {
        return stops[0].color;
    }
    if t >= stops[stops.len() - 1].offset {
        return stops[stops.len() - 1].color;
    }
    for i in 1..stops.len() {
        if t <= stops[i].offset {
            let prev = &stops[i - 1];
            let next = &stops[i];
            let range = next.offset - prev.offset;
            if range < 0.0001 {
                return next.color;
            }
            let f = (t - prev.offset) / range;
            return Color::rgba(
                prev.color.r + (next.color.r - prev.color.r) * f,
                prev.color.g + (next.color.g - prev.color.g) * f,
                prev.color.b + (next.color.b - prev.color.b) * f,
                prev.color.a + (next.color.a - prev.color.a) * f,
            );
        }
    }
    stops[stops.len() - 1].color
}
