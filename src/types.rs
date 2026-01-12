use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    pub const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
    
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Uniforms {
    pub mvp_matrix: [[f32; 4]; 4],
    pub model_matrix: [[f32; 4]; 4],
    pub base_color: [f32; 4],
}

// GPU-side instance data for a particle (billboard quad)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ParticleInstance {
    pub position: [f32; 3],
    pub size: f32,
    pub color: [f32; 4],
}

impl ParticleInstance {
    pub const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        2 => Float32x3, // position
        3 => Float32,   // size
        4 => Float32x4, // color
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ParticleInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

// CPU-side particle state
#[derive(Copy, Clone, Debug)]
pub struct Particle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub phase: f32,
    pub waver_amp: f32,
    pub waver_freq: f32,
}