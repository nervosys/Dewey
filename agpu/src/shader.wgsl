// Agpu — 2D shape rendering shader
//
// Converts pixel coordinates to NDC using a viewport uniform,
// then passes vertex colors through for interpolation.

struct Uniforms {
    viewport: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Convert pixel coordinates to normalised device coordinates.
    // x: [0, viewport.x] → [-1, 1]
    // y: [0, viewport.y] → [1, -1]  (y-down in pixels → y-up in NDC)
    let ndc_x = (in.position.x / uniforms.viewport.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (in.position.y / uniforms.viewport.y) * 2.0;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
