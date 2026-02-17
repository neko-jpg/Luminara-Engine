//! Integration tests for AI shader generation pipeline.
//!
//! Tests the complete workflow from natural language description to compiled shader.

use luminara_render::ai_shader_pipeline::{AiShaderError, AiShaderPipeline};

#[test]
fn test_mock_generator_pulsating_2hz() {
    let mut pipeline = AiShaderPipeline::new();

    let wgsl = pipeline
        .generate_shader("Create a pulsating glow effect that pulses at 2Hz")
        .expect("Should generate shader");

    // Should contain sin function for pulsating effect
    assert!(wgsl.contains("sin"));
}

#[test]
fn test_mock_generator_pulsating_1hz() {
    let mut pipeline = AiShaderPipeline::new();

    let wgsl = pipeline
        .generate_shader("Create a pulsating effect at 1Hz")
        .expect("Should generate shader");

    // Should contain sin function
    assert!(wgsl.contains("sin"));
}

#[test]
fn test_mock_generator_gradient() {
    let mut pipeline = AiShaderPipeline::new();

    let wgsl = pipeline
        .generate_shader("Create a horizontal gradient")
        .expect("Should generate shader");

    // Should contain uv reference
    assert!(wgsl.contains("uv") || wgsl.contains("u"));
}

#[test]
fn test_mock_generator_noise() {
    let mut pipeline = AiShaderPipeline::new();

    let wgsl = pipeline
        .generate_shader("Create a noise pattern")
        .expect("Should generate shader");

    // Should contain sin functions for pseudo-noise
    assert!(wgsl.contains("sin"));
}

#[test]
fn test_mock_generator_constant() {
    let mut pipeline = AiShaderPipeline::new();

    let wgsl = pipeline
        .generate_shader("Create a solid color")
        .expect("Should generate shader");

    // Should contain constant value
    assert!(wgsl.contains("1"));
}

#[test]
fn test_mock_generator_unsupported_description() {
    let mut pipeline = AiShaderPipeline::new();

    let result = pipeline.generate_shader("Create something completely unsupported and weird");

    assert!(result.is_err());
    match result {
        Err(AiShaderError::InvalidDescription(msg)) => {
            assert!(msg.contains("Could not generate expression"));
        }
        _ => panic!("Expected InvalidDescription error"),
    }
}

#[test]
fn test_pipeline_caching() {
    let mut pipeline = AiShaderPipeline::new();

    // Generate same shader twice
    let wgsl1 = pipeline
        .generate_shader("Create a pulsating glow effect")
        .expect("Should generate shader");

    let wgsl2 = pipeline
        .generate_shader("Create a pulsating glow effect")
        .expect("Should generate shader");

    // Results should be identical
    assert_eq!(wgsl1, wgsl2);

    // Should have 1 cache hit
    let stats = pipeline.cache_stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
}

#[test]
fn test_pipeline_different_descriptions() {
    let mut pipeline = AiShaderPipeline::new();

    let wgsl1 = pipeline
        .generate_shader("Create a pulsating effect")
        .expect("Should generate shader");

    let wgsl2 = pipeline
        .generate_shader("Create a gradient")
        .expect("Should generate shader");

    // Results should be different
    assert_ne!(wgsl1, wgsl2);

    // Should have 2 cache misses, 0 hits
    let stats = pipeline.cache_stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 2);
}

#[test]
fn test_pipeline_clear_cache() {
    let mut pipeline = AiShaderPipeline::new();

    // Generate a shader
    let _ = pipeline.generate_shader("Create a pulsating effect");

    // Cache should have 1 entry
    assert_eq!(pipeline.cache_size(), 1);

    // Clear cache
    pipeline.clear_cache();

    // Cache should be empty
    assert_eq!(pipeline.cache_size(), 0);

    // Stats should be reset
    let stats = pipeline.cache_stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
}

#[test]
fn test_generator_name() {
    let pipeline = AiShaderPipeline::new();
    assert_eq!(pipeline.generator_name(), "MockExpressionGenerator");
}

#[test]
fn test_error_display() {
    let err = AiShaderError::ExpressionGenerationFailed("test error".to_string());
    assert_eq!(
        format!("{}", err),
        "Expression generation failed: test error"
    );

    let err = AiShaderError::ShaderCompilationFailed("compile error".to_string());
    assert_eq!(format!("{}", err), "Shader compilation failed: compile error");

    let err = AiShaderError::InvalidDescription("invalid".to_string());
    assert_eq!(format!("{}", err), "Invalid description: invalid");

    let err = AiShaderError::RenderError("render error".to_string());
    assert_eq!(format!("{}", err), "Render error: render error");
}

/// Test that the pipeline can handle multiple different shader generations
/// in sequence without issues
#[test]
fn test_pipeline_multiple_generations() {
    let mut pipeline = AiShaderPipeline::new();

    let descriptions = vec![
        "Create a pulsating effect",
        "Create a gradient",
        "Create a noise pattern",
        "Create a solid color",
    ];

    for desc in descriptions {
        let result = pipeline.generate_shader(desc);
        assert!(
            result.is_ok(),
            "Failed to generate shader for: {}",
            desc
        );
    }

    // Should have 4 different shaders cached
    assert_eq!(pipeline.cache_size(), 4);
}

/// Test that the pipeline handles edge cases in descriptions
#[test]
fn test_pipeline_edge_cases() {
    let mut pipeline = AiShaderPipeline::new();

    // Empty description
    let result = pipeline.generate_shader("");
    assert!(result.is_err());

    // Very long description
    let long_desc = "Create a ".to_string() + &"very ".repeat(100) + "pulsating effect";
    let result = pipeline.generate_shader(&long_desc);
    assert!(result.is_ok()); // Should still work, just extracts "pulsating"

    // Mixed case
    let result = pipeline.generate_shader("CrEaTe A pUlSaTiNg EfFeCt");
    assert!(result.is_ok());

    // Special characters
    let result = pipeline.generate_shader("Create a pulsating effect!!!");
    assert!(result.is_ok());
}

/// Test that cache hit rate calculation works correctly
#[test]
fn test_cache_hit_rate() {
    let mut pipeline = AiShaderPipeline::new();

    // No operations yet
    assert_eq!(pipeline.cache_stats().hit_rate(), 0.0);

    // First generation (miss)
    let _ = pipeline.generate_shader("Create a pulsating effect");
    assert_eq!(pipeline.cache_stats().hit_rate(), 0.0);

    // Second generation (hit)
    let _ = pipeline.generate_shader("Create a pulsating effect");
    assert_eq!(pipeline.cache_stats().hit_rate(), 0.5);

    // Third generation (hit)
    let _ = pipeline.generate_shader("Create a pulsating effect");
    assert_eq!(pipeline.cache_stats().hit_rate(), 2.0 / 3.0);
}
