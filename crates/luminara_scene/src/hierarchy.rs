use luminara_core::{Entity, World};
use luminara_math::Transform;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parent(pub Entity);

impl luminara_core::Component for Parent {
    fn type_name() -> &'static str {
        "Parent"
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Children(pub Vec<Entity>);

impl luminara_core::Component for Children {
    fn type_name() -> &'static str {
        "Children"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalTransform(pub Transform);

impl GlobalTransform {
    /// Get the world transformation matrix.
    pub fn matrix(&self) -> luminara_math::Mat4 {
        self.0.to_matrix()
    }
}

impl Default for GlobalTransform {
    fn default() -> Self {
        Self(Transform::IDENTITY)
    }
}

impl luminara_core::Component for GlobalTransform {
    fn type_name() -> &'static str {
        "GlobalTransform"
    }
}

pub fn set_parent(world: &mut World, child: Entity, parent: Entity) {
    // Remove from old parent's children if any
    let old_parent = world.get_component::<Parent>(child).map(|p| p.0);
    if let Some(op) = old_parent {
        if let Some(children) = world.get_component_mut::<Children>(op) {
            children.0.retain(|&e| e != child);
        }
    }

    // Set new parent
    let _ = world.add_component(child, Parent(parent));

    // Add to new parent's children
    if let Some(children) = world.get_component_mut::<Children>(parent) {
        children.0.push(child);
    } else {
        let _ = world.add_component(parent, Children(vec![child]));
    }
}

pub fn remove_parent(world: &mut World, child: Entity) {
    if let Ok(Some(parent)) = world.remove_component::<Parent>(child) {
        if let Some(children) = world.get_component_mut::<Children>(parent.0) {
            children.0.retain(|&e| e != child);
        }
    }
}

/// Transform propagation system using breadth-first traversal.
///
/// This system traverses the entity hierarchy in breadth-first order to update
/// GlobalTransform components based on parent-child relationships.
///
/// Requirements: 5.1, 5.2
pub fn transform_propagate_system(world: &mut World) {
    use std::collections::VecDeque;

    // Find all root entities (entities with Transform but no Parent)
    let entities = world.entities();
    let roots: Vec<Entity> = entities
        .into_iter()
        .filter(|&e| {
            world.get_component::<Transform>(e).is_some()
                && world.get_component::<Parent>(e).is_none()
        })
        .collect();

    // Process each root hierarchy using breadth-first traversal
    for root in roots {
        let root_transform = *world.get_component::<Transform>(root).unwrap();
        let _ = world.add_component(root, GlobalTransform(root_transform));

        // Queue for breadth-first traversal: (entity, parent_global_matrix)
        let mut queue = VecDeque::new();

        // Add root's children to the queue
        if let Some(children) = world.get_component::<Children>(root) {
            let root_matrix = root_transform.to_matrix();
            for &child in &children.0 {
                queue.push_back((child, root_matrix));
            }
        }

        // Process queue in breadth-first order
        while let Some((entity, parent_matrix)) = queue.pop_front() {
            if let Some(local_transform) = world.get_component::<Transform>(entity).cloned() {
                // Compute global transform: parent_world * child_local
                let local_matrix = local_transform.to_matrix();
                let global_matrix = parent_matrix * local_matrix;

                // Decompose matrix back to Transform for GlobalTransform
                let (scale, rotation, translation) = global_matrix.to_scale_rotation_translation();
                let global_transform = Transform {
                    translation,
                    rotation,
                    scale,
                };

                let _ = world.add_component(entity, GlobalTransform(global_transform));

                // Add this entity's children to the queue
                if let Some(children) = world.get_component::<Children>(entity) {
                    for &child in &children.0 {
                        queue.push_back((child, global_matrix));
                    }
                }
            }
        }
    }
}
