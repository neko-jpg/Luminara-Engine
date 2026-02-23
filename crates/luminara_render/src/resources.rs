//! Resource management (buffers, textures, etc.)

use crate::device::RenderDevice;
use crate::mesh::MeshVertex;
use crate::texture::Texture;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub struct GpuBuffer {
    pub buffer: wgpu::Buffer,
    pub size: wgpu::BufferAddress,
    pub usage: wgpu::BufferUsages,
}

impl GpuBuffer {
    pub fn new(
        device: &RenderDevice,
        size: wgpu::BufferAddress,
        usage: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size,
            usage,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            size,
            usage,
        }
    }

    pub fn from_data<T: bytemuck::Pod>(
        device: &RenderDevice,
        data: &[T],
        usage: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        let size = (std::mem::size_of::<T>() * data.len()) as wgpu::BufferAddress;
        let mut buffer = Self::new(device, size, usage | wgpu::BufferUsages::COPY_DST, label);

        device
            .queue
            .write_buffer(&buffer.buffer, 0, bytemuck::cast_slice(data));

        buffer
    }
}

pub struct MeshBuffer {
    pub vertex_buffer: GpuBuffer,
    pub index_buffer: GpuBuffer,
    pub vertex_count: usize,
    pub index_count: usize,
    pub index_format: wgpu::IndexFormat,
}

impl MeshBuffer {
    pub fn from_mesh(device: &RenderDevice, mesh: &crate::mesh::Mesh) -> Self {
        let vertex_buffer = GpuBuffer::from_data(
            device,
            &mesh.vertices,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            Some("vertex_buffer"),
        );

        let index_format = match mesh.primitive_topology {
            crate::mesh::PrimitiveTopology::TriangleList
            | crate::mesh::PrimitiveTopology::TriangleStrip => wgpu::IndexFormat::Uint32,
            _ => wgpu::IndexFormat::Uint16,
        };

        let indices: Vec<u32> = if matches!(index_format, wgpu::IndexFormat::Uint16) {
            mesh.indices.iter().map(|&i| i as u32).collect()
        } else {
            mesh.indices.clone()
        };

        let index_buffer = GpuBuffer::from_data(
            device,
            &indices,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            Some("index_buffer"),
        );

        Self {
            vertex_buffer,
            index_buffer,
            vertex_count: mesh.vertex_count(),
            index_count: mesh.index_count(),
            index_format,
        }
    }
}

pub struct ResourceManager {
    device: Arc<RenderDevice>,
    mesh_buffers: RwLock<HashMap<u64, Arc<MeshBuffer>>>,
    textures: RwLock<HashMap<String, Arc<Texture>>>,
}

impl ResourceManager {
    pub fn new(device: Arc<RenderDevice>) -> Self {
        Self {
            device,
            mesh_buffers: RwLock::new(HashMap::new()),
            textures: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_or_create_mesh_buffer(
        &self,
        mesh_id: u64,
        mesh: &crate::mesh::Mesh,
    ) -> Arc<MeshBuffer> {
        if let Some(buffer) = self.mesh_buffers.read().get(&mesh_id) {
            return buffer.clone();
        }

        let buffer = Arc::new(MeshBuffer::from_mesh(&self.device, mesh));
        self.mesh_buffers.write().insert(mesh_id, buffer.clone());
        buffer
    }

    pub fn get_or_create_texture(
        &self,
        key: impl Into<String>,
        descriptor: &crate::texture::TextureDescriptor,
    ) -> Arc<Texture> {
        let name = key.into();

        if let Some(texture) = self.textures.read().get(&name) {
            return texture.clone();
        }

        let texture = Arc::new(Texture::new(&self.device, descriptor.clone()));
        self.textures.write().insert(name, texture.clone());
        texture
    }

    pub fn remove_mesh_buffer(&self, mesh_id: u64) {
        self.mesh_buffers.write().remove(&mesh_id);
    }

    pub fn remove_texture(&self, key: &str) {
        self.textures.write().remove(key);
    }

    pub fn clear(&self) {
        self.mesh_buffers.write().clear();
        self.textures.write().clear();
    }
}
