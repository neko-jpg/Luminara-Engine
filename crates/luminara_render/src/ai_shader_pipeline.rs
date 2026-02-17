//! AI-driven shader generation pipeline.
//!
//! This module implements the complete pipeline for AI-driven shader generation:
//! 1. AI generates mathematical expression from natural language description
//! 2. Expression is converted to WGSL using ShaderGenerator
//! 3. Shader is compiled and validated
//! 4. Shader is applied to a Material
//!
//! This satisfies Requirement 13.4: "WHEN generating shaders, THE System SHALL use
//! SymExpr to enable AI-driven shader generation from mathematical expressions"

use crate::error::RenderError;
use crate::material::Material;
use crate::shader::Shader;
use crate::shader_generator::ShaderGenerator;
use luminara_math::symbolic::SymExpr;
use std::rc::Rc;
use std::sync::Arc;

/// Result type for AI shader generation operations
pub type AiShaderResult<T> = Result<T, AiShaderError>;

/// Errors that can occur during AI shader generation
#[derive(Debug, Clone)]
pub enum AiShaderError {
    /// Failed to generate expression from description
    ExpressionGenerationFailed(String),
    /// Failed to compile shader
    ShaderCompilationFailed(String),
    /// Invalid shader description
    InvalidDescription(String),
    /// Rendering error
    RenderError(String),
}

impl From<RenderError> for AiShaderError {
    fn from(err: RenderError) -> Self {
        AiShaderError::RenderError(format!("{:?}", err))
    }
}

impl std::fmt::Display for AiShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiShaderError::ExpressionGenerationFailed(msg) => {
                write!(f, "Expression generation failed: {}", msg)
            }
            AiShaderError::ShaderCompilationFailed(msg) => {
                write!(f, "Shader compilation failed: {}", msg)
            }
            AiShaderError::InvalidDescription(msg) => {
                write!(f, "Invalid description: {}", msg)
            }
            AiShaderError::RenderError(msg) => write!(f, "Render error: {}", msg),
        }
    }
}

impl std::error::Error for AiShaderError {}

/// AI shader generation pipeline.
///
/// This pipeline connects AI expression generation to shader compilation and
/// material application. It provides the complete workflow for AI-driven shader
/// generation as specified in Requirement 13.4.
///
/// # Architecture
///
/// ```text
/// Natural Language Description
///          ↓
///    AI Expression Generator
///          ↓
///      SymExpr (Mathematical Expression)
///          ↓
///    ShaderGenerator (WGSL Codegen)
///          ↓
///      WGSL Shader Code
///          ↓
///    Shader Compilation
///          ↓
///      Compiled Shader
///          ↓
///    Material Application
/// ```
///
/// # Example
///
/// ```no_run
/// use luminara_render::ai_shader_pipeline::AiShaderPipeline;
///
/// let mut pipeline = AiShaderPipeline::new();
///
/// // Generate shader from natural language description
/// let description = "Create a pulsating glow effect that pulses at 2Hz";
/// # /*
/// let material = pipeline.generate_material(description, "pulsating_glow", &device)?;
/// # */
/// ```
pub struct AiShaderPipeline {
    /// Shader generator for WGSL codegen
    shader_generator: ShaderGenerator,
    /// Expression generator (placeholder for AI integration)
    expression_generator: Box<dyn ExpressionGenerator>,
}

/// Trait for AI expression generation.
///
/// This trait abstracts the AI/LLM integration, allowing different implementations:
/// - Mock generator for testing
/// - Local LLM integration
/// - Remote API integration (OpenAI, Anthropic, etc.)
/// - Custom fine-tuned models
pub trait ExpressionGenerator: Send + Sync {
    /// Generate a mathematical expression from a natural language description.
    ///
    /// # Arguments
    ///
    /// * `description` - Natural language description of the desired shader effect
    ///
    /// # Returns
    ///
    /// A SymExpr representing the mathematical expression for the shader
    fn generate_expression(&self, description: &str) -> AiShaderResult<Rc<SymExpr>>;

    /// Get the name/identifier of this generator
    fn name(&self) -> &str;
}

/// Mock expression generator for testing and demonstration.
///
/// This generator recognizes common shader patterns and generates appropriate
/// expressions. It serves as a placeholder until full LLM integration is available.
pub struct MockExpressionGenerator;

