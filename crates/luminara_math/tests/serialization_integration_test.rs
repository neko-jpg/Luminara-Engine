/// Integration test for serialization of all core types
///
/// This test verifies that all core types (Vec3, Quat, Transform, Color, Handle<T>)
/// support both RON (human-readable) and binary serialization formats as required
/// by Requirements 8.1 and 8.4.
///
/// **Validates: Requirements 8.1, 8.4**

use luminara_asset::{Asset, AssetId, Handle};
use luminara_math::{Color, Quat, Transform, Vec3};
use serde::{Deserialize, Serialize};

// Test asset type for Handle<T> testing
struct TestAsset;
impl Asset for TestAsset {
    fn type_name() -> &'static str {
        "TestAsset"
    }
}

/// Comprehensive structure containing all core types
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SceneEntity {
    // Vec3 - position vector
    position: Vec3,
    // Quat - rotation quaternion
    rotation: Quat,
    // Transform - complete transform
    transform: Transform,
    // Color - RGBA color
    color: Color,
    // Handle<T> - asset reference
    texture: Handle<TestAsset>,
    model: Handle<TestAsset>,
    // Collections of core types
    waypoints: Vec<Vec3>,
    child_transforms: Vec<Transform>,
    palette: Vec<Color>,
}

impl SceneEntity {
    fn new_test_entity() -> Self {
        Self {
            position: Vec3::new(10.5, 20.3, 30.7),
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
            transform: Transform {
                translation: Vec3::new(100.0, 200.0, 300.0),
                rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                scale: Vec3::splat(2.5),
            },
            color: Color::rgba(0.8, 0.6, 0.4, 0.9),
            texture: Handle::new(AssetId::from_path("textures/entity.png"), 0),
            model: Handle::new(AssetId::from_path("models/entity.gltf"), 0),
            waypoints: vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(10.0, 0.0, 0.0),
                Vec3::new(10.0, 10.0, 0.0),
                Vec3::new(0.0, 10.0, 0.0),
            ],
            child_transforms: vec![
                Transform::from_xyz(1.0, 0.0, 0.0),
                Transform::from_xyz(0.0, 1.0, 0.0),
                Transform::from_xyz(0.0, 0.0, 1.0),
            ],
            palette: vec![
                Color::RED,
                Color::GREEN,
                Color::BLUE,
                Color::rgba(0.5, 0.5, 0.5, 1.0),
            ],
        }
    }
}

#[test]
fn test_ron_serialization_all_core_types() {
    let entity = SceneEntity::new_test_entity();

    // Serialize to RON (human-readable format)
    let ron_string = ron::ser::to_string_pretty(&entity, ron::ser::PrettyConfig::default())
        .expect("Failed to serialize to RON");

    // Verify RON string is human-readable
    assert!(ron_string.contains("position"));
    assert!(ron_string.contains("rotation"));
    assert!(ron_string.contains("transform"));
    assert!(ron_string.contains("color"));
    assert!(ron_string.contains("texture"));

    // Deserialize from RON
    let deserialized: SceneEntity =
        ron::from_str(&ron_string).expect("Failed to deserialize from RON");

    // Verify all fields are preserved
    assert_eq!(entity.position, deserialized.position);
    assert_eq!(entity.rotation, deserialized.rotation);
    assert_eq!(entity.transform, deserialized.transform);
    assert_eq!(entity.color, deserialized.color);
    assert_eq!(entity.texture, deserialized.texture);
    assert_eq!(entity.model, deserialized.model);
    assert_eq!(entity.waypoints.len(), deserialized.waypoints.len());
    assert_eq!(
        entity.child_transforms.len(),
        deserialized.child_transforms.len()
    );
    assert_eq!(entity.palette.len(), deserialized.palette.len());

    println!("✓ RON serialization test passed");
    println!("RON output sample:\n{}", &ron_string[..ron_string.len().min(500)]);
}

#[test]
fn test_binary_serialization_all_core_types() {
    let entity = SceneEntity::new_test_entity();

    // Serialize to binary format
    let binary = bincode::serialize(&entity).expect("Failed to serialize to binary");

    // Verify binary is compact (should be much smaller than RON)
    println!("Binary size: {} bytes", binary.len());
    assert!(binary.len() < 1000, "Binary serialization should be compact");

    // Deserialize from binary
    let deserialized: SceneEntity =
        bincode::deserialize(&binary).expect("Failed to deserialize from binary");

    // Verify all fields are preserved
    assert_eq!(entity.position, deserialized.position);
    assert_eq!(entity.rotation, deserialized.rotation);
    assert_eq!(entity.transform, deserialized.transform);
    assert_eq!(entity.color, deserialized.color);
    assert_eq!(entity.texture, deserialized.texture);
    assert_eq!(entity.model, deserialized.model);
    assert_eq!(entity.waypoints.len(), deserialized.waypoints.len());
    assert_eq!(
        entity.child_transforms.len(),
        deserialized.child_transforms.len()
    );
    assert_eq!(entity.palette.len(), deserialized.palette.len());

    println!("✓ Binary serialization test passed");
}

