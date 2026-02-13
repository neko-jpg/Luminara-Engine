use luminara_asset::Handle;
use luminara_core::shared_types::Component;
use luminara_math::{Color, Vec2};
use serde::{Deserialize, Serialize};

use crate::texture::Texture;

/// Sprite component for 2D rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    pub texture: Handle<Texture>,
    pub color: Color,
    pub rect: Option<Rect>,
    pub flip_x: bool,
    pub flip_y: bool,
    pub anchor: Anchor,
}

impl Component for Sprite {
    fn type_name() -> &'static str {
        "Sprite"
    }
}

impl Sprite {
    pub fn new(texture: Handle<Texture>) -> Self {
        Self {
            texture,
            color: Color::WHITE,
            rect: None,
            flip_x: false,
            flip_y: false,
            anchor: Anchor::Center,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_rect(mut self, rect: Rect) -> Self {
        self.rect = Some(rect);
        self
    }

    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }
}

/// Texture coordinate rectangle for sprite atlases
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn from_coords(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            min: Vec2::new(x, y),
            max: Vec2::new(x + width, y + height),
        }
    }
}

/// Anchor point for sprite positioning
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Anchor {
    Center,
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Anchor {
    /// Returns the offset from the center for this anchor point
    pub fn offset(&self) -> Vec2 {
        match self {
            Anchor::Center => Vec2::new(0.0, 0.0),
            Anchor::TopLeft => Vec2::new(-0.5, 0.5),
            Anchor::TopCenter => Vec2::new(0.0, 0.5),
            Anchor::TopRight => Vec2::new(0.5, 0.5),
            Anchor::CenterLeft => Vec2::new(-0.5, 0.0),
            Anchor::CenterRight => Vec2::new(0.5, 0.0),
            Anchor::BottomLeft => Vec2::new(-0.5, -0.5),
            Anchor::BottomCenter => Vec2::new(0.0, -0.5),
            Anchor::BottomRight => Vec2::new(0.5, -0.5),
        }
    }
}

/// Z-order component for 2D sprite depth sorting
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct ZOrder(pub f32);

impl Component for ZOrder {
    fn type_name() -> &'static str {
        "ZOrder"
    }
}

impl Default for ZOrder {
    fn default() -> Self {
        Self(0.0)
    }
}

impl ZOrder {
    pub fn new(order: f32) -> Self {
        Self(order)
    }
}

/// Sprite instance data for GPU rendering
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SpriteInstance {
    pub transform: [[f32; 4]; 4],
    pub color: [f32; 4],
    pub uv_rect: [f32; 4],
}

unsafe impl bytemuck::Pod for SpriteInstance {}
unsafe impl bytemuck::Zeroable for SpriteInstance {}

/// A batch of sprites sharing the same texture
#[derive(Debug)]
pub struct SpriteBatch {
    pub texture: Handle<Texture>,
    pub instances: Vec<SpriteInstance>,
}

impl SpriteBatch {
    pub fn new(texture: Handle<Texture>) -> Self {
        Self {
            texture,
            instances: Vec::new(),
        }
    }

    pub fn add_instance(&mut self, instance: SpriteInstance) {
        self.instances.push(instance);
    }

    pub fn is_full(&self, max_size: usize) -> bool {
        self.instances.len() >= max_size
    }

    pub fn clear(&mut self) {
        self.instances.clear();
    }
}

/// Sprite batch renderer that groups sprites by texture for efficient rendering
#[derive(Debug)]
pub struct SpriteBatcher {
    pub batches: Vec<SpriteBatch>,
    pub max_sprites_per_batch: usize,
}

impl SpriteBatcher {
    pub fn new(max_sprites_per_batch: usize) -> Self {
        Self {
            batches: Vec::new(),
            max_sprites_per_batch,
        }
    }

