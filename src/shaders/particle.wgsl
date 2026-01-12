// Particle shader
struct Uniforms {
    mvp_matrix: mat4x4<f32>,
    model_matrix: mat4x4<f32>,
    base_color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct InstanceInput {
    @location(2) particle_position: vec3<f32>,
    @location(3) size: f32,
    @location(4) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Axis-aligned billboard elongated vertically for flame-like shape
    let sx = instance.size * 1.0; // slightly wider
    let sy = instance.size * 2.2; // more vertical elongation
    let world_pos = vec3<f32>(
        instance.particle_position.x + vertex.position.x * sx,
        instance.particle_position.y + vertex.position.y * sy,
        instance.particle_position.z
    );

    out.clip_position = uniforms.mvp_matrix * vec4<f32>(world_pos, 1.0);
    out.color = instance.color;
    out.uv = vertex.position.xy + 0.5;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Soft circular sprite
    let d = distance(in.uv, vec2<f32>(0.5, 0.5));
    var alpha = 1.0 - smoothstep(0.22, 0.52, d);
    // Slightly brighter core (component-wise)
    var color = in.color;
    // Flicker using time encoded in model_matrix translation.x
    let time = uniforms.model_matrix[3].x;
    let flicker = 0.80 + 0.20 * sin(16.0 * time + in.uv.x * 10.0 + in.uv.y * 7.0);
    let boost = (1.0 + (1.0 - d) * 0.5) * flicker;
    color = vec4<f32>(color.r * boost, color.g * boost, color.b * boost, color.a * alpha);
    return color;
}
