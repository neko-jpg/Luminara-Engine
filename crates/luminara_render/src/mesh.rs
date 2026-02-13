use bytemuck::{Pod, Zeroable};
use luminara_asset::Asset;
use luminara_core::shared_types::Component;
use luminara_math::Vec3;
use wgpu;

/// Axis-Aligned Bounding Box for mesh culling
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_vertices(vertices: &[Vertex]) -> Self {
        if vertices.is_empty() {
            return Self {
                min: Vec3::ZERO,
                max: Vec3::ZERO,
            };
        }

        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        for vertex in vertices {
            let pos = Vec3::from_array(vertex.position);
            min = min.min(pos);
            max = max.max(pos);
        }

        Self { min, max }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // UV
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Tangent
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub aabb: AABB,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
}

impl Component for Mesh {
    fn type_name() -> &'static str {
        "Mesh"
    }
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let aabb = AABB::from_vertices(&vertices);
        Self {
            vertices,
            indices,
            aabb,
            vertex_buffer: None,
            index_buffer: None,
        }
    }

    pub fn triangle() -> Self {
        let vertices = vec![
            Vertex {
                position: [0.0, 0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.5, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 1.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 1.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
        ];
        let indices = vec![0, 1, 2];
        Self::new(vertices, indices)
    }

    pub fn quad() -> Self {
        let vertices = vec![
            Vertex {
                position: [-0.5, 0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 1.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 1.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];
        Self::new(vertices, indices)
    }

    pub fn cube(size: f32) -> Self {
        let s = size / 2.0;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let add_face = |vertices: &mut Vec<Vertex>,
                        indices: &mut Vec<u32>,
                        p1,
                        p2,
                        p3,
                        p4,
                        normal: [f32; 3],
                        tangent: [f32; 4]| {
            let start = vertices.len() as u32;
            vertices.push(Vertex {
                position: p1,
                normal,
                uv: [0.0, 0.0],
                tangent,
            });
            vertices.push(Vertex {
                position: p2,
                normal,
                uv: [1.0, 0.0],
                tangent,
            });
            vertices.push(Vertex {
                position: p3,
                normal,
                uv: [1.0, 1.0],
                tangent,
            });
            vertices.push(Vertex {
                position: p4,
                normal,
                uv: [0.0, 1.0],
                tangent,
            });
            indices.extend_from_slice(&[start, start + 1, start + 2, start, start + 2, start + 3]);
        };

        // Front (normal: +Z, tangent: +X)
        add_face(
            &mut vertices,
            &mut indices,
            [-s, s, s],
            [s, s, s],
            [s, -s, s],
            [-s, -s, s],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 0.0, 1.0],
        );
        // Back (normal: -Z, tangent: -X)
        add_face(
            &mut vertices,
            &mut indices,
            [s, s, -s],
            [-s, s, -s],
            [-s, -s, -s],
            [s, -s, -s],
            [0.0, 0.0, -1.0],
            [-1.0, 0.0, 0.0, 1.0],
        );
        // Top (normal: +Y, tangent: +X)
        add_face(
            &mut vertices,
            &mut indices,
            [-s, s, -s],
            [s, s, -s],
            [s, s, s],
            [-s, s, s],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0, 1.0],
        );
        // Bottom (normal: -Y, tangent: +X)
        add_face(
            &mut vertices,
            &mut indices,
            [-s, -s, s],
            [s, -s, s],
            [s, -s, -s],
            [-s, -s, -s],
            [0.0, -1.0, 0.0],
            [1.0, 0.0, 0.0, 1.0],
        );
        // Right (normal: +X, tangent: -Z)
        add_face(
            &mut vertices,
            &mut indices,
            [s, s, s],
            [s, s, -s],
            [s, -s, -s],
            [s, -s, s],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, -1.0, 1.0],
        );
        // Left (normal: -X, tangent: +Z)
        add_face(
            &mut vertices,
            &mut indices,
            [-s, s, -s],
            [-s, s, s],
            [-s, -s, s],
            [-s, -s, -s],
            [-1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 1.0],
        );

        Self::new(vertices, indices)
    }

    pub fn plane(size: f32) -> Self {
        let s = size / 2.0;
        let vertices = vec![
            Vertex {
                position: [-s, 0.0, s],
                normal: [0.0, 1.0, 0.0],
                uv: [0.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [s, 0.0, s],
                normal: [0.0, 1.0, 0.0],
                uv: [1.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [s, 0.0, -s],
                normal: [0.0, 1.0, 0.0],
                uv: [1.0, 1.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-s, 0.0, -s],
                normal: [0.0, 1.0, 0.0],
                uv: [0.0, 1.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];
        Self::new(vertices, indices)
    }

    pub fn sphere(radius: f32, segments: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=segments {
            let phi = std::f32::consts::PI * i as f32 / segments as f32;
            for j in 0..=segments {
                let theta = 2.0 * std::f32::consts::PI * j as f32 / segments as f32;

                let x = radius * phi.sin() * theta.cos();
                let y = radius * phi.cos();
                let z = radius * phi.sin() * theta.sin();

                let normal = [x / radius, y / radius, z / radius];
                let uv = [j as f32 / segments as f32, i as f32 / segments as f32];

                // Compute tangent (derivative with respect to theta)
                let tx = -phi.sin() * theta.sin();
                let ty = 0.0;
                let tz = phi.sin() * theta.cos();
                let tangent_len = (tx * tx + ty * ty + tz * tz).sqrt();
                let tangent = if tangent_len > 0.0001 {
                    [tx / tangent_len, ty / tangent_len, tz / tangent_len, 1.0]
                } else {
                    [1.0, 0.0, 0.0, 1.0]
                };

                vertices.push(Vertex {
                    position: [x, y, z],
                    normal,
                    uv,
                    tangent,
                });
            }
        }

        for i in 0..segments {
            for j in 0..segments {
                let first = i * (segments + 1) + j;
                let second = (i + 1) * (segments + 1) + j;

                indices.push(first);
                indices.push(second);
                indices.push(first + 1);

                indices.push(second);
                indices.push(second + 1);
                indices.push(first + 1);
            }
        }

        Self::new(vertices, indices)
    }

    pub fn upload(&mut self, device: &wgpu::Device) {
        use wgpu::util::DeviceExt;

        if !self.vertices.is_empty() {
            self.vertex_buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&self.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ));
        }

        if !self.indices.is_empty() {
            self.index_buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&self.indices),
                    usage: wgpu::BufferUsages::INDEX,
                },
            ));
        }
    }
}

