use luminara_math::{Transform, Vec3, Quat};
use luminara_scene::{GlobalTransform, Parent, Children};

#[test]
fn test_transform_to_matrix() {
    let transform = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    
    let matrix = transform.to_matrix();
    
    // Verify the translation component
    assert_eq!(matrix.w_axis.x, 1.0);
    assert_eq!(matrix.w_axis.y, 2.0);
    assert_eq!(matrix.w_axis.z, 3.0);
}

#[test]
fn test_global_transform_matrix() {
    let transform = Transform {
        translation: Vec3::new(5.0, 10.0, 15.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    
    let global = GlobalTransform(transform);
    let matrix = global.matrix();
    
    // Verify the matrix matches the transform
    assert_eq!(matrix.w_axis.x, 5.0);
    assert_eq!(matrix.w_axis.y, 10.0);
    assert_eq!(matrix.w_axis.z, 15.0);
}

#[test]
fn test_parent_children_components() {
    use luminara_core::World;
    
    let mut world = World::new();
    
    let parent_entity = world.spawn();
    let child_entity = world.spawn();
    
    // Add Parent component to child
    world.add_component(child_entity, Parent(parent_entity));
    
    // Add Children component to parent
    world.add_component(parent_entity, Children(vec![child_entity]));
    
    // Verify components exist
    assert!(world.get_component::<Parent>(child_entity).is_some());
    assert!(world.get_component::<Children>(parent_entity).is_some());
    
    // Verify the relationship
    let parent = world.get_component::<Parent>(child_entity).unwrap();
    assert_eq!(parent.0, parent_entity);
    
    let children = world.get_component::<Children>(parent_entity).unwrap();
    assert_eq!(children.0.len(), 1);
    assert_eq!(children.0[0], child_entity);
}

#[test]
fn test_transform_hierarchy_basic() {
    use luminara_core::World;
    use luminara_scene::transform_propagate_system;
    
    let mut world = World::new();
    
    // Create parent with translation
    let parent = world.spawn();
    world.add_component(parent, Transform {
        translation: Vec3::new(10.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });
    world.add_component(parent, GlobalTransform::default());
    
    // Create child with local translation
    let child = world.spawn();
    world.add_component(child, Transform {
        translation: Vec3::new(5.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });
    world.add_component(child, GlobalTransform::default());
    world.add_component(child, Parent(parent));
    world.add_component(parent, Children(vec![child]));
    
    // Run transform propagation
    transform_propagate_system(&mut world);
    
    // Verify parent global transform
    let parent_global = world.get_component::<GlobalTransform>(parent).unwrap();
    let parent_matrix = parent_global.matrix();
    assert!((parent_matrix.w_axis.x - 10.0).abs() < 0.001);
    
    // Verify child global transform (should be parent + child local = 15.0)
    let child_global = world.get_component::<GlobalTransform>(child).unwrap();
    let child_matrix = child_global.matrix();
    assert!((child_matrix.w_axis.x - 15.0).abs() < 0.001);
}

#[test]
fn test_transform_hierarchy_breadth_first() {
    use luminara_core::World;
    use luminara_scene::transform_propagate_system;
    
    let mut world = World::new();
    
    // Create a hierarchy:
    //       root (10, 0, 0)
    //      /    \
    //   child1  child2 (5, 0, 0 each)
    //     |       |
    //  grand1  grand2 (2, 0, 0 each)
    
    let root = world.spawn();
    world.add_component(root, Transform {
        translation: Vec3::new(10.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });
    world.add_component(root, GlobalTransform::default());
    
    let child1 = world.spawn();
    world.add_component(child1, Transform {
        translation: Vec3::new(5.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });
    world.add_component(child1, GlobalTransform::default());
    world.add_component(child1, Parent(root));
    
    let child2 = world.spawn();
    world.add_component(child2, Transform {
        translation: Vec3::new(5.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });
    world.add_component(child2, GlobalTransform::default());
    world.add_component(child2, Parent(root));
    
    let grand1 = world.spawn();
    world.add_component(grand1, Transform {
        translation: Vec3::new(2.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });
    world.add_component(grand1, GlobalTransform::default());
    world.add_component(grand1, Parent(child1));
    
    let grand2 = world.spawn();
    world.add_component(grand2, Transform {
        translation: Vec3::new(2.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });
    world.add_component(grand2, GlobalTransform::default());
    world.add_component(grand2, Parent(child2));
    
    // Set up Children components
    world.add_component(root, Children(vec![child1, child2]));
    world.add_component(child1, Children(vec![grand1]));
    world.add_component(child2, Children(vec![grand2]));
    
    // Run transform propagation
    transform_propagate_system(&mut world);
    
    // Verify root global transform
    let root_global = world.get_component::<GlobalTransform>(root).unwrap();
    assert!((root_global.matrix().w_axis.x - 10.0).abs() < 0.001);
    
    // Verify child1 global transform (10 + 5 = 15)
    let child1_global = world.get_component::<GlobalTransform>(child1).unwrap();
    assert!((child1_global.matrix().w_axis.x - 15.0).abs() < 0.001);
    
    // Verify child2 global transform (10 + 5 = 15)
    let child2_global = world.get_component::<GlobalTransform>(child2).unwrap();
    assert!((child2_global.matrix().w_axis.x - 15.0).abs() < 0.001);
    
    // Verify grand1 global transform (15 + 2 = 17)
    let grand1_global = world.get_component::<GlobalTransform>(grand1).unwrap();
    assert!((grand1_global.matrix().w_axis.x - 17.0).abs() < 0.001);
    
    // Verify grand2 global transform (15 + 2 = 17)
    let grand2_global = world.get_component::<GlobalTransform>(grand2).unwrap();
    assert!((grand2_global.matrix().w_axis.x - 17.0).abs() < 0.001);
}

#[test]
fn test_transform_hierarchy_with_rotation_and_scale() {
    use luminara_core::World;
    use luminara_scene::transform_propagate_system;
    use std::f32::consts::PI;
    
    let mut world = World::new();
    
    // Create parent with rotation and scale
    let parent = world.spawn();
    world.add_component(parent, Transform {
        translation: Vec3::new(0.0, 0.0, 0.0),
        rotation: Quat::from_rotation_z(PI / 2.0), // 90 degrees around Z
        scale: Vec3::new(2.0, 2.0, 2.0),
    });
    world.add_component(parent, GlobalTransform::default());
    
    // Create child with local translation
    let child = world.spawn();
    world.add_component(child, Transform {
        translation: Vec3::new(1.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });
    world.add_component(child, GlobalTransform::default());
    world.add_component(child, Parent(parent));
    world.add_component(parent, Children(vec![child]));
    
    // Run transform propagation
    transform_propagate_system(&mut world);
    
    // Verify child global transform
    // After 90-degree rotation and 2x scale, (1, 0, 0) should become approximately (0, 2, 0)
    let child_global = world.get_component::<GlobalTransform>(child).unwrap();
    let child_pos = child_global.matrix().w_axis.truncate();
    
    assert!(child_pos.x.abs() < 0.001, "Expected x ≈ 0, got {}", child_pos.x);
    assert!((child_pos.y - 2.0).abs() < 0.001, "Expected y ≈ 2, got {}", child_pos.y);
    assert!(child_pos.z.abs() < 0.001, "Expected z ≈ 0, got {}", child_pos.z);
}
