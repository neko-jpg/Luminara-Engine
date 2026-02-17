use luminara_core::Component;
use luminara_reflect_derive::Reflect;
use serde::{Deserialize, Serialize};

/// Integration method for physics simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum IntegrationMethod {
    /// Standard Euler integration (default, compatible with Rapier)
    /// - Fast and simple
    /// - First-order accuracy
    /// - Good for most game physics scenarios
    Euler,
    
    /// Lie group RK4 integration (advanced, more accurate)
    /// - 2-3x more accurate than Euler
    /// - Better stability with high angular velocities
    /// - 3-4x slower than Euler
    /// - Recommended for precision-critical scenarios
    Rk4,
}

impl Default for IntegrationMethod {
    fn default() -> Self {
        // Default to Euler for backward compatibility
        IntegrationMethod::Euler
    }
}

/// Global physics integration configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PhysicsIntegrationConfig {
    /// Default integration method for all bodies
    pub default_method: IntegrationMethod,
}

impl Default for PhysicsIntegrationConfig {
    fn default() -> Self {
        Self {
            default_method: IntegrationMethod::Euler,
        }
    }
}

/// Per-body integration method override
///
/// Add this component to a specific entity to override the global
/// integration method for that body only.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
pub struct IntegrationMethodOverride {
    pub method: IntegrationMethod,
}

impl Component for IntegrationMethodOverride {
    fn type_name() -> &'static str {
        "IntegrationMethodOverride"
    }
}

impl IntegrationMethodOverride {
    /// Create a new override to use Euler integration
    pub fn euler() -> Self {
        Self {
            method: IntegrationMethod::Euler,
        }
    }
    
    /// Create a new override to use RK4 integration
    pub fn rk4() -> Self {
        Self {
            method: IntegrationMethod::Rk4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_integration_method() {
        let method = IntegrationMethod::default();
        assert_eq!(method, IntegrationMethod::Euler);
    }

    #[test]
    fn test_default_config() {
        let config = PhysicsIntegrationConfig::default();
        assert_eq!(config.default_method, IntegrationMethod::Euler);
    }

    #[test]
    fn test_override_creation() {
        let euler_override = IntegrationMethodOverride::euler();
        assert_eq!(euler_override.method, IntegrationMethod::Euler);

        let rk4_override = IntegrationMethodOverride::rk4();
        assert_eq!(rk4_override.method, IntegrationMethod::Rk4);
    }
}
