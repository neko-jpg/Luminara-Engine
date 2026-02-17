// ============================================================================
// Deserialization Validation Demo
// ============================================================================
//
// This example demonstrates the deserialization validation system,
// showing how it catches invalid data and provides helpful error messages.

use luminara_math::validation::{from_ron_validated, Validate};
use luminara_math::{Color, Quat, Transform, Vec3};

fn main() {
    println!("=== Deserialization Validation Demo ===\n");

    // Example 1: Valid Vec3
    println!("1. Valid Vec3:");
    let valid_vec3 = Vec3::new(1.0, 2.0, 3.0);
    let valid_vec3_ron = ron::to_string(&valid_vec3).unwrap();
    println!("   RON: {}", valid_vec3_ron);
    match from_ron_validated::<Vec3>(&valid_vec3_ron) {
        Ok(vec) => println!("   ✓ Successfully deserialized: {:?}\n", vec),
        Err(e) => println!("   ✗ Error: {}\n", e),
    }

    // Example 2: Invalid Vec3 (NaN)
    println!("2. Invalid Vec3 (NaN):");
    let vec_with_nan = Vec3::new(f32::NAN, 2.0, 3.0);
    match vec_with_nan.validate() {
        Ok(_) => println!("   ✓ Valid\n"),
        Err(e) => println!("   ✗ {}\n", e),
    }

    // Example 3: Valid Transform
    println!("3. Valid Transform:");
    let valid_transform = Transform::from_xyz(1.0, 2.0, 3.0);
    match valid_transform.validate() {
        Ok(_) => println!("   ✓ Valid transform\n"),
        Err(e) => println!("   ✗ {}\n", e),
    }

    // Example 4: Invalid Transform (unnormalized quaternion)
    println!("4. Invalid Transform (unnormalized quaternion):");
    let invalid_transform = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_xyzw(1.0, 1.0, 1.0, 1.0), // Not normalized
        scale: Vec3::splat(1.0),
    };
    match invalid_transform.validate() {
        Ok(_) => println!("   ✓ Valid\n"),
        Err(e) => println!("   ✗ {}\n", e),
    }

    // Example 5: Invalid Transform (negative scale)
    println!("5. Invalid Transform (negative scale):");
    let negative_scale_transform = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::new(1.0, -1.0, 1.0),
    };
    match negative_scale_transform.validate() {
        Ok(_) => println!("   ✓ Valid\n"),
        Err(e) => println!("   ✗ {}\n", e),
    }

    // Example 6: Valid Color
    println!("6. Valid Color:");
    let valid_color = Color::rgba(0.5, 0.75, 1.0, 0.8);
    match valid_color.validate() {
        Ok(_) => println!("   ✓ Valid color: {:?}\n", valid_color),
        Err(e) => println!("   ✗ {}\n", e),
    }

    // Example 7: Invalid Color (out of range)
    println!("7. Invalid Color (value > 1.0):");
    let invalid_color = Color::rgba(1.5, 0.5, 0.5, 1.0);
    match invalid_color.validate() {
        Ok(_) => println!("   ✓ Valid\n"),
        Err(e) => println!("   ✗ {}\n", e),
    }

    // Example 8: Invalid Color (negative value)
    println!("8. Invalid Color (negative value):");
    let negative_color = Color::rgba(0.5, -0.1, 0.5, 1.0);
    match negative_color.validate() {
        Ok(_) => println!("   ✓ Valid\n"),
        Err(e) => println!("   ✗ {}\n", e),
    }

    // Example 9: Deserialization with validation from RON
    println!("9. Deserializing invalid Transform from RON:");
    let invalid_transform_for_ron = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_xyzw(1.0, 1.0, 1.0, 1.0), // Not normalized
        scale: Vec3::splat(1.0),
    };
    let invalid_transform_ron = ron::to_string(&invalid_transform_for_ron).unwrap();
    println!("   RON: {}", invalid_transform_ron);
    match from_ron_validated::<Transform>(&invalid_transform_ron) {
        Ok(_) => println!("   ✓ Successfully deserialized\n"),
        Err(e) => println!("   ✗ {}\n", e),
    }

    println!("=== Demo Complete ===");
}