impl Asset for Mesh {
    fn type_name() -> &'static str
    where
        Self: Sized,
    {
        "Mesh"
    }
}

impl Mesh {
    /// Load a mesh from GLTF/GLB bytes
    pub fn from_gltf(bytes: &[u8]) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let (document, buffers, _images) = gltf::import_slice(bytes)?;

        let mut meshes = Vec::new();

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                // Read positions (required)
                let positions: Vec<[f32; 3]> = reader
                    .read_positions()
                    .ok_or("Missing position attribute")?
                    .collect();

                // Read normals (required for PBR)
                let normals: Vec<[f32; 3]> = reader
                    .read_normals()
                    .ok_or("Missing normal attribute")?
                    .collect();

                // Read UVs (optional, default to [0, 0])
                let uvs: Vec<[f32; 2]> = reader
                    .read_tex_coords(0)
                    .map(|iter| iter.into_f32().collect())
                    .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

                // Read tangents (optional, default to [1, 0, 0, 1])
                let tangents: Vec<[f32; 4]> = reader
                    .read_tangents()
                    .map(|iter| iter.collect())
                    .unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 1.0]; positions.len()]);

                // Build vertices
                let mut vertices = Vec::new();
                for i in 0..positions.len() {
                    vertices.push(Vertex {
                        position: positions[i],
                        normal: normals[i],
                        uv: uvs.get(i).copied().unwrap_or([0.0, 0.0]),
                        tangent: tangents.get(i).copied().unwrap_or([1.0, 0.0, 0.0, 1.0]),
                    });
                }

                // Read indices
                let indices: Vec<u32> = reader
                    .read_indices()
                    .map(|iter| iter.into_u32().collect())
                    .unwrap_or_else(|| (0..vertices.len() as u32).collect());

                meshes.push(Mesh::new(vertices, indices));
            }
        }

        if meshes.is_empty() {
            return Err("No meshes found in GLTF file".into());
        }

        Ok(meshes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_desc() {
        let desc = Vertex::desc();
        assert_eq!(desc.array_stride, std::mem::size_of::<Vertex>() as u64);
        assert_eq!(desc.attributes.len(), 4);
    }

    #[test]
    fn test_triangle_mesh() {
        let mesh = Mesh::triangle();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn test_cube_mesh() {
        let mesh = Mesh::cube(1.0);
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
    }

    #[test]
    fn test_aabb_from_vertices() {
        let vertices = vec![
            Vertex {
                position: [-1.0, -1.0, -1.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 1.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
        ];

        let aabb = AABB::from_vertices(&vertices);
        assert_eq!(aabb.min, Vec3::new(-1.0, -1.0, -1.0));
        assert_eq!(aabb.max, Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.center(), Vec3::ZERO);
        assert_eq!(aabb.extents(), Vec3::ONE);
    }

    #[test]
    fn test_aabb_empty_vertices() {
        let vertices = vec![];
        let aabb = AABB::from_vertices(&vertices);
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ZERO);
    }
}
