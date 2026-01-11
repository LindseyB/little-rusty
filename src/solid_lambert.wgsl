// Material shader for solid rendering
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
}

struct Uniforms {
    mvp_matrix: mat4x4<f32>,
    model_matrix: mat4x4<f32>,
    base_color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.mvp_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = (uniforms.model_matrix * vec4<f32>(model.position, 1.0)).xyz;
    out.world_normal = normalize((uniforms.model_matrix * vec4<f32>(model.normal, 0.0)).xyz);
    return out;
}

// Fragment shader with basic lighting
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let light_color = vec3<f32>(1.0, 1.0, 1.0);
    
    // Basic Lambert lighting
    let normal = normalize(in.world_normal);
    let light_intensity = max(dot(normal, light_dir), 0.1); // 0.1 ambient
    
    let final_color = uniforms.base_color.rgb * light_color * light_intensity;
    return vec4<f32>(final_color, uniforms.base_color.a);
}