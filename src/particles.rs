use crate::types::{Vertex, Uniforms, Particle, ParticleInstance};
use glam::{Mat4, Vec3};
use rand::Rng;
use wgpu::util::DeviceExt;

pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    pub max_particles: usize,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl ParticleSystem {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        // Quad geometry for particles
        let particle_vertices = vec![
            Vertex { position: [-0.5, -0.5, 0.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5, 0.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [ 0.5,  0.5, 0.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [-0.5,  0.5, 0.0], normal: [0.0, 0.0, 1.0] },
        ];
        let particle_indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Vertex Buffer"),
            contents: bytemuck::cast_slice(&particle_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Index Buffer"),
            contents: bytemuck::cast_slice(&particle_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let max_particles = 5000usize;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Instance Buffer"),
            size: (max_particles * std::mem::size_of::<ParticleInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() }],
            label: Some("particle_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/particle.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Pipeline Layout"),
            bind_group_layouts: &[bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), ParticleInstance::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
                        alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, strip_index_format: None, front_face: wgpu::FrontFace::Ccw, cull_mode: None, polygon_mode: wgpu::PolygonMode::Fill, unclipped_depth: false, conservative: false },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: false, depth_compare: wgpu::CompareFunction::Less, stencil: wgpu::StencilState::default(), bias: wgpu::DepthBiasState::default() }),
            multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            multiview_mask: Default::default(),
            cache: None,
        });

        Self {
            particles: Vec::new(),
            max_particles,
            pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            uniform_buffer,
            bind_group,
        }
    }

    pub fn update(&mut self, dt: f32, time: f32) {
        let mut rng = rand::thread_rng();

        // Update existing with upward drift and lateral turbulence
        self.particles.retain_mut(|p| {
            p.life -= dt;
            if p.life <= 0.0 { return false; }
            // Integrate position
            p.position[0] += p.velocity[0] * dt;
            p.position[1] += p.velocity[1] * dt;
            p.position[2] += p.velocity[2] * dt;
            // Buoyancy upward
            p.velocity[1] += 80.0 * dt;
            // Lateral turbulence (swirl)
            let swirl_amp = 30.0f32;
            let swirl_freq = 3.5f32;
            let angle = p.phase + time * swirl_freq + p.position[1] * 0.01;
            p.velocity[0] += swirl_amp * angle.sin() * dt;
            p.velocity[2] += swirl_amp * angle.cos() * dt;
            // Strict X waver
            let w = (time * p.waver_freq).sin();
            p.velocity[0] += p.waver_amp * w * dt;
            // Mild drag
            p.velocity[0] *= 1.0 - 0.25 * dt;
            p.velocity[2] *= 1.0 - 0.25 * dt;
            // Slight growth over life
            p.size += 2.5 * dt;
            true
        });

        // Spawn rate (denser base; multiple per frame)
        let spawn_rate = 900.0; // particles per second
        let desired = (spawn_rate * dt).floor() as usize;
        for _ in 0..desired {
            if self.particles.len() >= self.max_particles { break; }
            // Disk emitter behind mailbox
            let center = Vec3::new(0.0, -50.0, -300.0);
            let disk_radius = 80.0f32;
            let angle = rng.gen_range(0.0..(std::f32::consts::TAU));
            let r = rng.gen_range(0.0..disk_radius);
            let pos = center + Vec3::new(r * angle.cos(), 0.0, r * angle.sin());

            // Upward-biased velocity
            let upward = Vec3::new(0.0, rng.gen_range(180.0..260.0), 0.0);
            // Mild outward spread
            let radial = (pos - center).normalize_or_zero() * rng.gen_range(15.0..40.0);
            let vel = upward + radial;

            let p = Particle {
                position: [pos.x, pos.y, pos.z],
                velocity: [vel.x, vel.y, vel.z],
                life: rng.gen_range(1.2..2.2),
                max_life: 2.2,
                size: rng.gen_range(6.0..12.0),
                phase: rng.gen_range(0.0..std::f32::consts::TAU),
                waver_amp: rng.gen_range(50.0..120.0),
                waver_freq: rng.gen_range(3.0..7.5),
            };
            self.particles.push(p);
        }
    }

    pub fn render(&mut self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        texture_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        projection: Mat4,
        view: Mat4,
        time: f32,
    ) {
        if self.particles.is_empty() { return; }

        // Write uniforms (camera-only MVP, time in model translation.x)
        let p_mvp = projection * view * Mat4::IDENTITY;
        let uniforms = Uniforms {
            mvp_matrix: p_mvp.to_cols_array_2d(),
            model_matrix: Mat4::from_translation(Vec3::new(time, 0.0, 0.0)).to_cols_array_2d(),
            base_color: [1.0, 0.5, 0.0, 1.0],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Build instances buffer
        let instances: Vec<ParticleInstance> = self.particles.iter().map(|p| {
            let t = p.life / p.max_life;
            let (r, g, b) = if t > 0.7 { (1.0, 0.95, 0.7) } else if t > 0.4 { (1.0, 0.6, 0.2) } else { (1.0, 0.2, 0.05) };
            let size_curve = (t * (1.0 - t)) * 3.2;
            let alpha = (t * 1.3).clamp(0.35, 1.0);
            ParticleInstance { position: p.position, size: p.size * size_curve, color: [r, g, b, alpha] }
        }).collect();
        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Particle Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..6, 0, 0..instances.len() as u32);
    }
}
