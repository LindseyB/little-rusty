mod types;

use std::sync::Arc;
use types::{Vertex, Uniforms};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle},
    window::{Window, WindowId},
};



struct State {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    num_indices: u32,
    rotation: (f32, f32), // (x_rotation, y_rotation)
    base_color: [f32; 4],
}

impl State {
    async fn new(_display: OwnedDisplayHandle, window: Arc<Window>) -> State {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::POLYGON_MODE_LINE,
                ..Default::default()
            })
            .await
            .unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        // Load glTF file 
        let (vertices, indices, base_color) = Self::load_gltf("assets/9-5_mailbox/9-5_mailbox.gltf");
        
        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        // Create index buffer  
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        let num_indices = indices.len() as u32;
        
        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("uniform_bind_group_layout"),
        });
        
        // Create bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });
        
        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Solid Lambert Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/solid_lambert.wgsl").into()),
        });
        
        // Create render pipeline layout
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });
        
        // Create render pipeline for solid rendering
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Solid Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: Default::default(),
            cache: None,
        });

        let state = State {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            num_indices,
            rotation: (0.0, 0.0),
            base_color,
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }
    
    fn load_gltf(path: &str) -> (Vec<Vertex>, Vec<u16>, [f32; 4]) {
        // Try to load the glTF file with proper error handling
        let (gltf, buffers, _images) = match gltf::import(path) {
            Ok(data) => data,
            Err(e) => {
                println!("Failed to load glTF file '{}': {}", path, e);
                println!("Falling back to default cube");
                let (vertices, indices) = Self::create_fallback_cube();
                return (vertices, indices, [0.5, 0.5, 0.5, 1.0]);
            }
        };
        
        // Get material color from first material
        let base_color = if let Some(material) = gltf.materials().next() {
            let pbr = material.pbr_metallic_roughness();
            let color = pbr.base_color_factor();
            println!("🪨 Using material color: [{:.3}, {:.3}, {:.3}, {:.3}]", 
                     color[0], color[1], color[2], color[3]);
            color
        } else {
            [0.8, 0.8, 0.8, 1.0] // Default gray
        };
        
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                // Handle missing buffer data gracefully
                let reader = primitive.reader(|buffer| {
                    if buffer.index() < buffers.len() {
                        Some(&buffers[buffer.index()])
                    } else {
                        None
                    }
                });
                
                // Read positions and normals
                if let Some(positions) = reader.read_positions() {
                    let normals = reader.read_normals();
                    let vertex_offset = vertices.len() as u16;
                    
                    // Collect positions and normals
                    let positions: Vec<[f32; 3]> = positions.collect();
                    let normals: Vec<[f32; 3]> = if let Some(normals) = normals {
                        normals.collect()
                    } else {
                        // Generate simple normals if not present (pointing up)
                        vec![[0.0, 1.0, 0.0]; positions.len()]
                    };
                    
                    // Add vertices with normals
                    for (position, normal) in positions.iter().zip(normals.iter()) {
                        vertices.push(Vertex {
                            position: *position,
                            normal: *normal,
                        });
                    }
                    
                    // Read indices and keep as triangles (no wireframe conversion)
                    if let Some(indices_reader) = reader.read_indices() {
                        let triangle_indices: Vec<u32> = indices_reader.into_u32().collect();
                        
                        // Add triangle indices directly
                        for &index in triangle_indices.iter() {
                            indices.push((index as u16) + vertex_offset);
                        }
                    }
                } else {
                    println!("Warning: Mesh primitive has no position data");
                }
            }
        }

        if vertices.is_empty() {
            println!("No valid geometry found in glTF file, using fallback cube");
            let (vertices, indices) = Self::create_fallback_cube();
            return (vertices, indices, [0.5, 0.5, 0.5, 1.0]);
        }

        // Calculate model dimensions
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for vertex in &vertices {
            min_x = min_x.min(vertex.position[0]);
            max_x = max_x.max(vertex.position[0]);
            min_y = min_y.min(vertex.position[1]);
            max_y = max_y.max(vertex.position[1]);
            min_z = min_z.min(vertex.position[2]);
            max_z = max_z.max(vertex.position[2]);
        }

        let width = max_x - min_x;
        let height = max_y - min_y;
        let depth = max_z - min_z;

        println!("💾 Loaded glTF: {} vertices, {} triangle indices", vertices.len(), indices.len());
        println!("📏 Model dimensions:");
        println!("  Width (X): {:.4} (from {:.4} to {:.4})", width, min_x, max_x);
        println!("  Height (Y): {:.4} (from {:.4} to {:.4})", height, min_y, max_y);
        println!("  Depth (Z): {:.4} (from {:.4} to {:.4})", depth, min_z, max_z);
        println!("  Center: ({:.4}, {:.4}, {:.4})", 
                 (min_x + max_x) / 2.0, 
                 (min_y + max_y) / 2.0, 
                 (min_z + max_z) / 2.0);
        
        (vertices, indices, base_color)
    }
    
    // safety cube!!! 🧊
    fn create_fallback_cube() -> (Vec<Vertex>, Vec<u16>) {
        let vertices = vec![
            // Front face
            Vertex { position: [-1.0, -1.0,  1.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [ 1.0, -1.0,  1.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [ 1.0,  1.0,  1.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [-1.0,  1.0,  1.0], normal: [0.0, 0.0, 1.0] },
            // Back face
            Vertex { position: [-1.0, -1.0, -1.0], normal: [0.0, 0.0, -1.0] },
            Vertex { position: [ 1.0, -1.0, -1.0], normal: [0.0, 0.0, -1.0] },
            Vertex { position: [ 1.0,  1.0, -1.0], normal: [0.0, 0.0, -1.0] },
            Vertex { position: [-1.0,  1.0, -1.0], normal: [0.0, 0.0, -1.0] },
        ];

        let indices = vec![
            // Front face
            0, 1, 2,  2, 3, 0,
            // Back face  
            4, 6, 5,  6, 4, 7,
            // Left face
            4, 0, 3,  3, 7, 4,
            // Right face
            1, 5, 6,  6, 2, 1,
            // Top face
            3, 2, 6,  6, 7, 3,
            // Bottom face
            4, 5, 1,  1, 0, 4,
        ];

        println!("Using fallback cube: {} vertices, {} triangle indices", vertices.len(), indices.len());
        (vertices, indices)
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        if self.size.width > 0 && self.size.height > 0 {
            let surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                // Request compatibility with the sRGB-format texture view we're going to create later.
                view_formats: vec![self.surface_format.add_srgb_suffix()],
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                width: self.size.width,
                height: self.size.height,
                desired_maximum_frame_latency: 2,
                present_mode: wgpu::PresentMode::AutoVsync,
            };
            self.surface.configure(&self.device, &surface_config);
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    fn render(&mut self) {
        // Update rotation for animation
        self.rotation.0 += 0.01; // Rotate around X axis
        self.rotation.1 += 0.01; // Rotate around Y axis
        
        // Update MVP matrix
        let aspect = self.size.width as f32 / self.size.height as f32;
        let projection = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 2000.0);
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 800.0), // Eye position - moved back along Z
            Vec3::new(0.0, 0.0, 0.0),   // Look at center
            Vec3::new(0.0, 1.0, 0.0),     // Up vector
        );
        
        // Apply correct scaling to match original FBX dimensions
        // Original: X=158.61, Y=359.09, Z=149.86
        // Trying different coordinate mapping - height (359.09) to Z axis
        let scale = Mat4::from_scale(Vec3::new(
            158.61 / 2.0,  // X scale factor: 79.305
            149.86 / 2.0,  // Y scale factor: 74.93  
            359.09 / 2.0   // Z scale factor (height): 179.545
        ));
        
        let rotation_x = Mat4::from_rotation_x(self.rotation.0);
        let rotation_y = Mat4::from_rotation_y(self.rotation.1);
        let model = rotation_y * rotation_x * scale;
        let mvp = projection * view * model;
        
        let uniforms = Uniforms {
            mvp_matrix: mvp.to_cols_array_2d(),
            model_matrix: model.to_cols_array_2d(),
            base_color: self.base_color,
        };
        
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Get surface texture
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });
            
        // Create depth texture (needed for 3D rendering)
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.size.width,
                height: self.size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("depth_texture"),
            view_formats: &[],
        });
        
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Wireframe Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
            .create_window(Window::default_attributes().with_title("Little Rusty"))
            .unwrap(),
        );

        let state = pollster::block_on(State::new(
            event_loop.owned_display_handle(),
            window.clone(),
        ));
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed. Stopping 🛑");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
                // Emits a new redraw requested event.
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize(size);
            }
            _ => (),
        }
    }
}

fn main() {
    // wgpu uses `log` for all of our logging, so we initialize a logger with the `env_logger` crate.
    //
    // To change the log level, set the `RUST_LOG` environment variable. See the `env_logger`
    // documentation for more information.
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    // When the current loop iteration finishes, immediately begin a new
    // iteration regardless of whether or not new events are available to
    // process. Preferred for applications that want to render as fast as
    // possible, like games.
    event_loop.set_control_flow(ControlFlow::Poll);

    // When the current loop iteration finishes, suspend the thread until
    // another event arrives. Helps keeping CPU utilization low if nothing
    // is happening, which is preferred if the application might be idling in
    // the background.
    // event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}