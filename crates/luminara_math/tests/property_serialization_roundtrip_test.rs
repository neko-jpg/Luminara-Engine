use luminara_math::{Color, Quat, Transform, Vec3};
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// Property 9: Serialization Round-Trip Preservation
// Feature: pre-editor-engine-audit
// Validates: Requirements 8.1
// ============================================================================

/// **Validates: Requirements 8.1**
///
/// For any component value, serializing to RON or binary format and then
/// deserializing should produce an equivalent value.
///
/// This property test ensures that all core types (Vec3, Quat, Transform, Color)
/// can be serialized and deserialized without data loss, which is critical for
/// editor functionality where scenes and assets must be saved and loaded reliably.

// ============================================================================
// Strategies for generating random values
// ============================================================================

/// Strategy for generating random Vec3 values
fn vec3_strategy() -> impl Strategy<Value = Vec3> {
    (
        prop::num::f32::NORMAL,
        prop::num::f32::NORMAL,
        prop::num::f32::NORMAL,
    )
        .prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

/// Strategy for generating random Quat values (normalized)
fn quat_strategy() -> impl Strategy<Value = Quat> {
    (
        prop::num::f32::NORMAL,
        prop::num::f32::NORMAL,
        prop::num::f32::NORMAL,
        prop::num::f32::NORMAL,
    )
        .prop_map(|(x, y, z, w)| {
            let q = Quat::from_xyzw(x, y, z, w);
            // Normalize to ensure valid quaternion
            if q.length_squared() > 1e-6 {
                q.normalize()
            } else {
                Quat::IDENTITY
            }
        })
}

/// Strategy for generating random Transform values
fn transform_strategy() -> impl Strategy<Value = Transform> {
    (vec3_strategy(), quat_strategy(), vec3_strategy()).prop_map(
        |(translation, rotation, scale)| Transform {
            translation,
            rotation,
            scale,
        },
    )
}

/// Strategy for generating random Color values
fn color_strategy() -> impl Strategy<Value = Color> {
    (
        0.0f32..=1.0f32,
        0.0f32..=1.0f32,
        0.0f32..=1.0f32,
        0.0f32..=1.0f32,
    )
        .prop_map(|(r, g, b, a)| Color::rgba(r, g, b, a))
}

// ============================================================================
// Helper functions for approximate equality
// ============================================================================

/// Check if two Vec3 values are approximately equal
fn vec3_approx_eq(a: &Vec3, b: &Vec3, epsilon: f32) -> bool {
    (a.x - b.x).abs() < epsilon
        && (a.y - b.y).abs() < epsilon
        && (a.z - b.z).abs() < epsilon
}

/// Check if two Quat values are approximately equal
fn quat_approx_eq(a: &Quat, b: &Quat, epsilon: f32) -> bool {
    // Quaternions q and -q represent the same rotation
    let same_sign = (a.x - b.x).abs() < epsilon
        && (a.y - b.y).abs() < epsilon
        && (a.z - b.z).abs() < epsilon
        && (a.w - b.w).abs() < epsilon;
    
    let opposite_sign = (a.x + b.x).abs() < epsilon
        && (a.y + b.y).abs() < epsilon
        && (a.z + b.z).abs() < epsilon
        && (a.w + b.w).abs() < epsilon;
    
    same_sign || opposite_sign
}

/// Check if two Transform values are approximately equal
fn transform_approx_eq(a: &Transform, b: &Transform, epsilon: f32) -> bool {
    vec3_approx_eq(&a.translation, &b.translation, epsilon)
        && quat_approx_eq(&a.rotation, &b.rotation, epsilon)
        && vec3_approx_eq(&a.scale, &b.scale, epsilon)
}

/// Check if two Color values are approximately equal
fn color_approx_eq(a: &Color, b: &Color, epsilon: f32) -> bool {
    (a.r - b.r).abs() < epsilon
        && (a.g - b.g).abs() < epsilon
        && (a.b - b.b).abs() < epsilon
        && (a.a - b.a).abs() < epsilon
}

// ============================================================================
// Generic round-trip test functions
// ============================================================================

/// Test RON serialization round-trip for a value
fn test_ron_roundtrip<T>(value: &T, approx_eq: impl Fn(&T, &T, f32) -> bool) -> bool
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    // Serialize to RON
    let ron_result = ron::to_string(value);
    if ron_result.is_err() {
        return false;
    }
    let ron_str = ron_result.unwrap();
    
    // Deserialize from RON
    let deserialize_result: Result<T, _> = ron::from_str(&ron_str);
    if deserialize_result.is_err() {
        return false;
    }
    let deserialized = deserialize_result.unwrap();
    
    // Check approximate equality (for floating point values)
    approx_eq(value, &deserialized, 1e-6)
}

