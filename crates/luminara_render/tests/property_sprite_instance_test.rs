/// Property Test: Sprite Instance Data Correctness
///
/// **Validates: Requirements 3.2, 3.4**
///
/// **Property 14: Sprite Instance Data Correctness**
/// For any sprite with a texture, color, transform, and optional texture rect,
/// the batch renderer should create a sprite instance with the correct transform matrix,
/// color values, and UV coordinates.
use luminara_asset::{AssetId, Handle};
use luminara_math::{Color, Mat4, Quat, Vec2, Vec3};
use luminara_render::{Anchor, Rect, Sprite, SpriteBatcher, Texture, ZOrder};
use proptest::prelude::*;

// Generator for color
fn arb_color() -> impl Strategy<Value = Color> {
    (0.0f32..=1.0, 0.0f32..=1.0, 0.0f32..=1.0, 0.0f32..=1.0)
        .prop_map(|(r, g, b, a)| Color::rgba(r, g, b, a))
}

// Generator for transform
fn arb_transform() -> impl Strategy<Value = Mat4> {
    (
        prop::array::uniform3(-100.0f32..100.0f32),
        prop::array::uniform4(-1.0f32..1.0f32),
        prop::array::uniform3(0.1f32..10.0f32),
    )
        .prop_map(|(pos, rot, scale)| {
            let rotation = Quat::from_xyzw(rot[0], rot[1], rot[2], rot[3]).normalize();
            Mat4::from_scale_rotation_translation(
                Vec3::from_array(scale),
                rotation,
                Vec3::from_array(pos),
            )
        })
}

