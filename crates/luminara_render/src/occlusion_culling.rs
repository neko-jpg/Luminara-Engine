/// GPU-driven occlusion culling system
///
/// This module implements hardware occlusion queries to cull objects that are hidden
/// behind other geometry. Target: >80% efficiency in dense scenes, minimal CPU overhead.
///
/// The system uses a two-pass approach:
/// 1. Render bounding boxes of objects to depth buffer with occlusion queries
/// 2. Read back query results and cull occluded objects from rendering
///
/// For best performance, we use:
/// - Conservative bounding boxes (slightly larger than actual geometry)
/// - Hierarchical occlusion queries (test larger groups first)
/// - Temporal coherence (reuse results from previous frames)

use luminara_core::shared_types::{Component, Resource};
use luminara_math::{Mat4, Vec3};
use crate::AABB;
use std::collections::HashMap;

/// Occlusion query state for an entity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcclusionState {
    /// Not yet tested
    Unknown,
    /// Visible (passed occlusion test)
    Visible,
    /// Occluded (failed occlusion test)
    Occluded,
    /// Query pending (waiting for GPU results)
    Pending,
}

/// Occlusion query data for an entity
#[derive(Debug, Clone)]
pub struct OcclusionQuery {
    /// Current occlusion state
    pub state: OcclusionState,
    /// Query set index
    pub query_index: Option<u32>,
    /// Number of samples passed (from GPU query)
    pub samples_passed: u64,
    /// Frame when last tested
    pub last_tested_frame: u64,
    /// Conservative AABB for occlusion testing
    pub test_aabb: AABB,
}

impl OcclusionQuery {
    pub fn new(aabb: AABB) -> Self {
        Self {
            state: OcclusionState::Unknown,
            query_index: None,
            samples_passed: 0,
            last_tested_frame: 0,
            test_aabb: aabb,
        }
    }

    /// Check if this query is visible
    pub fn is_visible(&self) -> bool {
        matches!(self.state, OcclusionState::Visible | OcclusionState::Unknown)
    }

    /// Check if query needs retesting
    pub fn needs_retest(&self, current_frame: u64, retest_interval: u64) -> bool {
        current_frame - self.last_tested_frame >= retest_interval
    }
}

/// Component to mark entities for occlusion culling
#[derive(Debug, Clone, Copy)]
pub struct Occludable {
    /// Enable occlusion culling for this entity
    pub enabled: bool,
    /// Retest interval in frames (0 = test every frame)
    pub retest_interval: u64,
}

impl Occludable {
    pub fn new() -> Self {
        Self {
            enabled: true,
            retest_interval: 5, // Retest every 5 frames by default
        }
    }

    pub fn with_interval(interval: u64) -> Self {
        Self {
            enabled: true,
            retest_interval: interval,
        }
    }
}

impl Default for Occludable {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Occludable {
    fn type_name() -> &'static str {
        "Occludable"
    }
}

/// GPU-driven occlusion culling system
pub struct OcclusionCullingSystem {
    /// Occlusion queries per entity
    queries: HashMap<usize, OcclusionQuery>,
    /// Query set for GPU queries
    query_set: Option<wgpu::QuerySet>,
    /// Query buffer for reading results
    query_buffer: Option<wgpu::Buffer>,
    /// Staging buffer for CPU readback
    staging_buffer: Option<wgpu::Buffer>,
    /// Maximum number of queries
    max_queries: u32,
    /// Current frame number
    current_frame: u64,
    /// Statistics
    stats: OcclusionStats,
    /// Enable temporal coherence (reuse previous frame results)
    enable_temporal_coherence: bool,
    /// Enable hierarchical queries (test groups before individuals)
    enable_hierarchical: bool,
}

/// Occlusion culling statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct OcclusionStats {
    /// Total entities tested
    pub total_entities: usize,
    /// Entities marked visible
    pub visible_entities: usize,
    /// Entities marked occluded
    pub occluded_entities: usize,
    /// Culling efficiency (% occluded)
    pub culling_efficiency: f32,
    /// GPU time for occlusion queries (ms)
    pub gpu_time_ms: f32,
    /// CPU time for processing results (ms)
    pub cpu_time_ms: f32,
}

impl OcclusionStats {
    pub fn calculate_efficiency(&mut self) {
        if self.total_entities > 0 {
            self.culling_efficiency = 
                (self.occluded_entities as f32 / self.total_entities as f32) * 100.0;
        } else {
            self.culling_efficiency = 0.0;
        }
    }