/// Test binary serialization round-trip for a value
fn test_binary_roundtrip<T>(value: &T, approx_eq: impl Fn(&T, &T, f32) -> bool) -> bool
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    // Serialize to binary
    let binary_result = bincode::serialize(value);
    if binary_result.is_err() {
        return false;
    }
    let binary = binary_result.unwrap();
    
    // Deserialize from binary
    let deserialize_result: Result<T, _> = bincode::deserialize(&binary);
    if deserialize_result.is_err() {
        return false;
    }
    let deserialized = deserialize_result.unwrap();
    
    // Check approximate equality (for floating point values)
    approx_eq(value, &deserialized, 1e-6)
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 9.1: Vec3 RON Serialization Round-Trip
    ///
    /// For any Vec3 value, serializing to RON and deserializing should
    /// produce an equivalent value (within floating-point epsilon).
    #[test]
    fn prop_vec3_ron_roundtrip(vec in vec3_strategy()) {
        prop_assert!(
            test_ron_roundtrip(&vec, vec3_approx_eq),
            "Vec3 RON round-trip failed for {:?}", vec
        );
    }

    /// Property 9.2: Vec3 Binary Serialization Round-Trip
    ///
    /// For any Vec3 value, serializing to binary and deserializing should
    /// produce an equivalent value (within floating-point epsilon).
    #[test]
    fn prop_vec3_binary_roundtrip(vec in vec3_strategy()) {
        prop_assert!(
            test_binary_roundtrip(&vec, vec3_approx_eq),
            "Vec3 binary round-trip failed for {:?}", vec
        );
    }

    /// Property 9.3: Quat RON Serialization Round-Trip
    ///
    /// For any Quat value, serializing to RON and deserializing should
    /// produce an equivalent value (within floating-point epsilon).
    /// Note: q and -q represent the same rotation.
    #[test]
    fn prop_quat_ron_roundtrip(quat in quat_strategy()) {
        prop_assert!(
            test_ron_roundtrip(&quat, quat_approx_eq),
            "Quat RON round-trip failed for {:?}", quat
        );
    }

    /// Property 9.4: Quat Binary Serialization Round-Trip
    ///
    /// For any Quat value, serializing to binary and deserializing should
    /// produce an equivalent value (within floating-point epsilon).
    /// Note: q and -q represent the same rotation.
    #[test]
    fn prop_quat_binary_roundtrip(quat in quat_strategy()) {
        prop_assert!(
            test_binary_roundtrip(&quat, quat_approx_eq),
            "Quat binary round-trip failed for {:?}", quat
        );
    }

    /// Property 9.5: Transform RON Serialization Round-Trip
    ///
    /// For any Transform value, serializing to RON and deserializing should
    /// produce an equivalent value (within floating-point epsilon).
    #[test]
    fn prop_transform_ron_roundtrip(transform in transform_strategy()) {
        prop_assert!(
            test_ron_roundtrip(&transform, transform_approx_eq),
            "Transform RON round-trip failed for {:?}", transform
        );
    }

    /// Property 9.6: Transform Binary Serialization Round-Trip
    ///
    /// For any Transform value, serializing to binary and deserializing should
    /// produce an equivalent value (within floating-point epsilon).
    #[test]
    fn prop_transform_binary_roundtrip(transform in transform_strategy()) {
        prop_assert!(
            test_binary_roundtrip(&transform, transform_approx_eq),
            "Transform binary round-trip failed for {:?}", transform
        );
    }

    /// Property 9.7: Color RON Serialization Round-Trip
    ///
    /// For any Color value, serializing to RON and deserializing should
    /// produce an equivalent value (within floating-point epsilon).
    #[test]
    fn prop_color_ron_roundtrip(color in color_strategy()) {
        prop_assert!(
            test_ron_roundtrip(&color, color_approx_eq),
            "Color RON round-trip failed for {:?}", color
        );
    }

    /// Property 9.8: Color Binary Serialization Round-Trip
    ///
    /// For any Color value, serializing to binary and deserializing should
    /// produce an equivalent value (within floating-point epsilon).
    #[test]
    fn prop_color_binary_roundtrip(color in color_strategy()) {
        prop_assert!(
            test_binary_roundtrip(&color, color_approx_eq),
            "Color binary round-trip failed for {:?}", color
        );
    }
}

