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
    world.add_component(child, Parent(parent));

    // Add to new parent's children
    if let Some(children) = world.get_component_mut::<Children>(parent) {
        children.0.push(child);
    } else {
        world.add_component(parent, Children(vec![child]));
    }
}

pub fn remove_parent(world: &mut World, child: Entity) {
    if let Some(parent) = world.remove_component::<Parent>(child) {
        if let Some(children) = world.get_component_mut::<Children>(parent.0) {
            children.0.retain(|&e| e != child);
        }
    }
}

/// Transform伝搬System
/// Note: The design doc specified a signature using Query, but since our ECS
/// infrastructure is minimal, we implement it using World for now.
pub fn transform_propagate_system(world: &mut World) {
    let entities = world.entities();
    let roots: Vec<Entity> = entities
        .into_iter()
        .filter(|&e| {
            world.get_component::<Transform>(e).is_some()
                && world.get_component::<Parent>(e).is_none()
        })
        .collect();

    for root in roots {
        let root_transform = *world.get_component::<Transform>(root).unwrap();
        world.add_component(root, GlobalTransform(root_transform));
        propagate_recursive(world, root, root_transform);
    }
}

fn propagate_recursive(world: &mut World, parent: Entity, parent_global: Transform) {
    let children = if let Some(c) = world.get_component::<Children>(parent) {
        c.0.clone()
    } else {
        return;
    };

    for child in children {
        if let Some(child_local) = world.get_component::<Transform>(child).cloned() {
            let child_global = parent_global.mul_transform(&child_local);
            world.add_component(child, GlobalTransform(child_global));
            propagate_recursive(world, child, child_global);
        }
    }
}
