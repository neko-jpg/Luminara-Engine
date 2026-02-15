use luminara_math::{Color, Quat, Transform, Vec3};
use serde::{Deserialize, Serialize};

#[test]
fn test_vec3_ron_serialization() {
    let vec = Vec3::new(1.0, 2.0, 3.0);
    
    // Serialize to RON
    let ron_str = ron::to_string(&vec).expect("Failed to serialize Vec3 to RON");
    println!("Vec3 RON: {}", ron_str);
    
    // Deserialize from RON
    let deserialized: Vec3 = ron::from_str(&ron_str).expect("Failed to deserialize Vec3 from RON");
    
    assert_eq!(vec, deserialized);
}

#[test]
fn test_vec3_binary_serialization() {
    let vec = Vec3::new(1.0, 2.0, 3.0);
    
    // Serialize to binary
    let binary = bincode::serialize(&vec).expect("Failed to serialize Vec3 to binary");
    
    // Deserialize from binary
    let deserialized: Vec3 = bincode::deserialize(&binary).expect("Failed to deserialize Vec3 from binary");
    
    assert_eq!(vec, deserialized);
}

#[test]
fn test_quat_ron_serialization() {
    let quat = Quat::from_rotation_y(std::f32::consts::PI / 4.0);
    
    // Serialize to RON
    let ron_str = ron::to_string(&quat).expect("Failed to serialize Quat to RON");
    println!("Quat RON: {}", ron_str);
    
    // Deserialize from RON
    let deserialized: Quat = ron::from_str(&ron_str).expect("Failed to deserialize Quat from RON");
    
    // Use approximate comparison for floating point
    assert!((quat.x - deserialized.x).abs() < 1e-6);
    assert!((quat.y - deserialized.y).abs() < 1e-6);
    assert!((quat.z - deserialized.z).abs() < 1e-6);
    assert!((quat.w - deserialized.w).abs() < 1e-6);
}

#[test]
fn test_quat_binary_serialization() {
    let quat = Quat::from_rotation_y(std::f32::consts::PI / 4.0);
    
    // Serialize to binary
    let binary = bincode::serialize(&quat).expect("Failed to serialize Quat to binary");
    
    // Deserialize from binary
    let deserialized: Quat = bincode::deserialize(&binary).expect("Failed to deserialize Quat from binary");
    
    assert_eq!(quat, deserialized);
}

#[test]
fn test_transform_ron_serialization() {
    let transform = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
        scale: Vec3::new(2.0, 2.0, 2.0),
    };
    
    // Serialize to RON
    let ron_str = ron::to_string(&transform).expect("Failed to serialize Transform to RON");
    println!("Transform RON: {}", ron_str);
    
    // Deserialize from RON
    let deserialized: Transform = ron::from_str(&ron_str).expect("Failed to deserialize Transform from RON");
    
    // Compare fields
    assert_eq!(transform.translation, deserialized.translation);
    assert_eq!(transform.scale, deserialized.scale);
    
    // Use approximate comparison for rotation
    assert!((transform.rotation.x - deserialized.rotation.x).abs() < 1e-6);
    assert!((transform.rotation.y - deserialized.rotation.y).abs() < 1e-6);
    assert!((transform.rotation.z - deserialized.rotation.z).abs() < 1e-6);
    assert!((transform.rotation.w - deserialized.rotation.w).abs() < 1e-6);
}

#[test]
fn test_transform_binary_serialization() {
    let transform = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
        scale: Vec3::new(2.0, 2.0, 2.0),
    };
    
    // Serialize to binary
    let binary = bincode::serialize(&transform).expect("Failed to serialize Transform to binary");
    
    // Deserialize from binary
    let deserialized: Transform = bincode::deserialize(&binary).expect("Failed to deserialize Transform from binary");
    
    assert_eq!(transform, deserialized);
}

#[test]
fn test_color_ron_serialization() {
    let color = Color::rgba(0.5, 0.75, 1.0, 0.8);
    
    // Serialize to RON
    let ron_str = ron::to_string(&color).expect("Failed to serialize Color to RON");
    println!("Color RON: {}", ron_str);
    
    // Deserialize from RON
    let deserialized: Color = ron::from_str(&ron_str).expect("Failed to deserialize Color from RON");
    
    assert_eq!(color, deserialized);
}

#[test]
fn test_color_binary_serialization() {
    let color = Color::rgba(0.5, 0.75, 1.0, 0.8);
    
    // Serialize to binary
    let binary = bincode::serialize(&color).expect("Failed to serialize Color to binary");
    
    // Deserialize from binary
    let deserialized: Color = bincode::deserialize(&binary).expect("Failed to deserialize Color from binary");
    
    assert_eq!(color, deserialized);
}

#[test]
fn test_color_constants_serialization() {
    let colors = vec![
        Color::WHITE,
        Color::BLACK,
        Color::RED,
        Color::GREEN,
        Color::BLUE,
        Color::YELLOW,
        Color::CYAN,
        Color::MAGENTA,
        Color::GRAY,
        Color::TRANSPARENT,
    ];
    
    for color in colors {
        // Test RON
        let ron_str = ron::to_string(&color).expect("Failed to serialize color to RON");
        let deserialized: Color = ron::from_str(&ron_str).expect("Failed to deserialize color from RON");
        assert_eq!(color, deserialized);
        
        // Test binary
        let binary = bincode::serialize(&color).expect("Failed to serialize color to binary");
        let deserialized: Color = bincode::deserialize(&binary).expect("Failed to deserialize color from binary");
        assert_eq!(color, deserialized);
    }
}
