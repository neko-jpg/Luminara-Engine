use std::collections::VecDeque;

pub struct BufferPool {
    free_vertex_buffers: VecDeque<wgpu::Buffer>,
    free_index_buffers: VecDeque<wgpu::Buffer>,
    free_uniform_buffers: VecDeque<wgpu::Buffer>,
    buffer_size: u64,
}

impl BufferPool {
    pub fn new(size: u64) -> Self {
        Self {
            free_vertex_buffers: VecDeque::new(),
            free_index_buffers: VecDeque::new(),
            free_uniform_buffers: VecDeque::new(),
            buffer_size: size,
        }
    }

    pub fn acquire_vertex_buffer(&mut self, device: &wgpu::Device) -> wgpu::Buffer {
        self.free_vertex_buffers.pop_front().unwrap_or_else(|| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Pooled Vertex Buffer"),
                size: self.buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        })
    }

    pub fn release_vertex_buffer(&mut self, buffer: wgpu::Buffer) {
        if buffer.size() == self.buffer_size {
            self.free_vertex_buffers.push_back(buffer);
        }
    }

    // Similar for index and uniform...
}