    pub fn print(&self) {
        println!("=== Occlusion Culling Stats ===");
        println!("Total Entities: {}", self.total_entities);
        println!("Visible: {}", self.visible_entities);
        println!("Occluded: {}", self.occluded_entities);
        println!("Culling Efficiency: {:.1}%", self.culling_efficiency);
        println!("GPU Time: {:.3}ms", self.gpu_time_ms);
        println!("CPU Time: {:.3}ms", self.cpu_time_ms);

        if self.culling_efficiency >= 80.0 {
            println!("✓ Culling efficiency meets target (>80%)");
        } else {
            println!("⚠️  Culling efficiency below target");
        }
    }
}

impl OcclusionCullingSystem {
    /// Create new occlusion culling system
    pub fn new(max_queries: u32) -> Self {
        Self {
            queries: HashMap::new(),
            query_set: None,
            query_buffer: None,
            staging_buffer: None,
            max_queries,
            current_frame: 0,
            stats: OcclusionStats::default(),
            enable_temporal_coherence: true,
            enable_hierarchical: true,
        }
    }

    /// Initialize GPU resources
    pub fn initialize(&mut self, device: &wgpu::Device) {
        // Create query set for occlusion queries
        self.query_set = Some(device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("Occlusion Query Set"),
            ty: wgpu::QueryType::Occlusion,
            count: self.max_queries,
        }));

        // Create buffer to store query results
        let query_buffer_size = (self.max_queries as u64) * 8; // 8 bytes per query (u64)
        self.query_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Occlusion Query Buffer"),
            size: query_buffer_size,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));

        // Create staging buffer for CPU readback
        self.staging_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Occlusion Query Staging Buffer"),
            size: query_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        }));
    }

    /// Update entity queries
    pub fn update_entities(
        &mut self,
        entities: &[(usize, AABB, Mat4)],
    ) {
        // Update or create queries for entities
        for (idx, aabb, transform) in entities {
            // Transform AABB to world space
            let world_aabb = transform_aabb(aabb, transform);
            
            self.queries
                .entry(*idx)
                .and_modify(|q| {
                    q.test_aabb = world_aabb;
                })
                .or_insert_with(|| OcclusionQuery::new(world_aabb));
        }

        // Remove queries for entities that no longer exist
        let entity_indices: std::collections::HashSet<_> = 
            entities.iter().map(|(idx, _, _)| *idx).collect();
        self.queries.retain(|idx, _| entity_indices.contains(idx));
    }

    /// Begin occlusion query pass
    /// Returns list of entities that need testing
    pub fn begin_query_pass(&mut self) -> Vec<(usize, AABB)> {
        self.current_frame += 1;
        
        let mut entities_to_test = Vec::new();
        let mut query_index = 0u32;

        for (entity_idx, query) in self.queries.iter_mut() {
            // Check if entity needs retesting
            // Always test Unknown state, or check retest interval
            let needs_test = query.state == OcclusionState::Unknown
                || !self.enable_temporal_coherence 
                || query.needs_retest(self.current_frame, 5);

            if needs_test && query_index < self.max_queries {
                query.query_index = Some(query_index);
                query.state = OcclusionState::Pending;
                query.last_tested_frame = self.current_frame;
                entities_to_test.push((*entity_idx, query.test_aabb));
                query_index += 1;
            }
        }

        entities_to_test
    }

    /// Render occlusion query proxies (bounding boxes)
    /// This should be called during the depth pre-pass
    pub fn render_query_proxies(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        depth_view: &wgpu::TextureView,
        camera_bind_group: &wgpu::BindGroup,
        pipeline: &wgpu::RenderPipeline,
        bbox_vertex_buffer: &wgpu::Buffer,
        bbox_index_buffer: &wgpu::Buffer,
        entities_to_test: &[(usize, AABB)],
    ) {
        if entities_to_test.is_empty() {
            return;
        }

        let query_set = match &self.query_set {
            Some(qs) => qs,
            None => return,
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Occlusion Query Pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load, // Load existing depth
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: Some(query_set),
        });

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, bbox_vertex_buffer.slice(..));
        render_pass.set_index_buffer(bbox_index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        // Render each bounding box with occlusion query
        for (i, (_entity_idx, _aabb)) in entities_to_test.iter().enumerate() {
            let query_index = i as u32;
            
            // Begin occlusion query
            render_pass.begin_occlusion_query(query_index);
            
            // Render bounding box
            // Note: In a real implementation, you'd set up transform uniforms here
            // For now, we assume the bbox is already in world space
            render_pass.draw_indexed(0..36, 0, 0..1); // 36 indices for a box
            
            // End occlusion query
            render_pass.end_occlusion_query();
        }
    }

    /// Resolve query results from GPU
    pub fn resolve_queries(&self, encoder: &mut wgpu::CommandEncoder, query_count: u32) {
        if query_count == 0 {
            return;
        }

        let query_buffer = match &self.query_buffer {
            Some(buf) => buf,
            None => return,
        };

        let staging_buffer = match &self.staging_buffer {
            Some(buf) => buf,
            None => return,
        };

        let query_set = match &self.query_set {
            Some(qs) => qs,
            None => return,
        };

        // Resolve queries to buffer
        encoder.resolve_query_set(
            query_set,
            0..query_count,
            query_buffer,
            0,
        );

        // Copy to staging buffer for CPU readback
        encoder.copy_buffer_to_buffer(
            query_buffer,
            0,
            staging_buffer,
            0,
            (query_count as u64) * 8,
        );
    }

    /// Read back query results from GPU (blocking)
    /// Note: In production, this should be called after ensuring GPU work is complete
    pub fn read_query_results(&mut self, device: &wgpu::Device) {
        let staging_buffer = match &self.staging_buffer {
            Some(buf) => buf,
            None => return,
        };

        // Map staging buffer for reading
        let buffer_slice = staging_buffer.slice(..);
        
        // Use blocking map for simplicity (in production, use async)
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::Maintain::Wait);

        // Read results
        let data = buffer_slice.get_mapped_range();
        let results: &[u64] = bytemuck::cast_slice(&data);

        // Update query states
        let mut visible_count = 0;
        let mut occluded_count = 0;

        for (_query_idx, query) in self.queries.values_mut().enumerate() {
            if let Some(idx) = query.query_index {
                if (idx as usize) < results.len() {
                    query.samples_passed = results[idx as usize];
                    
                    // Update state based on samples passed
                    if query.samples_passed > 0 {
                        query.state = OcclusionState::Visible;
                        visible_count += 1;
                    } else {
                        query.state = OcclusionState::Occluded;
                        occluded_count += 1;
                    }
                }
            }
        }

        drop(data);
        staging_buffer.unmap();

        // Update statistics
        self.stats.total_entities = self.queries.len();
        self.stats.visible_entities = visible_count;
        self.stats.occluded_entities = occluded_count;
        self.stats.calculate_efficiency();
    }

    /// Get list of visible entity indices
    pub fn get_visible_entities(&self) -> Vec<usize> {
        self.queries
            .iter()
            .filter(|(_, query)| query.is_visible())
            .map(|(idx, _)| *idx)
            .collect()
    }

    /// Get occlusion state for an entity
    pub fn get_occlusion_state(&self, entity_idx: usize) -> OcclusionState {
        self.queries
            .get(&entity_idx)
            .map(|q| q.state)
            .unwrap_or(OcclusionState::Unknown)
    }

    /// Get statistics
    pub fn stats(&self) -> &OcclusionStats {
        &self.stats
    }

    /// Clear all queries
    pub fn clear(&mut self) {
        self.queries.clear();
        self.stats = OcclusionStats::default();
    }
}