impl ExpressionGenerator for MockExpressionGenerator {
    fn generate_expression(&self, description: &str) -> AiShaderResult<Rc<SymExpr>> {
        let desc_lower = description.to_lowercase();

        // Pattern matching for common shader effects
        if desc_lower.contains("pulsating") || desc_lower.contains("pulse") {
            // Extract frequency if mentioned (default to 2Hz)
            let freq = if desc_lower.contains("2hz") || desc_lower.contains("2 hz") {
                2.0
            } else if desc_lower.contains("1hz") || desc_lower.contains("1 hz") {
                1.0
            } else {
                2.0 // default
            };

            // Generate: sin(time * 2π * freq)
            let time_var = Rc::new(SymExpr::Var("time".to_string()));
            let freq_const = Rc::new(SymExpr::Const(2.0 * std::f64::consts::PI * freq));
            let expr = Rc::new(SymExpr::Sin(Rc::new(SymExpr::Mul(time_var, freq_const))));
            Ok(expr)
        } else if desc_lower.contains("gradient") || desc_lower.contains("fade") {
            // Generate: uv.x (horizontal gradient)
            Ok(Rc::new(SymExpr::Var("uv_x".to_string())))
        } else if desc_lower.contains("noise") || desc_lower.contains("random") {
            // Generate: sin(uv.x * 10) * sin(uv.y * 10) (pseudo-noise pattern)
            let uv_x = Rc::new(SymExpr::Var("uv_x".to_string()));
            let uv_y = Rc::new(SymExpr::Var("uv_y".to_string()));
            let ten = Rc::new(SymExpr::Const(10.0));
            let sin_x = Rc::new(SymExpr::Sin(Rc::new(SymExpr::Mul(uv_x, ten.clone()))));
            let sin_y = Rc::new(SymExpr::Sin(Rc::new(SymExpr::Mul(uv_y, ten))));
            Ok(Rc::new(SymExpr::Mul(sin_x, sin_y)))
        } else if desc_lower.contains("constant") || desc_lower.contains("solid") {
            // Generate: 1.0 (constant value)
            Ok(Rc::new(SymExpr::Const(1.0)))
        } else {
            Err(AiShaderError::InvalidDescription(format!(
                "Could not generate expression for description: {}. \
                 Supported patterns: pulsating, gradient, noise, constant",
                description
            )))
        }
    }

    fn name(&self) -> &str {
        "MockExpressionGenerator"
    }
}

impl AiShaderPipeline {
    /// Create a new AI shader pipeline with the default mock generator.
    pub fn new() -> Self {
        Self {
            shader_generator: ShaderGenerator::new(),
            expression_generator: Box::new(MockExpressionGenerator),
        }
    }

    /// Create a new AI shader pipeline with a custom expression generator.
    ///
    /// Use this to integrate with actual LLM services.
    pub fn with_generator(generator: Box<dyn ExpressionGenerator>) -> Self {
        Self {
            shader_generator: ShaderGenerator::new(),
            expression_generator: generator,
        }
    }

    /// Generate a shader from a natural language description.
    ///
    /// This is the main entry point for the AI shader generation pipeline.
    /// It performs the complete workflow:
    /// 1. AI generates mathematical expression
    /// 2. Convert expression to WGSL
    /// 3. Return WGSL source code
    ///
    /// # Arguments
    ///
    /// * `description` - Natural language description of the desired shader effect
    ///
    /// # Returns
    ///
    /// The generated WGSL shader source code
    ///
    /// # Example
    ///
    /// ```
    /// use luminara_render::ai_shader_pipeline::AiShaderPipeline;
    ///
    /// let mut pipeline = AiShaderPipeline::new();
    /// let wgsl = pipeline.generate_shader("Create a pulsating glow effect").unwrap();
    /// assert!(wgsl.contains("sin"));
    /// ```
    pub fn generate_shader(&mut self, description: &str) -> AiShaderResult<String> {
        // Step 1: AI generates mathematical expression
        let expr = self.expression_generator.generate_expression(description)?;

        // Step 2: Convert expression to WGSL
        let wgsl_source = self.shader_generator.generate_from_expr(&expr);

        Ok(wgsl_source)
    }

    /// Generate and compile a shader from a natural language description.
    ///
    /// This extends `generate_shader` by also compiling the shader with wgpu.
    ///
    /// # Arguments
    ///
    /// * `description` - Natural language description of the desired shader effect
    /// * `device` - The wgpu device to use for compilation
    ///
    /// # Returns
    ///
    /// A compiled Shader ready for use
    pub fn generate_and_compile_shader(
        &mut self,
        description: &str,
        device: &wgpu::Device,
    ) -> AiShaderResult<Shader> {
        // Step 1: AI generates mathematical expression
        let expr = self.expression_generator.generate_expression(description)?;

        // Step 2: Convert expression to WGSL and compile
        let shader = self
            .shader_generator
            .generate_and_compile(&expr, device)
            .map_err(|e| AiShaderError::ShaderCompilationFailed(format!("{:?}", e)))?;

        Ok(shader)
    }

