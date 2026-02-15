use std::collections::VecDeque;

pub struct BufferPool {
    free_vertex_buffers: VecDeque<wgpu::Buffer>,
    free_index_buffers: VecDeque<wgpu::Buffer>,
    free_uniform_buffers: VecDeque<wgpu::Buffer>,
    buffer_size: u64,
    /// Stats for monitoring pool efficiency.
    pub stats: BufferPoolStats,
}

/// Tracking stats for buffer pool usage.
#[derive(Debug, Clone, Default)]
pub struct BufferPoolStats {
    /// Total buffers created since initialization.
    pub total_created: u64,
    /// Buffers reused from pool.
    pub total_reused: u64,
    /// Current free pool sizes.
    pub free_vertex: usize,
    pub free_index: usize,
    pub free_uniform: usize,
}

impl BufferPool {
    pub fn new(size: u64) -> Self {
        Self {
            free_vertex_buffers: VecDeque::new(),
            free_index_buffers: VecDeque::new(),
            free_uniform_buffers: VecDeque::new(),
            buffer_size: size,
            stats: BufferPoolStats::default(),
        }
    }

    /// Acquire a vertex buffer from the pool, or create a new one if none are available.
    pub fn acquire_vertex_buffer(&mut self, device: &wgpu::Device) -> wgpu::Buffer {
        if let Some(buf) = self.free_vertex_buffers.pop_front() {
            self.stats.total_reused += 1;
            buf
        } else {
            self.stats.total_created += 1;
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Pooled Vertex Buffer"),
                size: self.buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        }
    }

    /// Release a vertex buffer back to the pool for reuse.
    pub fn release_vertex_buffer(&mut self, buffer: wgpu::Buffer) {
        if buffer.size() == self.buffer_size {
            self.free_vertex_buffers.push_back(buffer);
        }
        // Drop oversized/undersized buffers
    }

    /// Acquire an index buffer from the pool, or create a new one.
    pub fn acquire_index_buffer(&mut self, device: &wgpu::Device) -> wgpu::Buffer {
        if let Some(buf) = self.free_index_buffers.pop_front() {
            self.stats.total_reused += 1;
            buf
        } else {
            self.stats.total_created += 1;
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Pooled Index Buffer"),
                size: self.buffer_size,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        }
    }

    /// Release an index buffer back to the pool.
    pub fn release_index_buffer(&mut self, buffer: wgpu::Buffer) {
        if buffer.size() == self.buffer_size {
            self.free_index_buffers.push_back(buffer);
        }
    }

    /// Acquire a uniform buffer from the pool, or create a new one.
    pub fn acquire_uniform_buffer(&mut self, device: &wgpu::Device) -> wgpu::Buffer {
        if let Some(buf) = self.free_uniform_buffers.pop_front() {
            self.stats.total_reused += 1;
            buf
        } else {
            self.stats.total_created += 1;
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Pooled Uniform Buffer"),
                size: self.buffer_size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        }
    }

    /// Release a uniform buffer back to the pool.
    pub fn release_uniform_buffer(&mut self, buffer: wgpu::Buffer) {
        if buffer.size() == self.buffer_size {
            self.free_uniform_buffers.push_back(buffer);
        }
    }

    /// Release all buffers back to the pool (call at end of frame).
    pub fn release_all_vertex(&mut self, buffers: Vec<wgpu::Buffer>) {
        for buf in buffers {
            self.release_vertex_buffer(buf);
        }
    }

    /// Update stats snapshot.
    pub fn update_stats(&mut self) {
        self.stats.free_vertex = self.free_vertex_buffers.len();
        self.stats.free_index = self.free_index_buffers.len();
        self.stats.free_uniform = self.free_uniform_buffers.len();
    }

    /// Get current pool statistics.
    pub fn stats(&self) -> &BufferPoolStats {
        &self.stats
    }
}

impl Drop for BufferPool {
    fn drop(&mut self) {
        for buf in self.free_vertex_buffers.drain(..) {
            buf.destroy();
        }
        for buf in self.free_index_buffers.drain(..) {
            buf.destroy();
        }
        for buf in self.free_uniform_buffers.drain(..) {
            buf.destroy();
        }
    }
}
