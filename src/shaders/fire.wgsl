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

// Pseudo-random number generator using hash function
// Takes a 2D coordinate and returns a random value between 0 and 1
fn rand(n: vec2f) -> f32 {
    return fract(sin(dot(n, vec2f(12.9898, 78.233))) * 43758.5453);
}

// Perlin-style noise function
// Creates smooth noise by interpolating random values at grid points
fn noise(n: vec2f) -> f32 {
    const d = vec2f(0.0, 1.0);
    let b = floor(n);  // Grid cell coordinates
    var f = smoothstep(vec2f(0.0), vec2f(1.0), fract(n));  // Smooth interpolation weights
    // Bilinear interpolation of random values at four corners
    return mix(mix(rand(b), rand(b + d.yx), f.x),
               mix(rand(b + d.xy), rand(b + d.yy), f.x), f.y);
}

// Fractal Brownian Motion (FBM) - layers multiple octaves of noise
// Creates complex patterns by summing noise at different scales and amplitudes
fn fbm(n: vec2f, aspect: f32) -> f32 {
    var total = 0.0;
    var amplitude = aspect * 0.5;  // Initial amplitude based on aspect ratio
    var vn = n;
    // Sum 5 octaves of noise with increasing frequency and decreasing amplitude
    for (var i : i32 = 0; i < 5; i++) {
        total += noise(vn) * amplitude;
        vn += vn * 1.7;        // Increase frequency
        amplitude *= 0.5;       // Decrease amplitude
    }
    return total;
}

// Fragment shader - generates the fire effect for each pixel
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Fire color palette - various shades from dark red to bright yellow
    const c1 = vec3f(0.5, 0.0, 0.1);   // Dark red
    const c2 = vec3f(0.9, 0.1, 0.0);   // Bright red
    const c3 = vec3f(0.2, 0.1, 0.7);   // Purple (for depth)
    const c4 = vec3f(1.0, 0.9, 0.1);   // Yellow (hot flames)
    const c5 = vec3f(0.1);              // Dark
    const c6 = vec3f(0.9);              // Bright

    // Use clip position (screen coordinates) for the fire effect
    // For a fullscreen quad, this gives us proper pixel coordinates
    let fragCoord = in.clip_position.xy;
    let iResolution = vec2f(800.0, 600.0);  // Virtual resolution for effect
    let aspect = iResolution.x / iResolution.y;
    
    // Animate based on world position Y (which changes with model matrix)
    // For background quad, add time-based animation
    let iTime = in.world_position.x * 5.0 + in.world_position.y * 3.0;
    
    const speed = vec2f(0.0, 1.0);
    var shift = 1.327 + sin(iTime * 2.0) / 2.4;  // Color shift oscillation
    const alpha = 1.0;
    
    // Distance distortion that oscillates over time
    var dist = 3.5 - sin(iTime * 2.0) / 2.4;
    
    // Calculate UV coordinates
    var uv = fragCoord.xy / iResolution.xy;
    var p = fragCoord.xy * dist / iResolution.xx;
    
    // Add turbulence - small wiggles that create realistic fire movement
    p += sin(p.yx * 4.0 + vec2f(0.2, -0.3) * iTime) * 0.04;
    p += sin(p.yx * 8.0 + vec2f(0.5, 0.1) * iTime) * 0.01;
    
    // Scroll the pattern horizontally
    p.x -= iTime / 1.1;
    
    // Generate multiple layers of FBM with different time offsets and frequencies
    // This creates the complex, turbulent appearance of fire
    var q = fbm(p - iTime * 0.3 + 1.0 * sin(iTime + 0.5) / 2.0, aspect);
    var qb = fbm(p - iTime * 0.4 + 0.1 * cos(iTime) / 2.0, aspect);
    var q2 = fbm(p - iTime * 0.44 - 5.0 * cos(iTime) / 2.0, aspect) - 6.0;
    var q3 = fbm(p - iTime * 0.9 - 10.0 * cos(iTime) / 15.0, aspect) - 4.0;
    var q4 = fbm(p - iTime * 1.4 - 20.0 * sin(iTime) / 14.0, aspect) + 2.0;
    
    // Combine FBM layers with different weights to create depth
    q = (q + qb - 0.4 * q2 - 2.0 * q3 + 0.6 * q4) / 3.8;
    
    // Generate another layer of FBM using the previous result as domain distortion
    var r = vec2f(fbm(p + q / 2.0 + iTime * speed.x - p.x - p.y, aspect), 
                  fbm(p + q - iTime * speed.y, aspect));
    
    // Mix colors based on FBM values to create fire gradient
    var c = mix(c1, c2, fbm(p + r, aspect)) + mix(c3, c4, r.x) - mix(c5, c6, r.y);
    
    // Apply intensity falloff and color correction
    var color = vec3(1.0 / pow(c + 1.61, vec3f(4.0))) * cos(shift * fragCoord.y / iResolution.y);
    
    // Final fire color - bright orange/yellow in hottest areas
    // Using the FBM result and vertical position to create upward-flowing flames
    color = vec3f(1.0, 0.2, 0.05) / (pow((r.y + r.y) * max(0.0, p.y) + 0.1, 4.0));
    
    // Tone mapping to prevent oversaturation
    color = color / (1.0 + max(vec3f(0.0), color));
    
    return vec4f(color.x, color.y, color.z, alpha);
}