// ============================================================================
// Additional Integration Tests
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test that complex nested structures with all core types serialize correctly
    #[test]
    fn test_complex_structure_roundtrip() {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct ComplexData {
            position: Vec3,
            rotation: Quat,
            transform: Transform,
            color: Color,
            nested: Vec<Transform>,
        }

        let data = ComplexData {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
            transform: Transform {
                translation: Vec3::new(10.0, 20.0, 30.0),
                rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                scale: Vec3::splat(2.0),
            },
            color: Color::rgba(0.5, 0.75, 1.0, 0.8),
            nested: vec![
                Transform::from_xyz(1.0, 0.0, 0.0),
                Transform::from_xyz(0.0, 1.0, 0.0),
                Transform::from_xyz(0.0, 0.0, 1.0),
            ],
        };

        // Test RON
        let ron_str = ron::to_string(&data).expect("Failed to serialize to RON");
        let deserialized: ComplexData = ron::from_str(&ron_str)
            .expect("Failed to deserialize from RON");

        assert!(vec3_approx_eq(&data.position, &deserialized.position, 1e-6));
        assert!(quat_approx_eq(&data.rotation, &deserialized.rotation, 1e-6));
        assert!(transform_approx_eq(&data.transform, &deserialized.transform, 1e-6));
        assert!(color_approx_eq(&data.color, &deserialized.color, 1e-6));
        assert_eq!(data.nested.len(), deserialized.nested.len());

        // Test binary
        let binary = bincode::serialize(&data).expect("Failed to serialize to binary");
        let deserialized: ComplexData = bincode::deserialize(&binary)
            .expect("Failed to deserialize from binary");

        assert!(vec3_approx_eq(&data.position, &deserialized.position, 1e-6));
        assert!(quat_approx_eq(&data.rotation, &deserialized.rotation, 1e-6));
        assert!(transform_approx_eq(&data.transform, &deserialized.transform, 1e-6));
        assert!(color_approx_eq(&data.color, &deserialized.color, 1e-6));
        assert_eq!(data.nested.len(), deserialized.nested.len());
    }

    /// Test edge cases: zero vectors, identity quaternions, etc.
    #[test]
    fn test_edge_cases_roundtrip() {
        // Zero vector
        let zero = Vec3::ZERO;
        assert!(test_ron_roundtrip(&zero, vec3_approx_eq));
        assert!(test_binary_roundtrip(&zero, vec3_approx_eq));

        // Identity quaternion
        let identity = Quat::IDENTITY;
        assert!(test_ron_roundtrip(&identity, quat_approx_eq));
        assert!(test_binary_roundtrip(&identity, quat_approx_eq));

        // Identity transform
        let identity_transform = Transform::IDENTITY;
        assert!(test_ron_roundtrip(&identity_transform, transform_approx_eq));
        assert!(test_binary_roundtrip(&identity_transform, transform_approx_eq));

        // Transparent color
        let transparent = Color::TRANSPARENT;
        assert!(test_ron_roundtrip(&transparent, color_approx_eq));
        assert!(test_binary_roundtrip(&transparent, color_approx_eq));

        // White color
        let white = Color::WHITE;
        assert!(test_ron_roundtrip(&white, color_approx_eq));
        assert!(test_binary_roundtrip(&white, color_approx_eq));
    }

    /// Test that very large and very small values serialize correctly
    #[test]
    fn test_extreme_values_roundtrip() {
        // Very large values
        let large_vec = Vec3::new(1e10, 1e10, 1e10);
        assert!(test_ron_roundtrip(&large_vec, vec3_approx_eq));
        assert!(test_binary_roundtrip(&large_vec, vec3_approx_eq));

        // Very small values
        let small_vec = Vec3::new(1e-10, 1e-10, 1e-10);
        assert!(test_ron_roundtrip(&small_vec, vec3_approx_eq));
        assert!(test_binary_roundtrip(&small_vec, vec3_approx_eq));

        // Negative values
        let negative_vec = Vec3::new(-100.0, -200.0, -300.0);
        assert!(test_ron_roundtrip(&negative_vec, vec3_approx_eq));
        assert!(test_binary_roundtrip(&negative_vec, vec3_approx_eq));
    }
}


// ============================================================================
// Handle<T> Serialization Tests
// ============================================================================

#[cfg(test)]
mod handle_tests {
    use super::*;
    use luminara_asset::{Asset, AssetId, Handle};

    // Define a test asset type
    struct TestAsset;

    impl Asset for TestAsset {
        fn type_name() -> &'static str
        where
            Self: Sized,
        {
            "TestAsset"
        }
    }