#[test]
fn test_ron_vs_binary_size_comparison() {
    let entity = SceneEntity::new_test_entity();

    // Serialize to both formats
    let ron_string = ron::to_string(&entity).expect("Failed to serialize to RON");
    let binary = bincode::serialize(&entity).expect("Failed to serialize to binary");

    println!("RON size: {} bytes", ron_string.len());
    println!("Binary size: {} bytes", binary.len());
    println!(
        "Binary is {:.1}x smaller than RON",
        ron_string.len() as f32 / binary.len() as f32
    );

    // Binary should be significantly smaller
    assert!(
        binary.len() < ron_string.len(),
        "Binary format should be more compact than RON"
    );
}

#[test]
fn test_vec3_serialization_formats() {
    let vec = Vec3::new(1.5, 2.5, 3.5);

    // RON format
    let ron = ron::to_string(&vec).expect("Failed to serialize Vec3 to RON");
    println!("Vec3 RON: {}", ron);
    let deserialized: Vec3 = ron::from_str(&ron).expect("Failed to deserialize Vec3 from RON");
    assert_eq!(vec, deserialized);

    // Binary format
    let binary = bincode::serialize(&vec).expect("Failed to serialize Vec3 to binary");
    println!("Vec3 binary size: {} bytes", binary.len());
    let deserialized: Vec3 =
        bincode::deserialize(&binary).expect("Failed to deserialize Vec3 from binary");
    assert_eq!(vec, deserialized);
}

#[test]
fn test_quat_serialization_formats() {
    let quat = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);

    // RON format
    let ron = ron::to_string(&quat).expect("Failed to serialize Quat to RON");
    println!("Quat RON: {}", ron);
    let deserialized: Quat = ron::from_str(&ron).expect("Failed to deserialize Quat from RON");
    // Quaternions q and -q represent the same rotation
    assert!(
        (quat.x - deserialized.x).abs() < 1e-6
            && (quat.y - deserialized.y).abs() < 1e-6
            && (quat.z - deserialized.z).abs() < 1e-6
            && (quat.w - deserialized.w).abs() < 1e-6
    );

    // Binary format
    let binary = bincode::serialize(&quat).expect("Failed to serialize Quat to binary");
    println!("Quat binary size: {} bytes", binary.len());
    let deserialized: Quat =
        bincode::deserialize(&binary).expect("Failed to deserialize Quat from binary");
    assert!(
        (quat.x - deserialized.x).abs() < 1e-6
            && (quat.y - deserialized.y).abs() < 1e-6
            && (quat.z - deserialized.z).abs() < 1e-6
            && (quat.w - deserialized.w).abs() < 1e-6
    );
}

#[test]
fn test_transform_serialization_formats() {
    let transform = Transform {
        translation: Vec3::new(10.0, 20.0, 30.0),
        rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        scale: Vec3::splat(2.0),
    };

    // RON format
    let ron = ron::ser::to_string_pretty(&transform, ron::ser::PrettyConfig::default())
        .expect("Failed to serialize Transform to RON");
    println!("Transform RON:\n{}", ron);
    let deserialized: Transform =
        ron::from_str(&ron).expect("Failed to deserialize Transform from RON");
    assert_eq!(transform, deserialized);

    // Binary format
    let binary = bincode::serialize(&transform).expect("Failed to serialize Transform to binary");
    println!("Transform binary size: {} bytes", binary.len());
    let deserialized: Transform =
        bincode::deserialize(&binary).expect("Failed to deserialize Transform from binary");
    assert_eq!(transform, deserialized);
}

#[test]
fn test_color_serialization_formats() {
    let color = Color::rgba(0.8, 0.6, 0.4, 0.9);

    // RON format
    let ron = ron::to_string(&color).expect("Failed to serialize Color to RON");
    println!("Color RON: {}", ron);
    let deserialized: Color = ron::from_str(&ron).expect("Failed to deserialize Color from RON");
    assert_eq!(color, deserialized);

    // Binary format
    let binary = bincode::serialize(&color).expect("Failed to serialize Color to binary");
    println!("Color binary size: {} bytes", binary.len());
    let deserialized: Color =
        bincode::deserialize(&binary).expect("Failed to deserialize Color from binary");
    assert_eq!(color, deserialized);
}

#[test]
fn test_handle_serialization_formats() {
    let handle: Handle<TestAsset> = Handle::new(AssetId::from_path("assets/test.png"), 0);

    // RON format
    let ron = ron::to_string(&handle).expect("Failed to serialize Handle to RON");
    println!("Handle RON: {}", ron);
    let deserialized: Handle<TestAsset> =
        ron::from_str(&ron).expect("Failed to deserialize Handle from RON");
    assert_eq!(handle, deserialized);

    // Binary format
    let binary = bincode::serialize(&handle).expect("Failed to serialize Handle to binary");
    println!("Handle binary size: {} bytes", binary.len());
    let deserialized: Handle<TestAsset> =
        bincode::deserialize(&binary).expect("Failed to deserialize Handle from binary");
    assert_eq!(handle, deserialized);
}

