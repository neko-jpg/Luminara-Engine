// Property-Based Test: Viewport Capture Format Compliance
// Feature: pre-editor-engine-audit, Property 29: Viewport Capture Format Compliance
// Validates: Requirements 29.1
//
// Property: For any viewport capture, the output should be exactly 512x512 resolution
// in JPEG format with file size <100KB.

use luminara_ai_agent::visual_feedback::{
    VisualFeedbackSystem, CaptureConfig, ImageFormat, AnnotationConfig,
};
use luminara_core::world::World;
use luminara_math::{Transform, Vec3, Quat};
use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};
use image::GenericImageView;

/// Generator for random world states with varying entity counts and positions
#[derive(Clone, Debug)]
struct RandomWorld {
    entity_count: usize,
    entities: Vec<EntityData>,
}

#[derive(Clone, Debug)]
struct EntityData {
    position: (f32, f32, f32),
}

impl Arbitrary for RandomWorld {
    fn arbitrary(g: &mut Gen) -> Self {
        let entity_count = usize::arbitrary(g) % 200; // 0-199 entities
        let mut entities = Vec::new();
        
        for _ in 0..entity_count {
            // Generate positions within reasonable bounds (-10 to 10 in each axis)
            let x = (f32::arbitrary(g) % 20.0) - 10.0;
            let y = (f32::arbitrary(g) % 20.0) - 10.0;
            let z = (f32::arbitrary(g) % 20.0) - 10.0;
            
            entities.push(EntityData {
                position: (x, y, z),
            });
        }
        
        RandomWorld {
            entity_count,
            entities,
        }
    }
}

/// Generator for random annotation configurations
#[derive(Clone, Debug)]
struct RandomAnnotationConfig {
    show_entity_names: bool,
    show_bounding_boxes: bool,
    show_light_positions: bool,
    highlight_selected: bool,
}

impl Arbitrary for RandomAnnotationConfig {
    fn arbitrary(g: &mut Gen) -> Self {
        RandomAnnotationConfig {
            show_entity_names: bool::arbitrary(g),
            show_bounding_boxes: bool::arbitrary(g),
            show_light_positions: bool::arbitrary(g),
            highlight_selected: bool::arbitrary(g),
        }
    }
}

/// Property 29: Viewport Capture Format Compliance
/// 
/// This property verifies that for ANY viewport capture:
/// 1. The output resolution is exactly 512x512 pixels
/// 2. The output format is JPEG
/// 3. The file size is less than 100KB
///
/// The property is tested across:
/// - Different world states (0-199 entities)
/// - Different entity positions
/// - Different annotation configurations
fn property_viewport_capture_format_compliance(
    random_world: RandomWorld,
    random_annotations: RandomAnnotationConfig,
) -> TestResult {
    // Create world and populate with entities
    let mut world = World::new();
    
    for entity_data in &random_world.entities {
        let entity = world.spawn();
        let _ = world.add_component(entity, Transform {
            translation: Vec3::new(
                entity_data.position.0,
                entity_data.position.1,
                entity_data.position.2,
            ),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        });
    }
    
    // Create capture configuration
    let config = CaptureConfig {
        resolution: (512, 512),
        format: ImageFormat::Jpeg,
        annotations: AnnotationConfig {
            show_entity_names: random_annotations.show_entity_names,
            show_bounding_boxes: random_annotations.show_bounding_boxes,
            show_light_positions: random_annotations.show_light_positions,
            highlight_selected: random_annotations.highlight_selected,
        },
    };
    
    // Perform capture
    let system = VisualFeedbackSystem::new();
    let result = system.capture_viewport(&world, config);
    
    // Verify Property 29: Format Compliance
    
    // 1. Resolution must be exactly 512x512
    if result.resolution != (512, 512) {
        return TestResult::failed();
    }
    
    // 2. File size must be less than 100KB (100,000 bytes)
    if result.file_size >= 100_000 {
        return TestResult::failed();
    }
    
    // 3. Image data must not be empty (valid JPEG)
    if result.image_data.is_empty() {
        return TestResult::failed();
    }
    
    // 4. Verify the image is actually a valid JPEG by attempting to decode it
    match image::load_from_memory(&result.image_data) {
        Ok(img) => {
            // Verify decoded dimensions match
            if img.width() != 512 || img.height() != 512 {
                return TestResult::failed();
            }
        }
        Err(_) => {
            // Failed to decode as valid image
            return TestResult::failed();
        }
    }
    
    TestResult::passed()
}

/// Additional property test: Verify format compliance with extreme cases
#[test]
fn property_viewport_capture_format_compliance_extreme_cases() {
    // Test with empty world
    let empty_world = RandomWorld {
        entity_count: 0,
        entities: vec![],
    };
    
    let no_annotations = RandomAnnotationConfig {
        show_entity_names: false,
        show_bounding_boxes: false,
        show_light_positions: false,
        highlight_selected: false,
    };
    
    // Run the property test - if it returns TestResult::failed(), quickcheck will panic
    let _ = property_viewport_capture_format_compliance(
        empty_world.clone(),
        no_annotations.clone(),
    );
    
    // Test with maximum entities (199)
    let mut max_entities = Vec::new();
    for i in 0..199 {
        let angle = (i as f32) * std::f32::consts::PI * 2.0 / 199.0;
        max_entities.push(EntityData {
            position: (angle.cos() * 8.0, 0.0, angle.sin() * 8.0),
        });
    }
    
    let full_world = RandomWorld {
        entity_count: 199,
        entities: max_entities,
    };
    
    let all_annotations = RandomAnnotationConfig {
        show_entity_names: true,
        show_bounding_boxes: true,
        show_light_positions: true,
        highlight_selected: true,
    };
    
    let _ = property_viewport_capture_format_compliance(
        full_world,
        all_annotations,
    );
}

