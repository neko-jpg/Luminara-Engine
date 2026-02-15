use luminara_asset::{AssetId, Handle};
use luminara_math::{Mat4, Vec3};
use luminara_render::{Sprite, SpriteBatcher, Texture, ZOrder};

#[test]
fn test_instancing_batch_logic() {
    let mut batcher = SpriteBatcher::new(100);
    let texture = Handle::<Texture>::new(AssetId::new(), 0);
    let sprite = Sprite::new(texture.clone());

    // Add 10 sprites
    let transforms = vec![Mat4::IDENTITY; 10];
    let items: Vec<_> = transforms
        .iter()
        .map(|t| (&sprite, t, None::<&ZOrder>))
        .collect();

    batcher.prepare(items);

    // Should result in 1 batch
    assert_eq!(batcher.batches.len(), 1);
    assert_eq!(batcher.batches[0].instances.len(), 10);
}
