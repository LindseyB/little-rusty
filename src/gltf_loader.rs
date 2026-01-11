use crate::types::Vertex;

pub struct GltfLoader;

impl GltfLoader {
    pub fn load_gltf(path: &str) -> (Vec<Vertex>, Vec<u16>, [f32; 4]) {
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
            [0.5, 0.5, 0.5, 1.0] // Default gray
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
}