// Generator for optional UV rect
fn arb_uv_rect() -> impl Strategy<Value = Option<Rect>> {
    prop::option::of(
        (0.0f32..1.0, 0.0f32..1.0, 0.0f32..1.0, 0.0f32..1.0).prop_map(|(x, y, w, h)| {
            let min = Vec2::new(x, y);
            let max = Vec2::new((x + w).min(1.0), (y + h).min(1.0));
            Rect::new(min, max)
        }),
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 3.2**
    #[test]
    fn prop_sprite_instance_color(color in arb_color()) {
        let mut batcher = SpriteBatcher::new(1000);
        let texture = Handle::<Texture>::new(AssetId::new());
        let sprite = Sprite::new(texture).with_color(color);

        batcher.prepare(vec![(&sprite, &Mat4::IDENTITY, None::<&ZOrder>)]);

        prop_assert_eq!(batcher.batch_count(), 1);
        let instance = &batcher.batches[0].instances[0];

        // Verify color is correctly stored
        prop_assert!((instance.color[0] - color.r).abs() < 0.001);
        prop_assert!((instance.color[1] - color.g).abs() < 0.001);
        prop_assert!((instance.color[2] - color.b).abs() < 0.001);
        prop_assert!((instance.color[3] - color.a).abs() < 0.001);
    }

    /// **Validates: Requirements 3.2**
    #[test]
    fn prop_sprite_instance_transform(transform in arb_transform()) {
        let mut batcher = SpriteBatcher::new(1000);
        let texture = Handle::<Texture>::new(AssetId::new());
        let sprite = Sprite::new(texture);

        batcher.prepare(vec![(&sprite, &transform, None::<&ZOrder>)]);

        prop_assert_eq!(batcher.batch_count(), 1);
        let instance = &batcher.batches[0].instances[0];

        // Verify transform matrix is correctly stored
        let expected = transform.to_cols_array_2d();
        for i in 0..4 {
            for j in 0..4 {
                let diff = (instance.transform[i][j] - expected[i][j]).abs();
                prop_assert!(diff < 0.001, "Transform mismatch at [{}, {}]", i, j);
            }
        }
    }

    /// **Validates: Requirements 3.4**
    #[test]
    fn prop_sprite_instance_uv_rect(uv_rect in arb_uv_rect()) {
        let mut batcher = SpriteBatcher::new(1000);
        let texture = Handle::<Texture>::new(AssetId::new());
        let mut sprite = Sprite::new(texture);
        sprite.rect = uv_rect;

        batcher.prepare(vec![(&sprite, &Mat4::IDENTITY, None::<&ZOrder>)]);

        prop_assert_eq!(batcher.batch_count(), 1);
        let instance = &batcher.batches[0].instances[0];

        // Verify UV rect
        if let Some(rect) = uv_rect {
            prop_assert!((instance.uv_rect[0] - rect.min.x).abs() < 0.001);
            prop_assert!((instance.uv_rect[1] - rect.min.y).abs() < 0.001);
            prop_assert!((instance.uv_rect[2] - rect.max.x).abs() < 0.001);
            prop_assert!((instance.uv_rect[3] - rect.max.y).abs() < 0.001);
        } else {
            // Default UV rect should be [0, 0, 1, 1]
            prop_assert_eq!(instance.uv_rect, [0.0, 0.0, 1.0, 1.0]);
        }
    }

    /// **Validates: Requirements 3.4**
    #[test]
    fn prop_sprite_instance_flip_x(flip_x in prop::bool::ANY) {
        let mut batcher = SpriteBatcher::new(1000);
        let texture = Handle::<Texture>::new(AssetId::new());
        let mut sprite = Sprite::new(texture);
        sprite.flip_x = flip_x;

        batcher.prepare(vec![(&sprite, &Mat4::IDENTITY, None::<&ZOrder>)]);

        prop_assert_eq!(batcher.batch_count(), 1);
        let instance = &batcher.batches[0].instances[0];

        if flip_x {
            // When flipped, min.x and max.x should be swapped
            prop_assert_eq!(instance.uv_rect[0], 1.0);
            prop_assert_eq!(instance.uv_rect[2], 0.0);
        } else {
            prop_assert_eq!(instance.uv_rect[0], 0.0);
            prop_assert_eq!(instance.uv_rect[2], 1.0);
        }
    }

    /// **Validates: Requirements 3.4**
    #[test]
    fn prop_sprite_instance_flip_y(flip_y in prop::bool::ANY) {
        let mut batcher = SpriteBatcher::new(1000);
        let texture = Handle::<Texture>::new(AssetId::new());
        let mut sprite = Sprite::new(texture);
        sprite.flip_y = flip_y;

        batcher.prepare(vec![(&sprite, &Mat4::IDENTITY, None::<&ZOrder>)]);

        prop_assert_eq!(batcher.batch_count(), 1);
        let instance = &batcher.batches[0].instances[0];

        if flip_y {
            // When flipped, min.y and max.y should be swapped
            prop_assert_eq!(instance.uv_rect[1], 1.0);
            prop_assert_eq!(instance.uv_rect[3], 0.0);
        } else {
            prop_assert_eq!(instance.uv_rect[1], 0.0);
            prop_assert_eq!(instance.uv_rect[3], 1.0);
        }
    }

    /// **Validates: Requirements 3.2, 3.4**
    #[test]
    fn prop_sprite_instance_complete(
        color in arb_color(),
        transform in arb_transform(),
        uv_rect in arb_uv_rect(),
    ) {
        let mut batcher = SpriteBatcher::new(1000);
        let texture = Handle::<Texture>::new(AssetId::new());
        let mut sprite = Sprite::new(texture).with_color(color);
        sprite.rect = uv_rect;

        batcher.prepare(vec![(&sprite, &transform, None::<&ZOrder>)]);

        prop_assert_eq!(batcher.batch_count(), 1);
        let instance = &batcher.batches[0].instances[0];

        // Verify all properties are correctly stored
        // Color
        prop_assert!((instance.color[0] - color.r).abs() < 0.001);
        prop_assert!((instance.color[1] - color.g).abs() < 0.001);
        prop_assert!((instance.color[2] - color.b).abs() < 0.001);
        prop_assert!((instance.color[3] - color.a).abs() < 0.001);

        // Transform
        let expected = transform.to_cols_array_2d();
        for i in 0..4 {
            for j in 0..4 {
                let diff = (instance.transform[i][j] - expected[i][j]).abs();
                prop_assert!(diff < 0.001, "Transform mismatch at [{}, {}]", i, j);
            }
        }

        // UV rect
        if let Some(rect) = uv_rect {
            prop_assert!((instance.uv_rect[0] - rect.min.x).abs() < 0.001);
            prop_assert!((instance.uv_rect[1] - rect.min.y).abs() < 0.001);
            prop_assert!((instance.uv_rect[2] - rect.max.x).abs() < 0.001);
            prop_assert!((instance.uv_rect[3] - rect.max.y).abs() < 0.001);
        } else {
            prop_assert_eq!(instance.uv_rect, [0.0, 0.0, 1.0, 1.0]);
        }
    }
}
