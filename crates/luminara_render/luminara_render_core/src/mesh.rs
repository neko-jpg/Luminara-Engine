use bytemuck::{Pod, Zeroable};
use wgpu;
use luminara_core::shared_types::Component;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
}

impl Component for Mesh {
    fn type_name() -> &'static str { "Mesh" }
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            vertex_buffer: None,
            index_buffer: None,
        }
    }

    pub fn triangle() -> Self {
        let vertices = vec![
            Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0, 1.0], uv: [0.5, 0.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0, 1.0], uv: [0.0, 1.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0, 1.0], uv: [1.0, 1.0], normal: [0.0, 0.0, 1.0] },
        ];
        let indices = vec![0, 1, 2];
        Self::new(vertices, indices)
    }

    pub fn quad() -> Self {
        let vertices = vec![
            Vertex { position: [-0.5, 0.5, 0.0], color: [1.0, 1.0, 1.0, 1.0], uv: [0.0, 0.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [-0.5, -0.5, 0.0], color: [1.0, 1.0, 1.0, 1.0], uv: [0.0, 1.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [0.5, -0.5, 0.0], color: [1.0, 1.0, 1.0, 1.0], uv: [1.0, 1.0], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [0.5, 0.5, 0.0], color: [1.0, 1.0, 1.0, 1.0], uv: [1.0, 0.0], normal: [0.0, 0.0, 1.0] },
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];
        Self::new(vertices, indices)
    }

    pub fn cube(size: f32) -> Self {
        let s = size / 2.0;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let add_face = |vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, p1, p2, p3, p4, normal: [f32; 3]| {
            let start = vertices.len() as u32;
            vertices.push(Vertex { position: p1, color: [1.0, 1.0, 1.0, 1.0], uv: [0.0, 0.0], normal });
            vertices.push(Vertex { position: p2, color: [1.0, 1.0, 1.0, 1.0], uv: [1.0, 0.0], normal });
            vertices.push(Vertex { position: p3, color: [1.0, 1.0, 1.0, 1.0], uv: [1.0, 1.0], normal });
            vertices.push(Vertex { position: p4, color: [1.0, 1.0, 1.0, 1.0], uv: [0.0, 1.0], normal });
            indices.extend_from_slice(&[start, start + 1, start + 2, start, start + 2, start + 3]);
        };

        // Front
        add_face(&mut vertices, &mut indices, [-s, s, s], [s, s, s], [s, -s, s], [-s, -s, s], [0.0, 0.0, 1.0]);
        // Back
        add_face(&mut vertices, &mut indices, [s, s, -s], [-s, s, -s], [-s, -s, -s], [s, -s, -s], [0.0, 0.0, -1.0]);
        // Top
        add_face(&mut vertices, &mut indices, [-s, s, -s], [s, s, -s], [s, s, s], [-s, s, s], [0.0, 1.0, 0.0]);
        // Bottom
        add_face(&mut vertices, &mut indices, [-s, -s, s], [s, -s, s], [s, -s, -s], [-s, -s, -s], [0.0, -1.0, 0.0]);
        // Right
        add_face(&mut vertices, &mut indices, [s, s, s], [s, s, -s], [s, -s, -s], [s, -s, s], [1.0, 0.0, 0.0]);
        // Left
        add_face(&mut vertices, &mut indices, [-s, s, -s], [-s, s, s], [-s, -s, s], [-s, -s, -s], [-1.0, 0.0, 0.0]);

        Self::new(vertices, indices)
    }

    pub fn plane(size: f32) -> Self {
        let s = size / 2.0;
        let vertices = vec![
            Vertex { position: [-s, 0.0, s], color: [1.0, 1.0, 1.0, 1.0], uv: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [s, 0.0, s], color: [1.0, 1.0, 1.0, 1.0], uv: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [s, 0.0, -s], color: [1.0, 1.0, 1.0, 1.0], uv: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [-s, 0.0, -s], color: [1.0, 1.0, 1.0, 1.0], uv: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
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

                vertices.push(Vertex {
                    position: [x, y, z],
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv,
                    normal,
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
            self.vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }));
        }

        if !self.indices.is_empty() {
            self.index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            }));
        }
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
}
