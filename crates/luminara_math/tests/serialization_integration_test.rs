use luminara_math::{Color, Quat, Transform, Vec3};
use serde::{Deserialize, Serialize};

/// A complex scene structure that uses all core serializable types
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SceneEntity {
    name: String,
    transform: Transform,
    color: Color,
    velocity: Vec3,
    angular_velocity: Quat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComplexScene {
    entities: Vec<SceneEntity>,
    ambient_color: Color,
    gravity: Vec3,
}

#[test]
fn test_complex_scene_ron_serialization() {
    let scene = ComplexScene {
        entities: vec![
            SceneEntity {
                name: "Player".to_string(),
                transform: Transform {
                    translation: Vec3::new(0.0, 1.0, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                color: Color::BLUE,
                velocity: Vec3::new(1.0, 0.0, 0.0),
                angular_velocity: Quat::from_rotation_y(0.1),
            },
            SceneEntity {
                name: "Enemy".to_string(),
                transform: Transform {
                    translation: Vec3::new(5.0, 0.0, 5.0),
                    rotation: Quat::from_rotation_y(std::f32::consts::PI),
                    scale: Vec3::splat(1.5),
                },
                color: Color::RED,
                velocity: Vec3::new(-0.5, 0.0, -0.5),
                angular_velocity: Quat::from_rotation_z(0.05),
            },
        ],
        ambient_color: Color::rgba(0.2, 0.2, 0.3, 1.0),
        gravity: Vec3::new(0.0, -9.81, 0.0),
    };
    
    // Serialize to RON
    let ron_str = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default())
        .expect("Failed to serialize scene to RON");
    
    println!("Complex Scene RON:\n{}", ron_str);
    
    // Deserialize from RON
    let deserialized: ComplexScene = ron::from_str(&ron_str)
        .expect("Failed to deserialize scene from RON");
    
    // Verify entities
    assert_eq!(scene.entities.len(), deserialized.entities.len());
    
    for (original, deserialized) in scene.entities.iter().zip(deserialized.entities.iter()) {
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.transform.translation, deserialized.transform.translation);
        assert_eq!(original.transform.scale, deserialized.transform.scale);
        assert_eq!(original.color, deserialized.color);
        assert_eq!(original.velocity, deserialized.velocity);
    }
    
    // Verify scene properties
    assert_eq!(scene.ambient_color, deserialized.ambient_color);
    assert_eq!(scene.gravity, deserialized.gravity);
}

#[test]
fn test_complex_scene_binary_serialization() {
    let scene = ComplexScene {
        entities: vec![
            SceneEntity {
                name: "Light".to_string(),
                transform: Transform {
                    translation: Vec3::new(0.0, 10.0, 0.0),
                    rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
                    scale: Vec3::ONE,
                },
                color: Color::WHITE,
                velocity: Vec3::ZERO,
                angular_velocity: Quat::IDENTITY,
            },
        ],
        ambient_color: Color::GRAY,
        gravity: Vec3::new(0.0, -9.81, 0.0),
    };
    
    // Serialize to binary
    let binary = bincode::serialize(&scene).expect("Failed to serialize scene to binary");
    
    println!("Binary size: {} bytes", binary.len());
    
    // Deserialize from binary
    let deserialized: ComplexScene = bincode::deserialize(&binary)
        .expect("Failed to deserialize scene from binary");
    
    // Verify
    assert_eq!(scene.entities.len(), deserialized.entities.len());
    assert_eq!(scene.entities[0].name, deserialized.entities[0].name);
    assert_eq!(scene.ambient_color, deserialized.ambient_color);
    assert_eq!(scene.gravity, deserialized.gravity);
}

#[test]
fn test_transform_hierarchy_serialization() {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TransformNode {
        name: String,
        local_transform: Transform,
        children: Vec<TransformNode>,
    }
    
    let hierarchy = TransformNode {
        name: "Root".to_string(),
        local_transform: Transform::from_xyz(0.0, 0.0, 0.0),
        children: vec![
            TransformNode {
                name: "Child1".to_string(),
                local_transform: Transform::from_xyz(1.0, 0.0, 0.0),
                children: vec![
                    TransformNode {
                        name: "Grandchild1".to_string(),
                        local_transform: Transform::from_xyz(0.0, 1.0, 0.0),
                        children: vec![],
                    },
                ],
            },
            TransformNode {
                name: "Child2".to_string(),
                local_transform: Transform::from_xyz(-1.0, 0.0, 0.0),
                children: vec![],
            },
        ],
    };
    
    // Test RON serialization
    let ron_str = ron::ser::to_string_pretty(&hierarchy, ron::ser::PrettyConfig::default())
        .expect("Failed to serialize hierarchy to RON");
    
    println!("Transform Hierarchy RON:\n{}", ron_str);
    
    let deserialized: TransformNode = ron::from_str(&ron_str)
        .expect("Failed to deserialize hierarchy from RON");
    
    assert_eq!(hierarchy.name, deserialized.name);
    assert_eq!(hierarchy.children.len(), deserialized.children.len());
    assert_eq!(hierarchy.children[0].children.len(), deserialized.children[0].children.len());
    
    // Test binary serialization
    let binary = bincode::serialize(&hierarchy).expect("Failed to serialize hierarchy to binary");
    let deserialized_binary: TransformNode = bincode::deserialize(&binary)
        .expect("Failed to deserialize hierarchy from binary");
    
    assert_eq!(hierarchy.name, deserialized_binary.name);
    assert_eq!(hierarchy.children.len(), deserialized_binary.children.len());
}

#[test]
fn test_color_palette_serialization() {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ColorPalette {
        name: String,
        colors: Vec<Color>,
    }
    
    let palette = ColorPalette {
        name: "Sunset".to_string(),
        colors: vec![
            Color::rgb(1.0, 0.5, 0.0),  // Orange
            Color::rgb(1.0, 0.3, 0.3),  // Red-orange
            Color::rgb(0.8, 0.2, 0.5),  // Purple
            Color::rgb(0.2, 0.2, 0.4),  // Dark blue
        ],
    };
    
    // Test RON
    let ron_str = ron::ser::to_string_pretty(&palette, ron::ser::PrettyConfig::default())
        .expect("Failed to serialize palette to RON");
    
    println!("Color Palette RON:\n{}", ron_str);
    
    let deserialized: ColorPalette = ron::from_str(&ron_str)
        .expect("Failed to deserialize palette from RON");
    
    assert_eq!(palette.name, deserialized.name);
    assert_eq!(palette.colors.len(), deserialized.colors.len());
    
    for (original, deserialized) in palette.colors.iter().zip(deserialized.colors.iter()) {
        assert_eq!(original, deserialized);
    }
    
    // Test binary
    let binary = bincode::serialize(&palette).expect("Failed to serialize palette to binary");
    let deserialized_binary: ColorPalette = bincode::deserialize(&binary)
        .expect("Failed to deserialize palette from binary");
    
    assert_eq!(palette.colors, deserialized_binary.colors);
}

#[test]
fn test_serialization_format_comparison() {
    let transform = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
        scale: Vec3::splat(2.0),
    };
    
    // RON serialization
    let ron_str = ron::to_string(&transform).unwrap();
    let ron_size = ron_str.len();
    
    // Binary serialization
    let binary = bincode::serialize(&transform).unwrap();
    let binary_size = binary.len();
    
    println!("Transform serialization comparison:");
    println!("  RON size: {} bytes", ron_size);
    println!("  Binary size: {} bytes", binary_size);
    println!("  Compression ratio: {:.2}x", ron_size as f32 / binary_size as f32);
    
    // Binary should be more compact
    assert!(binary_size < ron_size);
    
    // Both should deserialize correctly
    let from_ron: Transform = ron::from_str(&ron_str).unwrap();
    let from_binary: Transform = bincode::deserialize(&binary).unwrap();
    
    assert_eq!(transform, from_ron);
    assert_eq!(transform, from_binary);
}
