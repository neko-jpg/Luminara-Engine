//! Motor-based transform synchronization systems.
//!
//! This module provides ECS systems to synchronize `TransformMotor` components
//! with standard `Transform` components, enabling gimbal-lock-free rotations
//! while maintaining compatibility with the existing transform hierarchy system.
//!
//! # Systems
//! - `sync_motor_to_transform_system`: Syncs TransformMotor → Transform
//! - `sync_transform_to_motor_system`: Syncs Transform → TransformMotor
//! - `motor_transform_propagate_system`: Hierarchical transform propagation for motors
//!
//! # Usage
//! Add `TransformMotor` to entities that need gimbal-lock-free rotations.
//! The sync systems will automatically keep Transform and TransformMotor in sync.

use luminara_core::{Entity, World};
use luminara_math::{Transform, TransformMotor};
use std::collections::VecDeque;

use crate::hierarchy::{Children, GlobalTransform, Parent};

/// Marker component to indicate that TransformMotor is the authoritative source.
///
/// When this component is present, the sync system will copy from TransformMotor
/// to Transform. Without this marker, Transform is considered authoritative.
#[derive(Debug, Clone, Copy, Default)]
pub struct MotorDriven;

impl luminara_core::Component for MotorDriven {
    fn type_name() -> &'static str {
        "MotorDriven"
    }
}

/// Synchronize TransformMotor to Transform for motor-driven entities.
///
/// This system runs for entities with both TransformMotor and Transform components,
/// and the MotorDriven marker. It copies the transform data from TransformMotor
/// to Transform, ensuring the standard transform hierarchy system can work correctly.
///
/// # Performance
/// Uses SIMD operations through the Motor implementation for efficient conversion.
///
/// Requirements: 13.1
pub fn sync_motor_to_transform_system(world: &mut World) {
    let entities = world.entities();
    
    // Find all motor-driven entities
    let motor_driven: Vec<Entity> = entities
        .into_iter()
        .filter(|&e| {
            world.get_component::<TransformMotor>(e).is_some()
                && world.get_component::<Transform>(e).is_some()
                && world.get_component::<MotorDriven>(e).is_some()
        })
        .collect();

    // Sync TransformMotor → Transform
    for entity in motor_driven {
        if let Some(motor_transform) = world.get_component::<TransformMotor>(entity).cloned() {
            let standard_transform = motor_transform.to_transform();
            let _ = world.add_component(entity, standard_transform);
        }
    }
}

/// Synchronize Transform to TransformMotor for transform-driven entities.
///
/// This system runs for entities with both Transform and TransformMotor components,
/// but WITHOUT the MotorDriven marker. It copies the transform data from Transform
/// to TransformMotor, allowing standard transform manipulation while maintaining
/// a motor representation.
///
/// Requirements: 13.1
pub fn sync_transform_to_motor_system(world: &mut World) {
    let entities = world.entities();
    
    // Find all transform-driven entities (have both components but no MotorDriven marker)
    let transform_driven: Vec<Entity> = entities
        .into_iter()
        .filter(|&e| {
            world.get_component::<Transform>(e).is_some()
                && world.get_component::<TransformMotor>(e).is_some()
                && world.get_component::<MotorDriven>(e).is_none()
        })
        .collect();

    // Sync Transform → TransformMotor
    for entity in transform_driven {
        if let Some(transform) = world.get_component::<Transform>(entity).cloned() {
            let motor_transform = TransformMotor::from_transform(&transform);
            let _ = world.add_component(entity, motor_transform);
        }
    }
}

/// Global transform using Motor representation.
///
/// This component stores the world-space transform as a Motor, providing
/// gimbal-lock-free hierarchical transforms.
#[derive(Debug, Clone, Copy)]
#[repr(C, align(32))]
pub struct GlobalTransformMotor(pub TransformMotor);

impl Default for GlobalTransformMotor {
    fn default() -> Self {
        Self(TransformMotor::IDENTITY)
    }
}

impl luminara_core::Component for GlobalTransformMotor {
    fn type_name() -> &'static str {
        "GlobalTransformMotor"
    }
}

