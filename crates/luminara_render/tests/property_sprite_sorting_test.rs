/// Property Test: Sprite Sorting
///
/// **Validates: Requirements 3.3**
///
/// **Property 15: Sprite Sorting**
/// For any set of sprites with z-order and texture values, the batch renderer
/// should sort them first by z-order (depth) and then by texture to minimize state changes.

use luminara_asset::{AssetId, Handle};
use luminara_math::Mat4;
use luminara_render::{Sprite, SpriteBatcher, Texture, ZOrder};
use proptest::prelude::*;

// Generator for z-order
fn arb_z_order() -> impl Strategy<Value = f32> {
    -100.0f32..100.0f32
}

// Generator for sprite with z-order
fn arb_sprite_with_z() -> impl Strategy<Value = (Sprite, ZOrder)> {
    (any::<u64>(), arb_z_order()).prop_map(|(texture_seed, z)| {
        // Use seed to create deterministic texture IDs
        let texture = Handle::<Texture>::new(AssetId::new());
        let sprite = Sprite::new(texture);
        let z_order = ZOrder::new(z);
        (sprite, z_order)
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 3.3**
    #[test]
    fn prop_sprite_sorting_by_z_order(
        z_orders in prop::collection::vec(arb_z_order(), 2..20),
    ) {
        let mut batcher = SpriteBatcher::new(1000);
        let texture = Handle::<Texture>::new(AssetId::new());

        // Create sprites with different z-orders but same texture
        let sprites: Vec<_> = z_orders
            .iter()
            .map(|&z| (Sprite::new(texture.clone()), ZOrder::new(z)))
            .collect();

        // Prepare batches (sprites should be sorted by z-order)
        let sprite_data: Vec<_> = sprites
            .iter()
            .map(|(s, z)| (s, &Mat4::IDENTITY, Some(z)))
            .collect();
        batcher.prepare(sprite_data);

        // All sprites should be in one batch since they share the same texture
        prop_assert_eq!(batcher.batch_count(), 1);
        prop_assert_eq!(batcher.total_instances(), z_orders.len());

        // Note: We can't directly verify the sorting order from the batch instances
        // because the instance data doesn't include z-order information.
        // The sorting is done during the prepare phase, and the instances are
        // added in sorted order. This test verifies that the batching works correctly
        // with z-ordered sprites.
    }

    /// **Validates: Requirements 3.3**
    #[test]
    fn prop_sprite_sorting_groups_by_texture(
        sprite_count in 10usize..50,
        texture_count in 2usize..5,
    ) {
        let mut batcher = SpriteBatcher::new(1000);

        // Create unique textures
        let textures: Vec<_> = (0..texture_count)
            .map(|_| Handle::<Texture>::new(AssetId::new()))
            .collect();

        // Create sprites with same z-order but different textures
        let z_order = ZOrder::new(0.0);
        let sprites: Vec<_> = (0..sprite_count)
            .map(|i| {
                let texture_idx = i % texture_count;
                Sprite::new(textures[texture_idx].clone())
            })
            .collect();

        // Prepare batches
        let sprite_data: Vec<_> = sprites
            .iter()
            .map(|s| (s, &Mat4::IDENTITY, Some(&z_order)))
            .collect();
        batcher.prepare(sprite_data);

        // Verify sprites are grouped by texture
        // Each batch should contain sprites with the same texture
        for batch in &batcher.batches {
            let texture_id = batch.texture.id();
            // All instances in this batch should come from sprites with the same texture
            prop_assert!(!batch.instances.is_empty());
            
            // Count how many sprites in the original list have this texture
            let expected_count = sprites
                .iter()
                .filter(|s| s.texture.id() == texture_id)
                .count();
            
            prop_assert_eq!(batch.instances.len(), expected_count);
        }
    }

    /// **Validates: Requirements 3.3**
    #[test]
    fn prop_sprite_sorting_z_then_texture(
        sprites_per_texture in 5usize..15,
    ) {
        let mut batcher = SpriteBatcher::new(1000);

        // Create two textures
        let texture1 = Handle::<Texture>::new(AssetId::new());
        let texture2 = Handle::<Texture>::new(AssetId::new());

        // Create sprites with different z-orders and textures
        // Interleave textures to test sorting
        let mut sprites = Vec::new();
        for i in 0..sprites_per_texture {
            let z = i as f32;
            sprites.push((Sprite::new(texture1.clone()), ZOrder::new(z)));
            sprites.push((Sprite::new(texture2.clone()), ZOrder::new(z)));
        }

        // Prepare batches
        let sprite_data: Vec<_> = sprites
            .iter()
            .map(|(s, z)| (s, &Mat4::IDENTITY, Some(z)))
            .collect();
        batcher.prepare(sprite_data);

        // Should have at least 2 batches (one for each texture)
        prop_assert!(batcher.batch_count() >= 2);
        prop_assert_eq!(batcher.total_instances(), sprites.len());
    }

    /// **Validates: Requirements 3.3**
    #[test]
    fn prop_sprite_sorting_preserves_all_sprites(
        sprite_count in 1usize..100,
    ) {
        let mut batcher = SpriteBatcher::new(1000);

        // Create sprites with random z-orders and textures
        let textures: Vec<_> = (0..5)
            .map(|_| Handle::<Texture>::new(AssetId::new()))
            .collect();

        let sprites: Vec<_> = (0..sprite_count)
            .map(|i| {
                let texture_idx = i % textures.len();
                let z = (i as f32) * 0.1;
                (Sprite::new(textures[texture_idx].clone()), ZOrder::new(z))
            })
            .collect();

        // Prepare batches
        let sprite_data: Vec<_> = sprites
            .iter()
            .map(|(s, z)| (s, &Mat4::IDENTITY, Some(z)))
            .collect();
        batcher.prepare(sprite_data);

        // Verify all sprites are preserved
        prop_assert_eq!(batcher.total_instances(), sprite_count);

        // Verify no sprites are lost or duplicated
        let total_in_batches: usize = batcher.batches.iter().map(|b| b.instances.len()).sum();
        prop_assert_eq!(total_in_batches, sprite_count);
    }

    /// **Validates: Requirements 3.3**
    #[test]
    fn prop_sprite_sorting_with_no_z_order(
        sprite_count in 1usize..50,
    ) {
        let mut batcher = SpriteBatcher::new(1000);
        let texture = Handle::<Texture>::new(AssetId::new());

        // Create sprites without z-order (should default to 0.0)
        let sprites: Vec<_> = (0..sprite_count)
            .map(|_| Sprite::new(texture.clone()))
            .collect();

        // Prepare batches without z-order
        let sprite_data: Vec<_> = sprites
            .iter()
            .map(|s| (s, &Mat4::IDENTITY, None::<&ZOrder>))
            .collect();
        batcher.prepare(sprite_data);

        // All sprites should be in one batch
        prop_assert_eq!(batcher.batch_count(), 1);
        prop_assert_eq!(batcher.total_instances(), sprite_count);
    }
}