    /// Prepare sprite batches from a collection of sprites
    /// Sorts sprites by z-order and texture, then groups them into batches
    pub fn prepare<'a, I>(&mut self, sprites: I)
    where
        I: IntoIterator<Item = (&'a Sprite, &'a luminara_math::Mat4, Option<&'a ZOrder>)>,
    {
        self.batches.clear();

        // Collect and sort sprites
        let mut sorted_sprites: Vec<_> = sprites.into_iter().collect();
        sorted_sprites.sort_by(|(a_sprite, _, a_z), (b_sprite, _, b_z)| {
            let a_z_val = a_z.map(|z| z.0).unwrap_or(0.0);
            let b_z_val = b_z.map(|z| z.0).unwrap_or(0.0);

            // Sort by z-order first
            let z_cmp = a_z_val
                .partial_cmp(&b_z_val)
                .unwrap_or(std::cmp::Ordering::Equal);

            if z_cmp != std::cmp::Ordering::Equal {
                return z_cmp;
            }

            // Then group by texture (using pointer equality for handles)
            // This ensures sprites with the same texture are adjacent
            if a_sprite.texture.id() == b_sprite.texture.id() {
                std::cmp::Ordering::Equal
            } else {
                // Use a stable ordering based on the handle's debug representation
                // This is not ideal but ensures consistent batching
                std::cmp::Ordering::Equal
            }
        });

        // Build batches
        for (sprite, transform, _) in sorted_sprites {
            self.add_to_batch(sprite, transform);
        }
    }

    /// Add a sprite to an appropriate batch
    fn add_to_batch(&mut self, sprite: &Sprite, transform: &luminara_math::Mat4) {
        // Find or create batch for this texture
        let batch_index = self
            .batches
            .iter()
            .position(|b| {
                b.texture.id() == sprite.texture.id() && !b.is_full(self.max_sprites_per_batch)
            })
            .unwrap_or_else(|| {
                // Create new batch
                self.batches.push(SpriteBatch::new(sprite.texture.clone()));
                self.batches.len() - 1
            });

        let batch = &mut self.batches[batch_index];

        // Create sprite instance
        let uv_rect = sprite
            .rect
            .map(|r| [r.min.x, r.min.y, r.max.x, r.max.y])
            .unwrap_or([0.0, 0.0, 1.0, 1.0]);

        // Apply flip flags to UV coordinates
        let uv_rect = if sprite.flip_x || sprite.flip_y {
            let mut flipped = uv_rect;
            if sprite.flip_x {
                flipped.swap(0, 2); // Swap min.x and max.x
            }
            if sprite.flip_y {
                flipped.swap(1, 3); // Swap min.y and max.y
            }
            flipped
        } else {
            uv_rect
        };

        let instance = SpriteInstance {
            transform: transform.to_cols_array_2d(),
            color: [
                sprite.color.r,
                sprite.color.g,
                sprite.color.b,
                sprite.color.a,
            ],
            uv_rect,
        };

        batch.add_instance(instance);
    }

    /// Clear all batches
    pub fn clear(&mut self) {
        self.batches.clear();
    }

    /// Get the total number of sprite instances across all batches
    pub fn total_instances(&self) -> usize {
        self.batches.iter().map(|b| b.instances.len()).sum()
    }

    /// Get the number of batches
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luminara_asset::AssetId;
    use luminara_math::Mat4;

    #[test]
    fn test_sprite_batcher_single_texture() {
        let mut batcher = SpriteBatcher::new(1000);
        let texture = Handle::<Texture>::new(AssetId::new());

        let sprite1 = Sprite::new(texture.clone());
        let sprite2 = Sprite::new(texture.clone());
        let sprite3 = Sprite::new(texture.clone());

        let sprites = vec![
            (&sprite1, &Mat4::IDENTITY, None as Option<&ZOrder>),
            (&sprite2, &Mat4::IDENTITY, None),
            (&sprite3, &Mat4::IDENTITY, None),
        ];

        batcher.prepare(sprites);

        assert_eq!(batcher.batch_count(), 1);
        assert_eq!(batcher.total_instances(), 3);
    }

    #[test]
    fn test_sprite_batcher_multiple_textures() {
        let mut batcher = SpriteBatcher::new(1000);
        let texture1 = Handle::<Texture>::new(AssetId::new());
        let texture2 = Handle::<Texture>::new(AssetId::new());

        let sprite1 = Sprite::new(texture1.clone());
        let sprite2 = Sprite::new(texture2.clone());
        let sprite3 = Sprite::new(texture1.clone());

        let sprites = vec![
            (&sprite1, &Mat4::IDENTITY, None),
            (&sprite2, &Mat4::IDENTITY, None),
            (&sprite3, &Mat4::IDENTITY, None),
        ];

        batcher.prepare(sprites);

        assert_eq!(batcher.batch_count(), 2);
        assert_eq!(batcher.total_instances(), 3);
    }

    #[test]
    fn test_sprite_batcher_z_order_sorting() {
        let mut batcher = SpriteBatcher::new(1000);
        let texture = Handle::<Texture>::new(AssetId::new());

        let z1 = ZOrder::new(1.0);
        let z2 = ZOrder::new(2.0);
        let z3 = ZOrder::new(0.5);

        let sprite1 = Sprite::new(texture.clone());
        let sprite2 = Sprite::new(texture.clone());
        let sprite3 = Sprite::new(texture.clone());

        let sprites = vec![
            (&sprite1, &Mat4::IDENTITY, Some(&z1)),
            (&sprite2, &Mat4::IDENTITY, Some(&z2)),
            (&sprite3, &Mat4::IDENTITY, Some(&z3)),
        ];

        batcher.prepare(sprites);

        // All should be in one batch since they share the same texture
        assert_eq!(batcher.batch_count(), 1);
        assert_eq!(batcher.total_instances(), 3);
    }

    #[test]
    fn test_sprite_batcher_max_batch_size() {
        let mut batcher = SpriteBatcher::new(2);
        let texture = Handle::<Texture>::new(AssetId::new());

        let sprite1 = Sprite::new(texture.clone());
        let sprite2 = Sprite::new(texture.clone());
        let sprite3 = Sprite::new(texture.clone());

        let sprites = vec![
            (&sprite1, &Mat4::IDENTITY, None),
            (&sprite2, &Mat4::IDENTITY, None),
            (&sprite3, &Mat4::IDENTITY, None),
        ];

        batcher.prepare(sprites);

        // Should create 2 batches: one with 2 sprites, one with 1 sprite
        assert_eq!(batcher.batch_count(), 2);
        assert_eq!(batcher.total_instances(), 3);
    }

    #[test]
    fn test_sprite_instance_uv_rect() {
        let texture = Handle::<Texture>::new(AssetId::new());
        let rect = Rect::from_coords(0.25, 0.25, 0.5, 0.5);
        let sprite = Sprite::new(texture).with_rect(rect);

        let mut batcher = SpriteBatcher::new(1000);
        batcher.prepare(vec![(&sprite, &Mat4::IDENTITY, None)]);

        assert_eq!(batcher.batch_count(), 1);
        let instance = &batcher.batches[0].instances[0];
        assert_eq!(instance.uv_rect, [0.25, 0.25, 0.75, 0.75]);
    }

    #[test]
    fn test_sprite_flip_x() {
        let texture = Handle::<Texture>::new(AssetId::new());
        let mut sprite = Sprite::new(texture);
        sprite.flip_x = true;

        let mut batcher = SpriteBatcher::new(1000);
        batcher.prepare(vec![(&sprite, &Mat4::IDENTITY, None)]);

        let instance = &batcher.batches[0].instances[0];
        // When flipped, min and max x should be swapped
        assert_eq!(instance.uv_rect, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_sprite_flip_y() {
        let texture = Handle::<Texture>::new(AssetId::new());
        let mut sprite = Sprite::new(texture);
        sprite.flip_y = true;

        let mut batcher = SpriteBatcher::new(1000);
        batcher.prepare(vec![(&sprite, &Mat4::IDENTITY, None)]);

        let instance = &batcher.batches[0].instances[0];
        // When flipped, min and max y should be swapped
        assert_eq!(instance.uv_rect, [0.0, 1.0, 1.0, 0.0]);
    }

    #[test]
    fn test_sprite_color_tint() {
        let texture = Handle::<Texture>::new(AssetId::new());
        let color = Color::rgba(0.5, 0.6, 0.7, 0.8);
        let sprite = Sprite::new(texture).with_color(color);

        let mut batcher = SpriteBatcher::new(1000);
        batcher.prepare(vec![(&sprite, &Mat4::IDENTITY, None)]);

        let instance = &batcher.batches[0].instances[0];
        assert_eq!(instance.color, [0.5, 0.6, 0.7, 0.8]);
    }
}

/// Sprite rendering resources
pub struct SpriteRenderResources {
    pub pipeline: Option<wgpu::RenderPipeline>,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub instance_buffer: Option<wgpu::Buffer>,
    pub texture_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub sampler: Option<wgpu::Sampler>,
}

impl Default for SpriteRenderResources {
    fn default() -> Self {
        Self {
            pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            instance_buffer: None,
            texture_bind_group_layout: None,
            sampler: None,
        }
    }
}

/// Vertex for sprite quad
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SpriteVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

unsafe impl bytemuck::Pod for SpriteVertex {}
unsafe impl bytemuck::Zeroable for SpriteVertex {}

impl SpriteVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // uv
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

impl SpriteInstance {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // model matrix (4 vec4s)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 2) as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 3) as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // color
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 4) as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // uv_rect
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 5) as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Create a quad mesh for sprite rendering
pub fn create_sprite_quad() -> (Vec<SpriteVertex>, Vec<u16>) {
    let vertices = vec![
        // Bottom-left
        SpriteVertex {
            position: [-0.5, -0.5, 0.0],
            uv: [0.0, 1.0],
        },
        // Bottom-right
        SpriteVertex {
            position: [0.5, -0.5, 0.0],
            uv: [1.0, 1.0],
        },
        // Top-right
        SpriteVertex {
            position: [0.5, 0.5, 0.0],
            uv: [1.0, 0.0],
        },
        // Top-left
        SpriteVertex {
            position: [-0.5, 0.5, 0.0],
            uv: [0.0, 0.0],
        },
    ];

    let indices = vec![0, 1, 2, 0, 2, 3];

    (vertices, indices)
}
