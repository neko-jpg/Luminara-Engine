use luminara_math::algebra::{Bivector, LieGroupIntegrator, Motor};
use luminara_math::{Transform, Vec3};

/// Physics integrator using Lie group methods for improved stability
///
/// This integrator wraps the luminara_math Lie group integrator and provides
/// integration with the physics pipeline. It uses the Munthe-Kaas RK4 method
/// for structure-preserving integration on SE(3), which is more stable than
/// standard Euler integration, especially for high-speed rotations.
pub struct LiePhysicsIntegrator;

impl LiePhysicsIntegrator {
    /// Integrate rigid body state using RK4 on Lie groups
    ///
    /// This method uses the Munthe-Kaas RK4 integrator from luminara_math,
    /// which preserves the Lie group structure and provides better stability
    /// than standard Euler integration.
    ///
    /// # Arguments
    /// * `motor` - Current orientation/position as a Motor
    /// * `velocity` - Velocity bivector (angular + linear velocity in body frame)
    /// * `dt` - Time step
    ///
    /// # Returns
    /// The integrated motor representing the new state
    pub fn integrate_rk4(motor: &Motor<f32>, velocity: &Bivector<f32>, dt: f32) -> Motor<f32> {
        // Use the Munthe-Kaas RK4 integrator from luminara_math
        // The velocity function is constant over this timestep
        LieGroupIntegrator::step(*motor, dt, |_| *velocity)
    }

    /// Simple Euler integration for comparison/fallback
    ///
    /// This is a first-order integration method that's faster but less stable
    /// than RK4. Useful for comparison or when performance is critical.
    ///
    /// # Arguments
    /// * `motor` - Current orientation/position as a Motor
    /// * `velocity` - Velocity bivector (angular + linear velocity)
    /// * `dt` - Time step
    pub fn integrate_euler(motor: &Motor<f32>, velocity: &Bivector<f32>, dt: f32) -> Motor<f32> {
        // Simple Euler step: M_new = M * exp(v * dt)
        let step = velocity.scale(dt);
        let delta = Motor::exp(&step);
        motor.geometric_product(&delta)
    }

    /// Integrate rigid body state and update in-place
    ///
    /// This is the main integration method that should be used by the physics
    /// pipeline. It uses RK4 integration and normalizes the result to prevent
    /// numerical drift.
    ///
    /// # Arguments
    /// * `motor` - Current motor (will be updated in-place)
    /// * `velocity` - Velocity bivector (angular + linear velocity)
    /// * `dt` - Time step
    pub fn step(motor: &mut Motor<f32>, velocity: &Bivector<f32>, dt: f32) {
        *motor = Self::integrate_rk4(motor, velocity, dt);
        motor.normalize(); // Prevent numerical drift
    }

    /// Convert Transform to Motor representation
    ///
    /// This allows integration with the existing ECS Transform component.
    ///
    /// # Arguments
    /// * `transform` - The Transform to convert
    ///
    /// # Returns
    /// A Motor representing the same transformation (ignoring scale)
    pub fn transform_to_motor(transform: &Transform) -> Motor<f32> {
        Motor::from_rotation_translation_glam(transform.rotation, transform.translation)
    }

    /// Convert Motor back to Transform
    ///
    /// This extracts position and rotation from a Motor and creates a Transform.
    /// The scale is preserved from the original transform.
    ///
    /// # Arguments
    /// * `motor` - The Motor to convert
    /// * `scale` - The scale to apply (Motors don't encode scale)
    ///
    /// # Returns
    /// A Transform with the motor's position and rotation
    pub fn motor_to_transform(motor: &Motor<f32>, scale: Vec3) -> Transform {
        let (rotation, translation) = motor.to_rotation_translation_glam();
        Transform {
            translation,
            rotation,
            scale,
        }
    }

    /// Integrate a rigid body's transform using Lie group methods
    ///
    /// This is a convenience method that handles the Transform <-> Motor conversion
    /// automatically. It's the recommended way to integrate transforms in the physics
    /// pipeline.
    ///
    /// # Arguments
    /// * `transform` - Current transform (will be updated in-place)
    /// * `linear_velocity` - Linear velocity in world space
    /// * `angular_velocity` - Angular velocity in body space (axis-angle representation)
    /// * `dt` - Time step
    pub fn integrate_transform(
        transform: &mut Transform,
        linear_velocity: Vec3,
        angular_velocity: Vec3,
        dt: f32,
    ) {
        // Convert to Motor
        let mut motor = Self::transform_to_motor(transform);

        // Create velocity bivector
        // In PGA, bivectors encode both rotation and translation
        // e23, e13, e12 are rotational basis elements (dual to x, y, z axes)
        // e01, e02, e03 are translational basis elements
        let velocity = Bivector::new(
            angular_velocity.x,
            angular_velocity.y,
            angular_velocity.z,
            linear_velocity.x,
            linear_velocity.y,
            linear_velocity.z,
        );

        // Integrate using RK4
        Self::step(&mut motor, &velocity, dt);

        // Convert back to Transform, preserving scale
        *transform = Self::motor_to_transform(&motor, transform.scale);
    }

    /// Compute energy of a rigid body state
    ///
    /// This is useful for verifying energy conservation in physics simulations.
    ///
    /// # Arguments
    /// * `velocity` - Velocity bivector
    /// * `mass` - Mass of the rigid body
    /// * `inertia` - Moment of inertia (simplified as scalar)
    ///
    /// # Returns
    /// Total kinetic energy (translational + rotational)
    pub fn compute_energy(velocity: &Bivector<f32>, mass: f32, inertia: f32) -> f32 {
        // Translational kinetic energy: 0.5 * m * v^2
        let linear_vel = Vec3::new(velocity.e01, velocity.e02, velocity.e03);
        let translational_energy = 0.5 * mass * linear_vel.length_squared();

        // Rotational kinetic energy: 0.5 * I * ω^2
        let angular_vel = Vec3::new(velocity.e23, velocity.e13, velocity.e12);
        let rotational_energy = 0.5 * inertia * angular_vel.length_squared();

        translational_energy + rotational_energy
    }
}