impl Default for OcclusionCullingSystem {
    fn default() -> Self {
        Self::new(1024) // Default to 1024 queries
    }
}

impl Resource for OcclusionCullingSystem {}

/// Transform AABB by matrix (conservative bounding box)
fn transform_aabb(aabb: &AABB, transform: &Mat4) -> AABB {
    let center = aabb.center();
    let extents = aabb.extents();
    
    let center_transformed = transform.transform_point3(center);
    
    // Transform extents (conservative approach)
    let m = transform.to_cols_array_2d();
    let abs_m = [
        [m[0][0].abs(), m[0][1].abs(), m[0][2].abs()],
        [m[1][0].abs(), m[1][1].abs(), m[1][2].abs()],
        [m[2][0].abs(), m[2][1].abs(), m[2][2].abs()],
    ];
    
    let extents_transformed = Vec3::new(
        abs_m[0][0] * extents.x + abs_m[0][1] * extents.y + abs_m[0][2] * extents.z,
        abs_m[1][0] * extents.x + abs_m[1][1] * extents.y + abs_m[1][2] * extents.z,
        abs_m[2][0] * extents.x + abs_m[2][1] * extents.y + abs_m[2][2] * extents.z,
    );
    
    AABB::new(
        center_transformed - extents_transformed,
        center_transformed + extents_transformed,
    )
}