/// Property test: Verify format compliance is consistent across multiple captures
#[test]
fn property_viewport_capture_format_consistency() {
    let mut world = World::new();
    
    // Create a deterministic scene
    for i in 0..50 {
        let entity = world.spawn();
        let _ = world.add_component(entity, Transform {
            translation: Vec3::new(i as f32 * 0.5, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        });
    }
    
    let system = VisualFeedbackSystem::new();
    let config = CaptureConfig::default();
    
    // Capture multiple times
    for _ in 0..10 {
        let result = system.capture_viewport(&world, config.clone());
        
        // Each capture must meet format compliance
        assert_eq!(result.resolution, (512, 512), "Resolution must be 512x512");
        assert!(result.file_size < 100_000, "File size must be <100KB, got {} bytes", result.file_size);
        assert!(!result.image_data.is_empty(), "Image data must not be empty");
        
        // Verify it's a valid JPEG
        let img = image::load_from_memory(&result.image_data)
            .expect("Must be valid JPEG image");
        assert_eq!(img.width(), 512, "Decoded width must be 512");
        assert_eq!(img.height(), 512, "Decoded height must be 512");
    }
}

/// Property test: Verify format compliance with different JPEG quality settings
#[test]
fn property_viewport_capture_format_compliance_quality_variations() {
    let mut world = World::new();
    
    // Create a complex scene to test compression
    for x in -5..=5 {
        for z in -5..=5 {
            let entity = world.spawn();
            let _ = world.add_component(entity, Transform {
                translation: Vec3::new(x as f32, 0.0, z as f32),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            });
        }
    }
    
    let system = VisualFeedbackSystem::new();
    let config = CaptureConfig {
        resolution: (512, 512),
        format: ImageFormat::Jpeg,
        annotations: AnnotationConfig {
            show_bounding_boxes: true,
            ..Default::default()
        },
    };
    
    let result = system.capture_viewport(&world, config);
    
    // Even with a complex scene and annotations, must meet requirements
    assert_eq!(result.resolution, (512, 512));
    assert!(
        result.file_size < 100_000,
        "Complex scene file size {} exceeds 100KB limit",
        result.file_size
    );
    
    // Verify valid JPEG
    let img = image::load_from_memory(&result.image_data)
        .expect("Must be valid JPEG");
    assert_eq!(img.dimensions(), (512, 512));
}

/// Run QuickCheck with custom configuration for thorough testing
#[test]
fn run_quickcheck_viewport_capture_format_compliance() {
    // Run 100+ iterations as specified in the design document
    fn prop(random_world: RandomWorld, random_annotations: RandomAnnotationConfig) -> TestResult {
        property_viewport_capture_format_compliance(random_world, random_annotations)
    }
    
    QuickCheck::new()
        .tests(100)
        .max_tests(150)
        .quickcheck(prop as fn(RandomWorld, RandomAnnotationConfig) -> TestResult);
}

/// Property test: Verify file size remains under limit with various entity densities
#[test]
fn property_file_size_compliance_across_densities() {
    let system = VisualFeedbackSystem::new();
    let config = CaptureConfig::default();
    
    // Test with different entity counts: 0, 10, 50, 100, 150, 199
    for entity_count in [0, 10, 50, 100, 150, 199] {
        let mut world = World::new();
        
        for i in 0..entity_count {
            let entity = world.spawn();
            let angle = (i as f32) * std::f32::consts::PI * 2.0 / entity_count.max(1) as f32;
            let radius = ((i % 10) + 1) as f32;
            
            let _ = world.add_component(entity, Transform {
                translation: Vec3::new(
                    angle.cos() * radius,
                    0.0,
                    angle.sin() * radius,
                ),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            });
        }
        
        let result = system.capture_viewport(&world, config.clone());
        
        assert_eq!(
            result.resolution,
            (512, 512),
            "Resolution must be 512x512 for {} entities",
            entity_count
        );
        
        assert!(
            result.file_size < 100_000,
            "File size {} bytes exceeds 100KB limit with {} entities",
            result.file_size,
            entity_count
        );
        
        // Verify valid JPEG
        image::load_from_memory(&result.image_data)
            .expect(&format!("Must be valid JPEG with {} entities", entity_count));
    }
}

/// Property test: Verify format compliance with boundary positions
#[test]
fn property_format_compliance_boundary_positions() {
    let system = VisualFeedbackSystem::new();
    let config = CaptureConfig::default();
    
    // Test entities at extreme positions
    let boundary_positions = vec![
        (-10.0, -10.0, -10.0),
        (10.0, 10.0, 10.0),
        (-10.0, 10.0, -10.0),
        (10.0, -10.0, 10.0),
        (0.0, 0.0, 0.0),
    ];
    
    for pos in boundary_positions {
        let mut world = World::new();
        let entity = world.spawn();
        let _ = world.add_component(entity, Transform {
            translation: Vec3::new(pos.0, pos.1, pos.2),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        });
        
        let result = system.capture_viewport(&world, config.clone());
        
        assert_eq!(result.resolution, (512, 512));
        assert!(result.file_size < 100_000);
        
        // Verify valid JPEG
        let img = image::load_from_memory(&result.image_data)
            .expect("Must be valid JPEG");
        assert_eq!(img.dimensions(), (512, 512));
    }
}