/// Hierarchical transform propagation for motor-based transforms.
///
/// This system is similar to `transform_propagate_system` but works with
/// `TransformMotor` components. It uses motor composition (geometric product)
/// for combining parent and child transforms, which is more efficient and
/// numerically stable than matrix multiplication.
///
/// # Algorithm
/// 1. Find all root entities (TransformMotor but no Parent)
/// 2. For each root, perform breadth-first traversal
/// 3. Compose parent and child motors using geometric product
/// 4. Store result in GlobalTransformMotor
///
/// # Performance
/// Motor composition is optimized with SIMD operations and avoids the
/// numerical issues of repeated matrix multiplications in deep hierarchies.
///
/// Requirements: 13.1, 13.7
pub fn motor_transform_propagate_system(world: &mut World) {
    let entities = world.entities();
    
    // Find all root entities (entities with TransformMotor but no Parent)
    let roots: Vec<Entity> = entities
        .into_iter()
        .filter(|&e| {
            world.get_component::<TransformMotor>(e).is_some()
                && world.get_component::<Parent>(e).is_none()
        })
        .collect();

    // Process each root hierarchy using breadth-first traversal
    for root in roots {
        let root_motor = *world.get_component::<TransformMotor>(root).unwrap();
        let _ = world.add_component(root, GlobalTransformMotor(root_motor));

        // Queue for breadth-first traversal: (entity, parent_global_motor)
        let mut queue = VecDeque::new();

        // Add root's children to the queue
        if let Some(children) = world.get_component::<Children>(root) {
            for &child in &children.0 {
                queue.push_back((child, root_motor));
            }
        }

        // Process queue in breadth-first order
        while let Some((entity, parent_motor)) = queue.pop_front() {
            if let Some(local_motor) = world.get_component::<TransformMotor>(entity).cloned() {
                // Compose transforms: parent_world ∘ child_local
                // Using motor geometric product for efficient composition
                let global_motor = parent_motor.compose(&local_motor);

                let _ = world.add_component(entity, GlobalTransformMotor(global_motor));

                // Add this entity's children to the queue
                if let Some(children) = world.get_component::<Children>(entity) {
                    for &child in &children.0 {
                        queue.push_back((child, global_motor));
                    }
                }
            }
        }
    }
}

/// Synchronize GlobalTransformMotor to GlobalTransform.
///
/// This system ensures that entities with GlobalTransformMotor also have
/// a corresponding GlobalTransform, maintaining compatibility with systems
/// that expect standard GlobalTransform components.
///
/// Requirements: 13.1
pub fn sync_global_motor_to_transform_system(world: &mut World) {
    let entities = world.entities();
    
    let motor_entities: Vec<Entity> = entities
        .into_iter()
        .filter(|&e| world.get_component::<GlobalTransformMotor>(e).is_some())
        .collect();

    for entity in motor_entities {
        if let Some(global_motor) = world.get_component::<GlobalTransformMotor>(entity).cloned() {
            let global_transform = global_motor.0.to_transform();
            let _ = world.add_component(entity, GlobalTransform(global_transform));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luminara_core::Component;
    use luminara_math::{Quat, Vec3};

    #[test]
    fn test_motor_driven_marker() {
        // Test that MotorDriven marker component can be created
        let _marker = MotorDriven;
        assert_eq!(MotorDriven::type_name(), "MotorDriven");
    }

    #[test]
    fn test_global_transform_motor_creation() {
        // Test that GlobalTransformMotor can be created
        let motor = TransformMotor::from_position_rotation(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from_rotation_y(std::f32::consts::PI / 4.0),
        );
        let global = GlobalTransformMotor(motor);
        
        let (_, translation) = global.0.to_rotation_translation();
        assert!((translation - Vec3::new(1.0, 2.0, 3.0)).length() < 1e-5);
    }

    #[test]
    fn test_motor_composition_logic() {
        // Test motor composition without ECS
        let parent_motor = TransformMotor::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let child_motor = TransformMotor::from_translation(Vec3::new(0.0, 5.0, 0.0));
        
        let global_motor = parent_motor.compose(&child_motor);
        let (_, translation) = global_motor.to_rotation_translation();
        
        // Should be parent + child = (10, 5, 0)
        assert!((translation - Vec3::new(10.0, 5.0, 0.0)).length() < 1e-5);
    }

    #[test]
    fn test_motor_hierarchy_with_rotation_logic() {
        // Test motor composition with rotation without ECS
        // In a hierarchy, we want: global = parent * child_local
        // This means the child's local transform is applied in the parent's space
        
        let parent_motor = TransformMotor::from_position_rotation(
            Vec3::ZERO,
            Quat::from_rotation_y(std::f32::consts::PI / 2.0), // 90 degrees around Y
        );
        
        // Apply parent rotation to child's local position
        let child_world_pos = parent_motor.transform_point(Vec3::new(10.0, 0.0, 0.0));
        
        // After 90-degree Y rotation, +X becomes -Z
        assert!((child_world_pos - Vec3::new(0.0, 0.0, -10.0)).length() < 0.1);
    }

    #[test]
    fn test_deep_hierarchy_logic() {
        // Test deep hierarchy composition without ECS
        let t1 = TransformMotor::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let t2 = TransformMotor::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let t3 = TransformMotor::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let t4 = TransformMotor::from_translation(Vec3::new(1.0, 0.0, 0.0));
        
        let result = t1.compose(&t2).compose(&t3).compose(&t4);
        let (_, translation) = result.to_rotation_translation();
        
        // Should be 4.0 along X (1 + 1 + 1 + 1)
        assert!((translation - Vec3::new(4.0, 0.0, 0.0)).length() < 1e-4);
    }

    #[test]
    fn test_motor_to_transform_conversion() {
        // Test conversion logic
        let motor = TransformMotor::from_position_rotation(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from_rotation_y(std::f32::consts::PI / 4.0),
        );
        
        let transform = motor.to_transform();
        assert!((transform.translation - Vec3::new(1.0, 2.0, 3.0)).length() < 1e-5);
    }
}