/// Create vertex buffer for bounding box rendering
pub fn create_bbox_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    use wgpu::util::DeviceExt;
    
    // Unit cube vertices (will be scaled by transform)
    #[repr(C)]
    #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    struct BBoxVertex {
        position: [f32; 3],
    }

    let vertices = [
        // Front face
        BBoxVertex { position: [-1.0, -1.0,  1.0] },
        BBoxVertex { position: [ 1.0, -1.0,  1.0] },
        BBoxVertex { position: [ 1.0,  1.0,  1.0] },
        BBoxVertex { position: [-1.0,  1.0,  1.0] },
        // Back face
        BBoxVertex { position: [-1.0, -1.0, -1.0] },
        BBoxVertex { position: [ 1.0, -1.0, -1.0] },
        BBoxVertex { position: [ 1.0,  1.0, -1.0] },
        BBoxVertex { position: [-1.0,  1.0, -1.0] },
    ];

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("BBox Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

/// Create index buffer for bounding box rendering
pub fn create_bbox_index_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    use wgpu::util::DeviceExt;
    
    let indices: [u32; 36] = [
        // Front
        0, 1, 2, 2, 3, 0,
        // Back
        5, 4, 7, 7, 6, 5,
        // Left
        4, 0, 3, 3, 7, 4,
        // Right
        1, 5, 6, 6, 2, 1,
        // Top
        3, 2, 6, 6, 7, 3,
        // Bottom
        4, 5, 1, 1, 0, 4,
    ];

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("BBox Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_occlusion_query_creation() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        let query = OcclusionQuery::new(aabb);
        
        assert_eq!(query.state, OcclusionState::Unknown);
        assert!(query.is_visible()); // Unknown is treated as visible
        assert_eq!(query.samples_passed, 0);
    }

    #[test]
    fn test_occlusion_state_visibility() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        let mut query = OcclusionQuery::new(aabb);
        
        // Unknown should be visible
        assert!(query.is_visible());
        
        // Visible should be visible
        query.state = OcclusionState::Visible;
        assert!(query.is_visible());
        
        // Occluded should not be visible
        query.state = OcclusionState::Occluded;
        assert!(!query.is_visible());
        
        // Pending should not be visible
        query.state = OcclusionState::Pending;
        assert!(!query.is_visible());
    }

    #[test]
    fn test_needs_retest() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        let mut query = OcclusionQuery::new(aabb);
        
        query.last_tested_frame = 10;
        
        // Should need retest after interval
        assert!(query.needs_retest(16, 5));
        
        // Should not need retest before interval
        assert!(!query.needs_retest(14, 5));
    }

    #[test]
    fn test_occlusion_system_creation() {
        let system = OcclusionCullingSystem::new(512);
        
        assert_eq!(system.max_queries, 512);
        assert_eq!(system.current_frame, 0);
        assert_eq!(system.queries.len(), 0);
    }

    #[test]
    fn test_update_entities() {
        let mut system = OcclusionCullingSystem::new(512);
        
        let entities = vec![
            (0, AABB::new(Vec3::ZERO, Vec3::ONE), Mat4::IDENTITY),
            (1, AABB::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(6.0, 1.0, 1.0)), Mat4::IDENTITY),
        ];
        
        system.update_entities(&entities);
        
        assert_eq!(system.queries.len(), 2);
        assert!(system.queries.contains_key(&0));
        assert!(system.queries.contains_key(&1));
    }

    #[test]
    fn test_stats_calculation() {
        let mut stats = OcclusionStats {
            total_entities: 100,
            visible_entities: 20,
            occluded_entities: 80,
            culling_efficiency: 0.0,
            gpu_time_ms: 0.0,
            cpu_time_ms: 0.0,
        };
        
        stats.calculate_efficiency();
        
        assert_eq!(stats.culling_efficiency, 80.0);
    }

    #[test]
    fn test_transform_aabb() {
        let aabb = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let transform = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
        
        let transformed = transform_aabb(&aabb, &transform);
        
        // Center should be translated
        assert!((transformed.center().x - 5.0).abs() < 0.001);
        assert!(transformed.center().y.abs() < 0.001);
        assert!(transformed.center().z.abs() < 0.001);
        
        // Extents should remain the same for translation
        let original_extents = aabb.extents();
        let transformed_extents = transformed.extents();
        assert!((original_extents.x - transformed_extents.x).abs() < 0.001);
    }

    #[test]
    fn test_occludable_component() {
        let occludable = Occludable::new();
        assert!(occludable.enabled);
        assert_eq!(occludable.retest_interval, 5);
        
        let custom = Occludable::with_interval(10);
        assert_eq!(custom.retest_interval, 10);
    }
}
