// Fire shader - animated procedural fire effect
// This shader uses fractal Brownian motion (FBM) to create realistic fire patterns

// Vertex input structure matching the Vertex struct in types.rs
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

// Data passed from vertex shader to fragment shader
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
}

// Uniform buffer structure matching the Uniforms struct in types.rs
struct Uniforms {
    mvp_matrix: mat4x4<f32>,      // Model-View-Projection matrix for transforming vertices
    model_matrix: mat4x4<f32>,     // Model matrix for world space calculations
    base_color: vec4<f32>,         // Base color (not used in fire effect, but required for compatibility)
}

// Bind the uniform buffer to group 0, binding 0
@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Vertex shader - transforms vertices and passes data to fragment shader
@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Transform vertex position to clip space
    out.clip_position = uniforms.mvp_matrix * vec4<f32>(model.position, 1.0);
    // Calculate world space position for fire effect
    out.world_position = (uniforms.model_matrix * vec4<f32>(model.position, 1.0)).xyz;
    // Transform normal to world space (needed for potential lighting)
    out.world_normal = normalize((uniforms.model_matrix * vec4<f32>(model.normal, 0.0)).xyz);
    return out;
}

// Pseudo-random function
fn rand(n: vec2f) -> f32 {
    return fract(sin(dot(n, vec2f(12.9898, 78.233))) * 43758.5453);
}

// Hash function - returns vec2 between -1.0 and 1.0
fn hash(p: vec2f) -> vec2f {
    let p2 = vec2f(
        dot(p, vec2f(127.1, 311.7)),
        dot(p, vec2f(269.5, 183.3))
    );
    return -1.0 + 2.0 * fract(sin(p2) * 43758.5453123);
}

// Simplex noise function
fn noise(p: vec2f) -> f32 {
    const K1 = 0.366025404; // (sqrt(3)-1)/2
    const K2 = 0.211324865; // (3-sqrt(3))/6
    
    let i = floor(p + (p.x + p.y) * K1);
    let a = p - i + (i.x + i.y) * K2;
    let o = select(vec2f(0.0, 1.0), vec2f(1.0, 0.0), a.x > a.y);
    let b = a - o + K2;
    let c = a - 1.0 + 2.0 * K2;
    let h = max(0.5 - vec3f(dot(a, a), dot(b, b), dot(c, c)), vec3f(0.0));
    let n = h * h * h * h * vec3f(dot(a, hash(i + 0.0)), dot(b, hash(i + o)), dot(c, hash(i + 1.0)));
    
    return dot(n, vec3f(70.0));
}

// Fractal Brownian Motion
fn fbm(n: vec2f, aspect: f32) -> f32 {
    var total = 0.0;
    var p = n;
    var amplitude = aspect * 0.5;
    
    // Rotation matrix for fbm layers
    let m = mat2x2f(1.6, 1.2, -1.2, 1.6);
    
    for (var i = 0; i < 4; i++) {
        total += noise(p) * amplitude;
        p = m * p;
        amplitude *= 0.5;
    }
    
    return total * 0.5 + 0.5;
}

// Fragment shader - generates the fire effect for each pixel
// Based on Shadertoy fire shader by @301z
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Screen resolution
    let iResolution = vec2f(800.0, 600.0);
    let iTime = in.world_position.x;
    
    // Get fragment coordinates
    let fragCoord = vec2f(in.clip_position.x, iResolution.y - in.clip_position.y);
    
    // Normalize coordinates
    var uv = fragCoord / iResolution.xy;
    
    // Movement vectors for distortion and fire
    let distortionMovement = vec2f(0.01, -0.3);
    let fireMovement = vec2f(0.01, -0.5);
    
    // Parameters
    let normalStrength = 40.0;
    let distortionStrength = 0.1;
    
    // Calculate bump map (normal) for creating displacement
    let uvT = vec2f(uv.x * 0.6, uv.y * 0.3) + distortionMovement * iTime;
    let s = 1.0 / iResolution.x;
    
    let n0 = fbm(uvT, 1.0);
    let n1 = fbm(uvT + vec2f(s, 0.0), 1.0);
    let n2 = fbm(uvT + vec2f(0.0, s), 1.0);
    
    // Calculate normal
    var normal = vec3f(0.0);
    normal.x = (n0 - n1) * normalStrength;
    normal.y = (n0 - n2) * normalStrength;
    normal.z = 1.0;
    normal = normalize(normal);
    
    // Create displacement from normal
    let displacement = clamp(
        normal.xy * distortionStrength,
        vec2f(-1.0),
        vec2f(1.0)
    );
    
    // Apply displacement and fire movement
    let uvT2 = vec2f(uv.x * 0.6, uv.y * 0.5) + displacement + fireMovement * iTime;
    
    // Get fire noise
    let n = fbm(uvT2 * 8.0, 1.0);
    
    // Create vertical gradient - stronger at bottom
    let gradientPower = 5.0 * pow(1.0 - uv.y, 2.0);
    
    // Final noise combined with gradient
    let finalNoise = n * gradientPower;
    
    // Create fire color - red strongest, then green, then blue
    var color = vec3f(0.0);
    let n2 = n * n;
    let n4 = n2 * n2;
    let n6 = n4 * n2;
    color = finalNoise * vec3f(2.0 * n2, 2.0 * n4, n6);
    
    // Clamp and boost
    color = clamp(color * 1.5, vec3f(0.0), vec3f(1.0));
    
    // Alpha based on fire intensity
    let alpha = clamp(finalNoise * 1.2, 0.0, 0.9);
    
    return vec4f(color, alpha);
}