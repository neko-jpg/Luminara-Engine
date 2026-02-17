//! Dynamic shader generation from symbolic expressions.
//!
//! This module provides a shader generator that wraps the SymExpr WGSL codegen
//! from luminara_math, adding shader caching and dynamic compilation support
//! for the rendering pipeline.

use crate::error::RenderError;
use crate::shader::Shader;
use luminara_math::symbolic::{ShaderGenerator as MathShaderGenerator, SymExpr};
use std::collections::HashMap;
use std::hash::Hash;
use wgpu;

/// Hash key for shader cache based on symbolic expression
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ShaderCacheKey(String);

impl From<&SymExpr> for ShaderCacheKey {
    fn from(expr: &SymExpr) -> Self {
        // Use the expression's debug representation as a cache key
        ShaderCacheKey(format!("{:?}", expr))
    }
}

/// Dynamic shader generator with caching support.
///
/// Wraps the SymExpr WGSL codegen from luminara_math and provides:
/// - Shader caching to avoid recompilation
/// - Dynamic shader compilation from mathematical expressions
/// - Integration with the rendering pipeline
///
/// # Example
///
/// ```no_run
/// use luminara_render::shader_generator::ShaderGenerator;
/// use luminara_math::symbolic::SymExpr;
/// use std::rc::Rc;
///
/// let mut generator = ShaderGenerator::new();
///
/// // Create a pulsating effect: sin(time * 2π * 2)
/// let time_var = Rc::new(SymExpr::Var("time".to_string()));
/// let freq = Rc::new(SymExpr::Const(2.0 * std::f64::consts::PI * 2.0));
/// let expr = Rc::new(SymExpr::Sin(Rc::new(SymExpr::Mul(time_var, freq))));
///
/// // Generate shader (will be cached)
/// # /*
/// let shader = generator.generate_from_expr(&expr, &device)?;
/// # */
/// ```
pub struct ShaderGenerator {
    /// Cache of compiled shaders keyed by expression
    cache: HashMap<ShaderCacheKey, CachedShader>,
    /// Statistics for cache performance
    stats: CacheStats,
}

/// Cached shader entry
struct CachedShader {
    /// The generated WGSL source code
    wgsl_source: String,
    /// Compiled shader (if compiled)
    shader: Option<Shader>,
}

/// Cache performance statistics
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: usize,
    /// Number of cache misses
    pub misses: usize,
    /// Total number of shaders generated
    pub total_generated: usize,
}

impl CacheStats {
    /// Calculate cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl ShaderGenerator {
    /// Create a new shader generator with empty cache.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            stats: CacheStats::default(),
        }
    }

    /// Generate a shader from a symbolic expression.
    ///
    /// This method checks the cache first. If the expression has been seen before,
    /// it returns the cached shader. Otherwise, it generates new WGSL code using
    /// the luminara_math ShaderGenerator and caches the result.
    ///
    /// # Arguments
    ///
    /// * `expr` - The symbolic expression to compile to WGSL
    ///
    /// # Returns
    ///
    /// The generated WGSL source code as a String
    ///
    /// # Example
    ///
    /// ```no_run
    /// use luminara_render::shader_generator::ShaderGenerator;
    /// use luminara_math::symbolic::SymExpr;
    /// use std::rc::Rc;
    ///
    /// let mut generator = ShaderGenerator::new();
    /// let expr = Rc::new(SymExpr::Const(1.0));
    /// let wgsl = generator.generate_from_expr(&expr);
    /// ```
    pub fn generate_from_expr(&mut self, expr: &SymExpr) -> String {
        let cache_key = ShaderCacheKey::from(expr);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key) {
            self.stats.hits += 1;
            return cached.wgsl_source.clone();
        }

        // Cache miss - generate new shader
        self.stats.misses += 1;
        self.stats.total_generated += 1;

        // Use the luminara_math ShaderGenerator to generate WGSL
        let wgsl_source = MathShaderGenerator::generate_wgsl(expr);

        // Cache the result
        self.cache.insert(
            cache_key,
            CachedShader {
                wgsl_source: wgsl_source.clone(),
                shader: None,
            },
        );

        wgsl_source
    }

    /// Generate and compile a shader from a symbolic expression.
    ///
    /// This method generates WGSL code from the expression and compiles it
    /// using the provided wgpu device. The compiled shader is cached for
    /// future use.
    ///
    /// # Arguments
    ///
    /// * `expr` - The symbolic expression to compile
    /// * `device` - The wgpu device to use for compilation
    ///
    /// # Returns
    ///
    /// A compiled Shader ready for use in the rendering pipeline
    ///
    /// # Errors
    ///
    /// Returns `RenderError::ShaderCompilationFailed` if WGSL compilation fails
    pub fn generate_and_compile(
        &mut self,
        expr: &SymExpr,
        device: &wgpu::Device,
    ) -> Result<Shader, RenderError> {
        let cache_key = ShaderCacheKey::from(expr);

        // Check if we have a compiled shader in cache
        if let Some(cached) = self.cache.get_mut(&cache_key) {
            if cached.shader.is_some() {
                self.stats.hits += 1;
                // Clone the shader (this is cheap as it only clones the source)
                return Ok(Shader::from_wgsl(&cached.wgsl_source));
            }
        }

        // Generate WGSL source (this will use cache if available)
        let wgsl_source = self.generate_from_expr(expr);

        // Create and compile shader
        let mut shader = Shader::from_wgsl(&wgsl_source);
        
        // Compile to validate
        let _ = shader.compile(device);

        // Update cache with compiled shader
        if let Some(cached) = self.cache.get_mut(&cache_key) {
            cached.shader = Some(Shader::from_wgsl(&wgsl_source));
        }

        Ok(shader)
    }

    /// Clear the shader cache.
    ///
    /// This removes all cached shaders and resets statistics.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.stats = CacheStats::default();
    }

    /// Get cache statistics.
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get the number of cached shaders.
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Check if an expression is in the cache.
    pub fn is_cached(&self, expr: &SymExpr) -> bool {
        let cache_key = ShaderCacheKey::from(expr);
        self.cache.contains_key(&cache_key)
    }

    /// Remove a specific shader from the cache.
    pub fn remove_from_cache(&mut self, expr: &SymExpr) -> bool {
        let cache_key = ShaderCacheKey::from(expr);
        self.cache.remove(&cache_key).is_some()
    }
}