    /// Generate a complete material from a natural language description.
    ///
    /// This is the highest-level API that performs the complete pipeline:
    /// 1. AI generates mathematical expression
    /// 2. Convert expression to WGSL
    /// 3. Compile shader
    /// 4. Create and return Material with the shader
    ///
    /// # Arguments
    ///
    /// * `description` - Natural language description of the desired shader effect
    /// * `material_name` - Name for the generated material
    /// * `device` - The wgpu device to use for compilation
    ///
    /// # Returns
    ///
    /// A Material with the AI-generated shader applied
    ///
    /// # Example
    ///
    /// ```no_run
    /// use luminara_render::ai_shader_pipeline::AiShaderPipeline;
    ///
    /// let mut pipeline = AiShaderPipeline::new();
    /// # /*
    /// let material = pipeline.generate_material(
    ///     "Create a pulsating glow effect that pulses at 2Hz",
    ///     "pulsating_glow",
    ///     &device
    /// ).unwrap();
    /// # */
    /// ```
    pub fn generate_material(
        &mut self,
        description: &str,
        material_name: &str,
        device: &wgpu::Device,
    ) -> AiShaderResult<Material> {
        // Generate and compile shader
        let shader = self.generate_and_compile_shader(description, device)?;

        // Create material with the shader
        let material = Material::new(material_name, Arc::new(shader));

        Ok(material)
    }

    /// Get statistics about the shader generator cache.
    pub fn cache_stats(&self) -> &crate::shader_generator::CacheStats {
        self.shader_generator.stats()
    }

    /// Clear the shader cache.
    pub fn clear_cache(&mut self) {
        self.shader_generator.clear_cache();
    }

    /// Get the name of the current expression generator.
    pub fn generator_name(&self) -> &str {
        self.expression_generator.name()
    }

    /// Get the size of the shader cache.
    pub fn cache_size(&self) -> usize {
        self.shader_generator.cache_size()
    }
}

impl Default for AiShaderPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_generator_pulsating() {
        let generator = MockExpressionGenerator;
        let expr = generator
            .generate_expression("Create a pulsating glow effect that pulses at 2Hz")
            .unwrap();

        // Should generate a sin expression
        match expr.as_ref() {
            SymExpr::Sin(_) => {} // Expected
            _ => panic!("Expected Sin expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_mock_generator_gradient() {
        let generator = MockExpressionGenerator;
        let expr = generator
            .generate_expression("Create a horizontal gradient")
            .unwrap();

        // Should generate a variable expression
        match expr.as_ref() {
            SymExpr::Var(name) => assert_eq!(name, "uv_x"),
            _ => panic!("Expected Var expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_mock_generator_noise() {
        let generator = MockExpressionGenerator;
        let expr = generator
            .generate_expression("Create a noise pattern")
            .unwrap();

        // Should generate a multiplication of sin expressions
        match expr.as_ref() {
            SymExpr::Mul(_, _) => {} // Expected
            _ => panic!("Expected Mul expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_mock_generator_invalid() {
        let generator = MockExpressionGenerator;
        let result = generator.generate_expression("Something completely unsupported");

        assert!(result.is_err());
        match result {
            Err(AiShaderError::InvalidDescription(_)) => {} // Expected
            _ => panic!("Expected InvalidDescription error"),
        }
    }

    #[test]
    fn test_pipeline_generate_shader() {
        let mut pipeline = AiShaderPipeline::new();
        let wgsl = pipeline
            .generate_shader("Create a pulsating glow effect")
            .unwrap();

        // Should contain sin function
        assert!(wgsl.contains("sin"));
    }

    #[test]
    fn test_pipeline_cache_stats() {
        let mut pipeline = AiShaderPipeline::new();

        // Generate same shader twice
        let _ = pipeline.generate_shader("Create a pulsating glow effect");
        let _ = pipeline.generate_shader("Create a pulsating glow effect");

        let stats = pipeline.cache_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_pipeline_clear_cache() {
        let mut pipeline = AiShaderPipeline::new();

        let _ = pipeline.generate_shader("Create a pulsating glow effect");
        assert_eq!(pipeline.shader_generator.cache_size(), 1);

        pipeline.clear_cache();
        assert_eq!(pipeline.shader_generator.cache_size(), 0);
    }

    #[test]
    fn test_custom_generator() {
        struct CustomGenerator;

        impl ExpressionGenerator for CustomGenerator {
            fn generate_expression(&self, _description: &str) -> AiShaderResult<Rc<SymExpr>> {
                Ok(Rc::new(SymExpr::Const(42.0)))
            }

            fn name(&self) -> &str {
                "CustomGenerator"
            }
        }

        let mut pipeline = AiShaderPipeline::with_generator(Box::new(CustomGenerator));
        assert_eq!(pipeline.generator_name(), "CustomGenerator");

        let wgsl = pipeline.generate_shader("anything").unwrap();
        assert!(wgsl.contains("42"));
    }
}
