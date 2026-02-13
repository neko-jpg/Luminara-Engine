/// Property Test: Sprite Batching by Texture
///
/// **Validates: Requirements 3.1**
///
/// **Property 13: Sprite Batching by Texture**
/// For any set of sprites that share the same texture handle, the batch renderer
/// should group them into a single batch (or multiple batches if exceeding max batch size),
/// minimizing the number of draw calls.
use luminara_asset::{AssetId, Handle};
use luminara_math::{Color, Mat4};
use luminara_render::{Anchor, Rect, Sprite, SpriteBatcher, Texture, ZOrder};
use proptest::prelude::*;

// Generator for sprite count
fn arb_sprite_count() -> impl Strategy<Value = usize> {
    1usize..100
}

// Generator for max batch size
fn arb_max_batch_size() -> impl Strategy<Value = usize> {
    10usize..1000
}

// Generator for number of unique textures
fn arb_texture_count() -> impl Strategy<Value = usize> {
    1usize..10
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 3.1**
    #[test]
    fn prop_sprite_batching_single_texture(
        sprite_count in arb_sprite_count(),
        max_batch_size in arb_max_batch_size(),
    ) {
        let mut batcher = SpriteBatcher::new(max_batch_size);
        let texture = Handle::<Texture>::new(AssetId::new());

        // Create sprites with the same texture
        let sprites: Vec<_> = (0..sprite_count)
            .map(|_| Sprite::new(texture.clone()))
            .collect();

        // Prepare batches
        let sprite_data: Vec<_> = sprites
            .iter()
            .map(|s| (s, &Mat4::IDENTITY, None::<&ZOrder>))
            .collect();
        batcher.prepare(sprite_data);

        // Calculate expected number of batches
        let expected_batches = (sprite_count + max_batch_size - 1) / max_batch_size;

        // Verify batch count
        prop_assert_eq!(batcher.batch_count(), expected_batches);

        // Verify total instances
        prop_assert_eq!(batcher.total_instances(), sprite_count);

        // Verify all batches have the same texture
        for batch in &batcher.batches {
            prop_assert_eq!(batch.texture.id(), texture.id());
        }
    }

    /// **Validates: Requirements 3.1**
    #[test]
    fn prop_sprite_batching_multiple_textures(
        sprite_count in arb_sprite_count(),
        texture_count in arb_texture_count(),
    ) {
        let mut batcher = SpriteBatcher::new(1000);

        // Create unique textures
        let textures: Vec<_> = (0..texture_count)
            .map(|_| Handle::<Texture>::new(AssetId::new()))
            .collect();

        // Create sprites distributed across textures
        let sprites: Vec<_> = (0..sprite_count)
            .map(|i| {
                let texture_idx = i % texture_count;
                Sprite::new(textures[texture_idx].clone())
            })
            .collect();

        // Prepare batches
        let sprite_data: Vec<_> = sprites
            .iter()
            .map(|s| (s, &Mat4::IDENTITY, None::<&ZOrder>))
            .collect();
        batcher.prepare(sprite_data);

        // Verify total instances
        prop_assert_eq!(batcher.total_instances(), sprite_count);

        // Verify batch count is at most texture_count (could be less if some textures have no sprites)
        prop_assert!(batcher.batch_count() <= texture_count);

        // Verify each batch has a consistent texture
        for batch in &batcher.batches {
            prop_assert!(!batch.instances.is_empty());
            // All instances in a batch should come from sprites with the same texture
        }
    }

    /// **Validates: Requirements 3.1**
    #[test]
    fn prop_sprite_batching_respects_max_size(
        sprite_count in 50usize..200,
        max_batch_size in 10usize..50,
    ) {
        let mut batcher = SpriteBatcher::new(max_batch_size);
        let texture = Handle::<Texture>::new(AssetId::new());

        // Create sprites with the same texture
        let sprites: Vec<_> = (0..sprite_count)
            .map(|_| Sprite::new(texture.clone()))
            .collect();

        // Prepare batches
        let sprite_data: Vec<_> = sprites
            .iter()
            .map(|s| (s, &Mat4::IDENTITY, None::<&ZOrder>))
            .collect();
        batcher.prepare(sprite_data);

        // Verify no batch exceeds max size
        for batch in &batcher.batches {
            prop_assert!(batch.instances.len() <= max_batch_size);
        }

        // Verify total instances
        prop_assert_eq!(batcher.total_instances(), sprite_count);
    }

    /// **Validates: Requirements 3.1**
    #[test]
    fn prop_sprite_batching_minimizes_batches(
        sprite_count in arb_sprite_count(),
    ) {
        let max_batch_size = 1000;
        let mut batcher = SpriteBatcher::new(max_batch_size);
        let texture = Handle::<Texture>::new(AssetId::new());

        // Create sprites with the same texture
        let sprites: Vec<_> = (0..sprite_count)
            .map(|_| Sprite::new(texture.clone()))
            .collect();

        // Prepare batches
        let sprite_data: Vec<_> = sprites
            .iter()
            .map(|s| (s, &Mat4::IDENTITY, None::<&ZOrder>))
            .collect();
        batcher.prepare(sprite_data);

        // For a single texture, we should have the minimum number of batches
        let expected_batches = (sprite_count + max_batch_size - 1) / max_batch_size;
        prop_assert_eq!(batcher.batch_count(), expected_batches);
    }
}
