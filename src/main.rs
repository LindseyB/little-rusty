mod types;
mod gltf_loader;
mod input;
mod audio;

use std::sync::Arc;
use types::{Vertex, Uniforms};
use gltf_loader::GltfLoader;
use input::InputHandler;
use audio::AudioSystem;
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
    start_time: std::time::Instant,
    audio_system: AudioSystem,
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

        // Initialize audio system 🎵
        let mut audio_system = AudioSystem::new().expect("Failed to initialize audio system");
        
        // Set volume first
        audio_system.set_volume(0.3); // 30% volume

        // Load glTF file 
        let (vertices, indices, base_color) = GltfLoader::load_gltf("assets/9-5_mailbox/9-5_mailbox.gltf");
        
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
            start_time: std::time::Instant::now(),
            audio_system,
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }
    
    fn load_background_music(&mut self) {
        // LET THE MUSIC PLAY! 🎶
        if let Err(e) = self.audio_system.play_file_looped("assets/251461__joshuaempyre__arcade-music-loop.wav", 1.0) {
            println!("⚠️ Note: Could not load music file: {} (this is normal if you don't have a music file)", e);
        }
    }
    
    // Generate a rainbow color based on elapsed time 🌈
    fn get_rainbow_color(&self) -> wgpu::Color {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let hue = (elapsed * 0.5) % 1.0; // Complete rainbow cycle every 2 seconds
        
        // Convert HSV to RGB (with S=1, V=1 for vibrant colors)
        let c = 1.0;
        let x = c * (1.0 - ((hue * 6.0) % 2.0 - 1.0).abs());
        let m = 0.0;
        
        let (r, g, b) = match (hue * 6.0) as i32 {
            0 => (c, x, 0.0),      // Red to Yellow
            1 => (x, c, 0.0),      // Yellow to Green  
            2 => (0.0, c, x),      // Green to Cyan
            3 => (0.0, x, c),      // Cyan to Blue
            4 => (x, 0.0, c),      // Blue to Magenta
            _ => (c, 0.0, x),      // Magenta to Red
        };
        
        wgpu::Color {
            r: (r + m) as f64,
            g: (g + m) as f64, 
            b: (b + m) as f64,
            a: 1.0,
        }
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
        // Ensure minimum size to prevent crashes
        let width = new_size.width.max(800);
        let height = new_size.height.max(600);
        
        self.size = winit::dpi::PhysicalSize::new(width, height);

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
            let rainbow_color = self.get_rainbow_color();
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Wireframe Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(rainbow_color),
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
        // Create window object with minimum size constraint
        let window = Arc::new(
            event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("Little Rusty")
                    .with_min_inner_size(winit::dpi::LogicalSize::new(800, 600))
            )
            .unwrap(),
        );

        let state = pollster::block_on(State::new(
            event_loop.owned_display_handle(),
            window.clone(),
        ));
        self.state = Some(state);
        
        // Load background music after State is created
        if let Some(state) = self.state.as_mut() {
            state.load_background_music();
        }

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        
        // Handle input events first
        if InputHandler::handle_window_event(&event, event_loop) {
            return;
        }
        
        match event {
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