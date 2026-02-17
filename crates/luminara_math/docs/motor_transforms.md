# Motor-Based Transforms: A Practical Guide

**Requirements:** 13.6 - Provide documentation and examples for developers unfamiliar with PGA

## Table of Contents

1. [Introduction](#introduction)
2. [What is Projective Geometric Algebra (PGA)?](#what-is-projective-geometric-algebra-pga)
3. [Why Use Motors?](#why-use-motors)
4. [Getting Started](#getting-started)
5. [Core Concepts](#core-concepts)
6. [Usage Examples](#usage-examples)
7. [Performance Characteristics](#performance-characteristics)
8. [When to Use Motors vs Standard Transforms](#when-to-use-motors-vs-standard-transforms)
9. [Advanced Topics](#advanced-topics)
10. [Troubleshooting](#troubleshooting)

---

## Introduction

Luminara Engine provides two transform representations:
- **Standard Transform**: Uses quaternions for rotation, separate vectors for translation and scale
- **TransformMotor**: Uses PGA Motors for unified rotation and translation representation

This guide explains what Motors are, when to use them, and how to integrate them into your game.

---

## What is Projective Geometric Algebra (PGA)?

### The Problem with Traditional Transforms

Traditional 3D transforms use separate mathematical objects:
- **Rotation**: Quaternions (4 components: x, y, z, w)
- **Translation**: Vectors (3 components: x, y, z)
- **Scale**: Vectors (3 components: x, y, z)

This separation leads to:
- **Gimbal lock**: Euler angles can lose a degree of freedom at certain orientations
- **Numerical instability**: Repeated matrix multiplications accumulate error
- **Complex composition**: Combining transforms requires careful ordering and normalization

### The PGA Solution

Projective Geometric Algebra (PGA) provides a unified mathematical framework where:
- **Rotation and translation** are represented as a single algebraic object called a **Motor**
- **Composition** uses the geometric product (a single operation)
- **No gimbal lock**: Motors naturally avoid singularities
- **Better numerical stability**: The geometric product preserves structure

### What is a Motor?

A Motor is an 8-component object in PGA(3,0,1) that encodes both rotation and translation:

```
Motor = {
    s,              // Scalar (rotation angle)
    e12, e13, e23,  // Rotational bivector (rotation axis)
    e01, e02, e03,  // Translational bivector (translation)
    e0123           // Pseudoscalar (related to translation)
}
```

**Key insight**: A Motor is like a "screw motion" - it combines rotation around an axis with translation along that axis, generalizing to arbitrary rigid transformations.

---

## Why Use Motors?

### Advantages

✅ **Gimbal-Lock-Free**: Motors never lose degrees of freedom, making them ideal for:
- Spacecraft simulation
- Free-flying cameras
- Physics with high angular velocities
- Procedural animation

✅ **4x Faster Repeated Compositions**: When composing transforms repeatedly (physics integration, skeletal animation), Motors are significantly faster due to better numerical stability.

✅ **Unified Representation**: Rotation and translation in one structure simplifies:
- Interpolation (SLERP for rotation, LERP for translation in one operation)
- Composition (single geometric product)
- Inversion (single reverse operation)

✅ **Numerical Stability**: Motors maintain accuracy over many operations without normalization.

### Trade-offs

⚠️ **33% Larger Memory**: 64 bytes vs 48 bytes for standard Transform
⚠️ **Slower Interpolation**: 2.64x slower (can be optimized with native SLERP)
⚠️ **Learning Curve**: Requires understanding PGA concepts

---

## Getting Started

### Basic Usage

```rust
use luminara_math::{TransformMotor, Vec3, Quat};

// Create a Motor transform from position and rotation
let transform = TransformMotor::from_position_rotation(
    Vec3::new(1.0, 2.0, 3.0),  // position
    Quat::from_rotation_y(std::f32::consts::PI / 4.0)  // rotation
);

// Transform a point
let point = Vec3::new(0.0, 0.0, 1.0);
let transformed_point = transform.transform_point(point);

// Convert to standard Transform for compatibility
let standard_transform = transform.to_transform();
```

### ECS Integration

```rust
use luminara_core::World;
use luminara_math::TransformMotor;
use luminara_scene::{MotorDriven, sync_motor_to_transform_system};

// Create an entity with Motor-based transform
let entity = world.spawn();
world.add_component(entity, TransformMotor::from_translation(Vec3::new(5.0, 0.0, 0.0)));
world.add_component(entity, MotorDriven); // Mark as motor-driven

// The sync system will automatically update the standard Transform
sync_motor_to_transform_system(&mut world);
```

---

## Core Concepts

### 1. Identity Transform

The identity Motor represents no transformation:

```rust
let identity = TransformMotor::IDENTITY;
// Equivalent to: position = (0,0,0), rotation = identity, scale = (1,1,1)
```

### 2. Creating Motors

```rust
// From translation only
let t = TransformMotor::from_translation(Vec3::new(1.0, 2.0, 3.0));

// From rotation only
let r = TransformMotor::from_rotation(Quat::from_rotation_y(PI / 2.0));

// From position and rotation
let tr = TransformMotor::from_position_rotation(
    Vec3::new(1.0, 2.0, 3.0),
    Quat::from_rotation_y(PI / 2.0)
);

// From position, rotation, and scale
let trs = TransformMotor::from_position_rotation_scale(
    Vec3::new(1.0, 2.0, 3.0),
    Quat::from_rotation_y(PI / 2.0),
    Vec3::new(2.0, 2.0, 2.0)
);

// From standard Transform
let standard = Transform { /* ... */ };
let motor = TransformMotor::from_transform(&standard);
```

### 3. Composition

Combining two transforms uses the `compose` method:

```rust
let parent = TransformMotor::from_translation(Vec3::new(10.0, 0.0, 0.0));
let child = TransformMotor::from_rotation(Quat::from_rotation_y(PI / 2.0));

// Combine: apply parent, then child
let combined = parent.compose(&child);
```

**Important**: Composition order matters! `parent.compose(&child)` means "apply parent's transform, then child's transform in parent's space."

### 4. Interpolation

Smoothly interpolate between two transforms:

```rust
let start = TransformMotor::from_position_rotation(
    Vec3::ZERO,
    Quat::IDENTITY
);
let end = TransformMotor::from_position_rotation(
    Vec3::new(10.0, 0.0, 0.0),
    Quat::from_rotation_y(PI)
);

// Interpolate halfway (t = 0.5)
let mid = start.interpolate(&end, 0.5);
```

This performs:
- **SLERP** (Spherical Linear Interpolation) for rotation
- **LERP** (Linear Interpolation) for translation and scale

### 5. Inverse Transform

Compute the inverse (undo) of a transform:

```rust
let transform = TransformMotor::from_position_rotation(
    Vec3::new(1.0, 2.0, 3.0),
    Quat::from_rotation_y(PI / 4.0)
);

let inverse = transform.inverse();

// Composing with inverse gives identity
let identity = transform.compose(&inverse);
```

---

## Usage Examples

### Example 1: Gimbal-Lock-Free Camera

```rust
use luminara_math::{TransformMotor, Vec3, Quat};

struct FreeCamera {
    transform: TransformMotor,
    pitch: f32,
    yaw: f32,
}

impl FreeCamera {
    fn new(position: Vec3) -> Self {
        Self {
            transform: TransformMotor::from_translation(position),
            pitch: 0.0,
            yaw: 0.0,
        }
    }

    fn rotate(&mut self, delta_pitch: f32, delta_yaw: f32) {
        self.pitch += delta_pitch;
        self.yaw += delta_yaw;

        // Clamp pitch to avoid flipping
        self.pitch = self.pitch.clamp(-PI / 2.0 + 0.01, PI / 2.0 - 0.01);

        // Create rotation from pitch and yaw
        let rotation = Quat::from_rotation_y(self.yaw) 
            * Quat::from_rotation_x(self.pitch);

        // Update transform (no gimbal lock!)
        let position = self.transform.to_rotation_translation().1;
        self.transform = TransformMotor::from_position_rotation(position, rotation);
    }

    fn move_forward(&mut self, distance: f32) {
        let forward = self.transform.transform_vector(Vec3::new(0.0, 0.0, -1.0));
        let (rotation, position) = self.transform.to_rotation_translation();
        self.transform = TransformMotor::from_position_rotation(
            position + forward * distance,
            rotation
        );
    }
}
```

### Example 2: Skeletal Animation with Motors

```rust
use luminara_math::TransformMotor;

struct Bone {
    local_transform: TransformMotor,
    world_transform: TransformMotor,
    parent: Option<usize>,
}

struct Skeleton {
    bones: Vec<Bone>,
}

impl Skeleton {
    fn update_world_transforms(&mut self) {
        for i in 0..self.bones.len() {
            let world_transform = if let Some(parent_idx) = self.bones[i].parent {
                // Compose parent's world transform with this bone's local transform
                let parent_world = self.bones[parent_idx].world_transform;
                parent_world.compose(&self.bones[i].local_transform)
            } else {
                // Root bone: world = local
                self.bones[i].local_transform
            };
            
            self.bones[i].world_transform = world_transform;
        }
    }

    fn interpolate_pose(&mut self, target_pose: &[TransformMotor], t: f32) {
        for (bone, target) in self.bones.iter_mut().zip(target_pose.iter()) {
            bone.local_transform = bone.local_transform.interpolate(target, t);
        }
        self.update_world_transforms();
    }
}
```

### Example 3: Physics Integration with Motors

```rust
use luminara_math::{TransformMotor, Vec3, Quat};

struct RigidBody {
    transform: TransformMotor,
    velocity: Vec3,
    angular_velocity: Vec3,
}

impl RigidBody {
    fn integrate(&mut self, dt: f32) {
        // Update position
        let (rotation, position) = self.transform.to_rotation_translation();
        let new_position = position + self.velocity * dt;

        // Update rotation using angular velocity
        let angular_speed = self.angular_velocity.length();
        if angular_speed > 1e-6 {
            let axis = self.angular_velocity / angular_speed;
            let angle = angular_speed * dt;
            let delta_rotation = Quat::from_axis_angle(axis, angle);
            let new_rotation = delta_rotation * rotation;

            self.transform = TransformMotor::from_position_rotation(
                new_position,
                new_rotation
            );
        } else {
            self.transform = TransformMotor::from_position_rotation(
                new_position,
                rotation
            );
        }
    }
}
```

### Example 4: Hierarchical Transform System

```rust
use luminara_core::World;
use luminara_math::TransformMotor;
use luminara_scene::{Parent, Children, GlobalTransformMotor};

fn propagate_motor_transforms(world: &mut World) {
    // Find all root entities (no parent)
    let roots: Vec<_> = world.entities()
        .into_iter()
        .filter(|&e| {
            world.get_component::<TransformMotor>(e).is_some()
                && world.get_component::<Parent>(e).is_none()
        })
        .collect();

    // Process each hierarchy
    for root in roots {
        let root_motor = *world.get_component::<TransformMotor>(root).unwrap();
        world.add_component(root, GlobalTransformMotor(root_motor));

        // Recursively update children
        propagate_children(world, root, root_motor);
    }
}

fn propagate_children(world: &mut World, entity: Entity, parent_global: TransformMotor) {
    if let Some(children) = world.get_component::<Children>(entity) {
        for &child in &children.0 {
            if let Some(local) = world.get_component::<TransformMotor>(child).cloned() {
                // Compose parent's global with child's local
                let child_global = parent_global.compose(&local);
                world.add_component(child, GlobalTransformMotor(child_global));

                // Recurse to grandchildren
                propagate_children(world, child, child_global);
            }
        }
    }
}
```

---

## Performance Characteristics

Based on comprehensive benchmarks (see `motor_vs_quaternion_benchmark_results.md`):

### Memory Usage

| Transform Type | Size | Overhead |
|---------------|------|----------|
| Standard Transform | 48 bytes | baseline |
| TransformMotor | 64 bytes | +33% |

### Operation Performance

| Operation | Quaternion | Motor | Winner |
|-----------|-----------|-------|--------|
| **Single Composition** | 10.9ns | 13.0ns | Quaternion (19% faster) |
| **Repeated Composition (1000x)** | 40.75μs | 10.04μs | **Motor (4x faster!)** |
| **Interpolation** | 27.8ns | 73.5ns | Quaternion (2.6x faster) |
| **Point Transform** | 6.3ns | 6.9ns | Quaternion (9% faster) |
| **Batch (100 points)** | 188ns | 182ns | **Motor (3% faster)** |
| **Batch (1000 points)** | 1128ns | 1136ns | Tie (within 1%) |

### Key Findings

🚀 **Motors Excel At:**
- **Repeated compositions**: 4x faster (critical for physics and animation!)
- **Medium-sized batches**: Competitive or faster for 100-1000 points
- **Numerical stability**: No normalization needed, maintains accuracy

⚠️ **Motors Are Slower At:**
- **Single compositions**: 19% slower (but still only 13ns - negligible)
- **Interpolation**: 2.6x slower (optimization opportunity)
- **Small batches**: 45% slower for 10 points

### Performance Recommendations

**Use Motors when:**
- Composing transforms repeatedly (physics integration, skeletal animation)
- Numerical stability is critical (deep hierarchies, long simulations)
- Gimbal-lock-free rotation is required (free cameras, spacecraft)
- Processing medium-sized batches (100-1000 points)

**Use Standard Transforms when:**
- Performing frequent interpolation (keyframe animation)
- Processing small batches (<10 points)
- Memory is constrained (mobile, embedded)
- Simple one-time transformations

---

## When to Use Motors vs Standard Transforms

### Use TransformMotor For:

#### 1. Physics Simulations ⭐⭐⭐⭐⭐
**Why**: 4x faster repeated compositions, better numerical stability
```rust
// Physics integration requires many transform updates per frame
struct PhysicsBody {
    transform: TransformMotor,  // ✅ Perfect use case
    velocity: Vec3,
    angular_velocity: Vec3,
}
```

#### 2. Skeletal Animation ⭐⭐⭐⭐⭐
**Why**: Deep hierarchies benefit from numerical stability, repeated updates
```rust
// Bone hierarchies with many levels
struct Bone {
    local_transform: TransformMotor,  // ✅ Excellent choice
    world_transform: TransformMotor,
}
```

#### 3. Free-Flying Cameras ⭐⭐⭐⭐
**Why**: Gimbal-lock-free, smooth rotation in all directions
```rust
struct FreeCamera {
    transform: TransformMotor,  // ✅ Avoids gimbal lock
}
```

#### 4. Procedural Animation ⭐⭐⭐⭐
**Why**: Continuous transform updates, numerical stability
```rust
// Procedurally animated objects
fn update_procedural_animation(transform: &mut TransformMotor, time: f32) {
    // ✅ Stable over many updates
}
```

#### 5. High Angular Velocity Objects ⭐⭐⭐⭐
**Why**: Gimbal-lock-free, robust at extreme rotations
```rust
// Spinning objects, tumbling debris
struct SpinningObject {
    transform: TransformMotor,  // ✅ Handles extreme rotation
    spin_rate: f32,
}
```

### Use Standard Transform For:

#### 1. Keyframe Animation ⭐⭐⭐⭐⭐
**Why**: Frequent interpolation, Motors are 2.6x slower
```rust
struct AnimationClip {
    keyframes: Vec<Transform>,  // ✅ Better for interpolation
}
```

#### 2. Static Objects ⭐⭐⭐⭐
**Why**: No repeated operations, smaller memory footprint
```rust
struct StaticMesh {
    transform: Transform,  // ✅ Simpler, smaller
}
```

#### 3. UI Elements ⭐⭐⭐⭐
**Why**: Simple transforms, memory efficiency
```rust
struct UIElement {
    transform: Transform,  // ✅ Lightweight
}
```

#### 4. Simple Parent-Child Relationships ⭐⭐⭐
**Why**: Single composition per frame, standard Transform is simpler
```rust
// Simple hierarchies without deep nesting
struct SimpleHierarchy {
    parent_transform: Transform,  // ✅ Adequate
    child_offset: Transform,
}
```

### Decision Matrix

| Use Case | Motor | Standard | Reason |
|----------|-------|----------|--------|
| Physics integration | ✅ | ❌ | 4x faster repeated composition |
| Skeletal animation | ✅ | ❌ | Numerical stability + repeated updates |
| Keyframe animation | ❌ | ✅ | Frequent interpolation |
| Free camera | ✅ | ⚠️ | Gimbal-lock-free |
| Static objects | ❌ | ✅ | Simpler, smaller |
| Procedural animation | ✅ | ❌ | Continuous updates |
| UI elements | ❌ | ✅ | Lightweight |
| Particle systems (100-1000) | ✅ | ⚠️ | Competitive batch performance |
| Particle systems (<10) | ❌ | ✅ | Faster for small batches |

---

## Advanced Topics

### 1. Understanding the Geometric Product

The geometric product is the fundamental operation for composing Motors:

```rust
let combined = motor1.geometric_product(&motor2);
```

**Mathematical insight**: The geometric product combines:
- Rotation composition (like quaternion multiplication)
- Translation composition (accounting for rotation)
- All in a single, unified operation

**Properties**:
- **Associative**: `(A * B) * C = A * (B * C)`
- **Not commutative**: `A * B ≠ B * A` (order matters!)
- **Identity**: `M * I = I * M = M`

### 2. The Sandwich Product

Transforming a point uses the "sandwich product": `p' = M p M̃`

Where:
- `M` is the Motor
- `p` is the point
- `M̃` is the reverse (conjugate) of M
- `p'` is the transformed point

```rust
// This is what happens inside transform_point:
let reversed = motor.reverse();
let result = motor.geometric_product(&point_as_motor)
                  .geometric_product(&reversed);
```

### 3. Motor Logarithm and Exponential

Motors can be converted to/from bivectors using log/exp:

```rust
use luminara_math::algebra::bivector::Bivector;
use luminara_math::algebra::motor::Motor;

// Convert Motor to Bivector (logarithm)
let motor = Motor::from_axis_angle(Vec3::Y, PI / 4.0);
let bivector = motor.log();

// Convert Bivector to Motor (exponential)
let motor_reconstructed = Motor::exp(&bivector);
```

**Use case**: Velocity representation in physics (bivector = angular + linear velocity)

### 4. SIMD Optimization

Motors are structured for SIMD optimization:

```rust
// On x86_64 with AVX2, use optimized version
#[cfg(target_feature = "avx2")]
let result = motor1.geometric_product_simd(&motor2);

// Fallback to auto-vectorized version
#[cfg(not(target_feature = "avx2"))]
let result = motor1.geometric_product(&motor2);
```

The compiler auto-vectorizes the geometric product, achieving near-SIMD performance even without explicit intrinsics.

### 5. Numerical Stability in Deep Hierarchies

Motors maintain better numerical stability than matrices:

```rust
// Deep hierarchy (e.g., 50 levels)
let mut accumulated = TransformMotor::IDENTITY;
for bone_transform in bone_hierarchy {
    accumulated = accumulated.compose(&bone_transform);
    // No normalization needed! Motor stays stable.
}
```

**Why**: The geometric product preserves the motor's structure, while matrix multiplication accumulates rounding errors.

---

## Troubleshooting

### Issue: Transforms Not Updating

**Problem**: Motor transforms aren't reflected in rendering.

**Solution**: Ensure you're running the sync systems:

```rust
use luminara_scene::{
    sync_motor_to_transform_system,
    motor_transform_propagate_system,
    sync_global_motor_to_transform_system,
};

// In your update loop:
sync_motor_to_transform_system(&mut world);
motor_transform_propagate_system(&mut world);
sync_global_motor_to_transform_system(&mut world);
```

### Issue: Unexpected Rotation Behavior

**Problem**: Rotations don't behave as expected.

**Solution**: Check composition order. Remember: `parent.compose(&child)` applies parent first, then child in parent's space.

```rust
// ❌ Wrong: child's rotation in world space
let wrong = child.compose(&parent);

// ✅ Correct: child's rotation in parent's space
let correct = parent.compose(&child);
```

### Issue: Scale Not Working Correctly

**Problem**: Scale behaves unexpectedly with Motors.

**Solution**: Motors don't naturally encode scale. Scale is stored separately and applied after rotation/translation:

```rust
// Scale is applied to the final result
let motor = TransformMotor::from_position_rotation_scale(
    position,
    rotation,
    Vec3::new(2.0, 2.0, 2.0)  // Scale applied separately
);
```

### Issue: Performance Not as Expected

**Problem**: Motors aren't faster than quaternions.

**Solution**: Motors excel at **repeated** compositions. For single operations, quaternions may be faster:

```rust
// ❌ Single composition: quaternions faster
let result = motor1.compose(&motor2);  // 13ns

// ✅ Repeated compositions: Motors 4x faster!
for _ in 0..1000 {
    accumulated = accumulated.compose(&delta);  // Motors win!
}
```

### Issue: Interpolation is Slow

**Problem**: Motor interpolation is 2.6x slower than quaternion SLERP.

**Solution**: This is a known limitation. For animation-heavy workloads with frequent interpolation, consider using standard Transforms. Alternatively, wait for native Motor SLERP implementation (planned optimization).

### Issue: Memory Usage Too High

**Problem**: Motors use 33% more memory than standard Transforms.

**Solution**: For memory-constrained scenarios (mobile, embedded), use standard Transforms for static objects and reserve Motors for dynamic objects that benefit from their advantages.

---

## Further Reading

- **PGA Primer**: [bivector.net](https://bivector.net) - Interactive introduction to Geometric Algebra
- **Benchmark Results**: `motor_vs_quaternion_benchmark_results.md` - Detailed performance analysis
- **Property Tests**: `property_motor_transform_correctness_test.rs` - Correctness validation
- **Source Code**: `crates/luminara_math/src/algebra/motor.rs` - Implementation details

---

## Summary

**Motors provide a powerful alternative to quaternion-based transforms**, offering:
- ✅ Gimbal-lock-free rotations
- ✅ 4x faster repeated compositions (critical for physics/animation)
- ✅ Better numerical stability
- ✅ Unified rotation/translation representation

**Use Motors for**: Physics simulations, skeletal animation, free cameras, procedural animation, high angular velocity objects.

**Use Standard Transforms for**: Keyframe animation, static objects, UI elements, simple hierarchies.

**The key insight**: Motors aren't universally better - they excel in specific scenarios. Choose the right tool for your use case!