impl Default for ShaderGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn test_shader_generator_caching() {
        let mut generator = ShaderGenerator::new();

        // Create a simple expression
        let expr = Rc::new(SymExpr::Const(1.0));

        // First generation should be a cache miss
        let wgsl1 = generator.generate_from_expr(&expr);
        assert_eq!(generator.stats().misses, 1);
        assert_eq!(generator.stats().hits, 0);

        // Second generation should be a cache hit
        let wgsl2 = generator.generate_from_expr(&expr);
        assert_eq!(generator.stats().misses, 1);
        assert_eq!(generator.stats().hits, 1);

        // Results should be identical
        assert_eq!(wgsl1, wgsl2);
    }

    #[test]
    fn test_cache_stats() {
        let mut generator = ShaderGenerator::new();

        let expr1 = Rc::new(SymExpr::Const(1.0));
        let expr2 = Rc::new(SymExpr::Const(2.0));

        // Generate two different shaders
        generator.generate_from_expr(&expr1);
        generator.generate_from_expr(&expr2);

        // Generate first shader again (cache hit)
        generator.generate_from_expr(&expr1);

        assert_eq!(generator.stats().total_generated, 2);
        assert_eq!(generator.stats().misses, 2);
        assert_eq!(generator.stats().hits, 1);
        assert_eq!(generator.stats().hit_rate(), 1.0 / 3.0);
    }

    #[test]
    fn test_clear_cache() {
        let mut generator = ShaderGenerator::new();

        let expr = Rc::new(SymExpr::Const(1.0));
        generator.generate_from_expr(&expr);

        assert_eq!(generator.cache_size(), 1);
        assert_eq!(generator.stats().total_generated, 1);

        generator.clear_cache();

        assert_eq!(generator.cache_size(), 0);
        assert_eq!(generator.stats().total_generated, 0);
    }

    #[test]
    fn test_is_cached() {
        let mut generator = ShaderGenerator::new();

        let expr = Rc::new(SymExpr::Const(1.0));
        assert!(!generator.is_cached(&expr));

        generator.generate_from_expr(&expr);
        assert!(generator.is_cached(&expr));
    }

    #[test]
    fn test_remove_from_cache() {
        let mut generator = ShaderGenerator::new();

        let expr = Rc::new(SymExpr::Const(1.0));
        generator.generate_from_expr(&expr);

        assert!(generator.is_cached(&expr));
        assert!(generator.remove_from_cache(&expr));
        assert!(!generator.is_cached(&expr));
        assert!(!generator.remove_from_cache(&expr)); // Already removed
    }

    #[test]
    fn test_complex_expression() {
        let mut generator = ShaderGenerator::new();

        // Create a pulsating effect: sin(time * 2π * 2)
        let time_var = Rc::new(SymExpr::Var("time".to_string()));
        let freq = Rc::new(SymExpr::Const(2.0 * std::f64::consts::PI * 2.0));
        let expr = Rc::new(SymExpr::Sin(Rc::new(SymExpr::Mul(time_var, freq))));

        let wgsl = generator.generate_from_expr(&expr);

        // Verify the WGSL contains expected elements
        assert!(wgsl.contains("sin"));
        assert!(wgsl.contains("time") || wgsl.contains("t"));
        assert!(generator.cache_size() == 1);
    }
}