#[test]
fn test_scene_file_simulation() {
    // Simulate a scene file with multiple entities
    #[derive(Debug, Serialize, Deserialize)]
    struct Scene {
        name: String,
        entities: Vec<SceneEntity>,
    }

    let scene = Scene {
        name: "TestScene".to_string(),
        entities: vec![
            SceneEntity::new_test_entity(),
            SceneEntity::new_test_entity(),
            SceneEntity::new_test_entity(),
        ],
    };

    // Save as RON (human-editable scene file)
    let ron_scene = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default())
        .expect("Failed to serialize scene to RON");

    println!("Scene RON size: {} bytes", ron_scene.len());
    println!(
        "Scene RON preview:\n{}",
        &ron_scene[..ron_scene.len().min(300)]
    );

    // Load from RON
    let loaded_scene: Scene =
        ron::from_str(&ron_scene).expect("Failed to deserialize scene from RON");
    assert_eq!(scene.name, loaded_scene.name);
    assert_eq!(scene.entities.len(), loaded_scene.entities.len());

    // Save as binary (optimized runtime format)
    let binary_scene = bincode::serialize(&scene).expect("Failed to serialize scene to binary");
    println!("Scene binary size: {} bytes", binary_scene.len());
    println!(
        "Binary is {:.1}x smaller than RON",
        ron_scene.len() as f32 / binary_scene.len() as f32
    );

    // Load from binary
    let loaded_scene: Scene =
        bincode::deserialize(&binary_scene).expect("Failed to deserialize scene from binary");
    assert_eq!(scene.name, loaded_scene.name);
    assert_eq!(scene.entities.len(), loaded_scene.entities.len());
}

#[test]
fn test_all_core_types_individually() {
    println!("\n=== Testing Individual Core Type Serialization ===\n");

    // Vec3
    let vec3 = Vec3::new(1.0, 2.0, 3.0);
    let ron = ron::to_string(&vec3).unwrap();
    let binary = bincode::serialize(&vec3).unwrap();
    println!("Vec3: RON={} bytes, Binary={} bytes", ron.len(), binary.len());
    assert_eq!(vec3, ron::from_str::<Vec3>(&ron).unwrap());
    assert_eq!(vec3, bincode::deserialize::<Vec3>(&binary).unwrap());

    // Quat
    let quat = Quat::from_rotation_y(1.0);
    let ron = ron::to_string(&quat).unwrap();
    let binary = bincode::serialize(&quat).unwrap();
    println!(
        "Quat: RON={} bytes, Binary={} bytes",
        ron.len(),
        binary.len()
    );
    let deserialized = ron::from_str::<Quat>(&ron).unwrap();
    assert!((quat.x - deserialized.x).abs() < 1e-6);
    let deserialized = bincode::deserialize::<Quat>(&binary).unwrap();
    assert!((quat.x - deserialized.x).abs() < 1e-6);

    // Transform
    let transform = Transform::from_xyz(1.0, 2.0, 3.0);
    let ron = ron::to_string(&transform).unwrap();
    let binary = bincode::serialize(&transform).unwrap();
    println!(
        "Transform: RON={} bytes, Binary={} bytes",
        ron.len(),
        binary.len()
    );
    assert_eq!(transform, ron::from_str::<Transform>(&ron).unwrap());
    assert_eq!(
        transform,
        bincode::deserialize::<Transform>(&binary).unwrap()
    );

    // Color
    let color = Color::rgba(0.5, 0.6, 0.7, 0.8);
    let ron = ron::to_string(&color).unwrap();
    let binary = bincode::serialize(&color).unwrap();
    println!(
        "Color: RON={} bytes, Binary={} bytes",
        ron.len(),
        binary.len()
    );
    assert_eq!(color, ron::from_str::<Color>(&ron).unwrap());
    assert_eq!(color, bincode::deserialize::<Color>(&binary).unwrap());

    // Handle<T>
    let handle: Handle<TestAsset> = Handle::new(AssetId::from_path("test.asset"), 0);
    let ron = ron::to_string(&handle).unwrap();
    let binary = bincode::serialize(&handle).unwrap();
    println!(
        "Handle<T>: RON={} bytes, Binary={} bytes",
        ron.len(),
        binary.len()
    );
    assert_eq!(handle, ron::from_str::<Handle<TestAsset>>(&ron).unwrap());
    assert_eq!(
        handle,
        bincode::deserialize::<Handle<TestAsset>>(&binary).unwrap()
    );

    println!("\n✓ All core types support both RON and binary serialization");
}