    /// Strategy for generating random AssetId
    fn asset_id_strategy() -> impl Strategy<Value = AssetId> {
        prop::string::string_regex("[a-zA-Z0-9_/]{1,50}").unwrap()
            .prop_map(|path| AssetId::from_path(&path))
    }

    /// Strategy for generating random Handle<TestAsset>
    fn handle_strategy() -> impl Strategy<Value = Handle<TestAsset>> {
        (asset_id_strategy(), prop::num::u32::ANY)
            .prop_map(|(id, generation)| Handle::new(id, generation))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 9.9: Handle<T> RON Serialization Round-Trip
        ///
        /// For any Handle<T> value, serializing to RON and deserializing should
        /// produce an equivalent handle (same asset ID).
        #[test]
        fn prop_handle_ron_roundtrip(handle in handle_strategy()) {
            // Serialize to RON
            let ron_str = ron::to_string(&handle).expect("Failed to serialize Handle to RON");
            
            // Deserialize from RON
            let deserialized: Handle<TestAsset> = ron::from_str(&ron_str)
                .expect("Failed to deserialize Handle from RON");
            
            // Handles should be equal (same asset ID)
            prop_assert_eq!(handle, deserialized);
        }

        /// Property 9.10: Handle<T> Binary Serialization Round-Trip
        ///
        /// For any Handle<T> value, serializing to binary and deserializing should
        /// produce an equivalent handle (same asset ID).
        #[test]
        fn prop_handle_binary_roundtrip(handle in handle_strategy()) {
            // Serialize to binary
            let binary = bincode::serialize(&handle).expect("Failed to serialize Handle to binary");
            
            // Deserialize from binary
            let deserialized: Handle<TestAsset> = bincode::deserialize(&binary)
                .expect("Failed to deserialize Handle from binary");
            
            // Handles should be equal (same asset ID)
            prop_assert_eq!(handle, deserialized);
        }
    }

    #[test]
    fn test_handle_edge_cases() {
        // Test with specific asset paths
        let paths = vec![
            "textures/player.png",
            "models/enemy.gltf",
            "audio/music.ogg",
            "scenes/level1.scene",
        ];

        for path in paths {
            let id = AssetId::from_path(path);
            let handle: Handle<TestAsset> = Handle::new(id, 0);

            // Test RON
            let ron_str = ron::to_string(&handle).expect("Failed to serialize");
            let deserialized: Handle<TestAsset> = ron::from_str(&ron_str)
                .expect("Failed to deserialize");
            assert_eq!(handle, deserialized);

            // Test binary
            let binary = bincode::serialize(&handle).expect("Failed to serialize");
            let deserialized: Handle<TestAsset> = bincode::deserialize(&binary)
                .expect("Failed to deserialize");
            assert_eq!(handle, deserialized);
        }
    }

    #[test]
    fn test_handle_in_complex_structure() {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct EntityWithAssets {
            name: String,
            transform: Transform,
            texture: Handle<TestAsset>,
            model: Handle<TestAsset>,
            materials: Vec<Handle<TestAsset>>,
        }

        let entity = EntityWithAssets {
            name: "Player".to_string(),
            transform: Transform::from_xyz(1.0, 2.0, 3.0),
            texture: Handle::new(AssetId::from_path("textures/player.png"), 0),
            model: Handle::new(AssetId::from_path("models/player.gltf"), 0),
            materials: vec![
                Handle::new(AssetId::from_path("materials/skin.mat"), 0),
                Handle::new(AssetId::from_path("materials/armor.mat"), 0),
            ],
        };

        // Test RON
        let ron_str = ron::ser::to_string_pretty(&entity, ron::ser::PrettyConfig::default())
            .expect("Failed to serialize");
        let deserialized: EntityWithAssets = ron::from_str(&ron_str)
            .expect("Failed to deserialize");
        
        assert_eq!(entity.name, deserialized.name);
        assert_eq!(entity.texture, deserialized.texture);
        assert_eq!(entity.model, deserialized.model);
        assert_eq!(entity.materials.len(), deserialized.materials.len());

        // Test binary
        let binary = bincode::serialize(&entity).expect("Failed to serialize");
        let deserialized: EntityWithAssets = bincode::deserialize(&binary)
            .expect("Failed to deserialize");
        
        assert_eq!(entity.name, deserialized.name);
        assert_eq!(entity.texture, deserialized.texture);
        assert_eq!(entity.model, deserialized.model);
        assert_eq!(entity.materials.len(), deserialized.materials.len());
    }
}
