use glam::Vec3;
use luminara_core::World;
use luminara_math::Transform;
use luminara_scene::*;

#[test]
fn test_scene_serialization_ron() {
    let mut components = std::collections::HashMap::new();
    components.insert(
        "Transform".to_string(),
        serde_json::to_value(Transform::from_translation(Vec3::new(1.0, 2.0, 3.0))).unwrap(),
    );

    let scene = Scene {
        meta: SceneMeta {
            name: "Test Scene".to_string(),
            description: "A test scene".to_string(),
            version: "1.0".to_string(),
            tags: vec!["test".to_string()],
        },
        entities: vec![EntityData {
            name: "Main Entity".to_string(),
            id: Some(1),
            parent: None,
            components,
            children: vec![EntityData {
                name: "Child Entity".to_string(),
                id: Some(2),
                parent: Some(1),
                components: std::collections::HashMap::new(),
                children: vec![],
                tags: vec!["child".to_string()],
            }],
            tags: vec!["main".to_string()],
        }],
    };

    let ron_string = scene.to_ron().unwrap();
    let deserialized = Scene::from_ron(&ron_string).unwrap();
    assert_eq!(deserialized.meta.name, "Test Scene");
    assert_eq!(deserialized.entities[0].children[0].name, "Child Entity");
}

#[test]
fn test_scene_serialization_json() {
    let scene = Scene {
        meta: SceneMeta {
            name: "Test Scene".to_string(),
            description: "A test scene".to_string(),
            version: "1.0".to_string(),
            tags: vec![],
        },
        entities: vec![],
    };

    let json_string = scene.to_json().unwrap();
    let deserialized = Scene::from_json(&json_string).unwrap();
    assert_eq!(deserialized.meta.name, "Test Scene");
}

#[test]
fn test_hierarchy_propagation() {
    let mut world = World::new();

    let root = world.spawn();
    let child = world.spawn();
    let grandchild = world.spawn();

    world.add_component(root, Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)));
    world.add_component(child, Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)));
    world.add_component(
        grandchild,
        Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    );

    set_parent(&mut world, child, root);
    set_parent(&mut world, grandchild, child);

    transform_propagate_system(&mut world);

    let root_global = world.get_component::<GlobalTransform>(root).unwrap().0;
    let child_global = world.get_component::<GlobalTransform>(child).unwrap().0;
    let grandchild_global = world
        .get_component::<GlobalTransform>(grandchild)
        .unwrap()
        .0;

    assert_eq!(root_global.translation, Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(child_global.translation, Vec3::new(2.0, 0.0, 0.0));
    assert_eq!(grandchild_global.translation, Vec3::new(3.0, 0.0, 0.0));
}

#[test]
fn test_name_tag_search() {
    let mut world = World::new();
    let e1 = world.spawn();
    world.add_component(e1, Name::new("Hero"));
    let mut t1 = Tag::new();
    t1.insert("Player");
    t1.insert("Heroic");
    world.add_component(e1, t1);

    let e2 = world.spawn();
    world.add_component(e2, Name::new("Enemy"));
    let mut t2 = Tag::new();
    t2.insert("Hostile");
    world.add_component(e2, t2);

    assert_eq!(find_entity_by_name(&world, "Hero"), Some(e1));
    assert_eq!(find_entity_by_name(&world, "Enemy"), Some(e2));
    assert_eq!(find_entities_by_tag(&world, "Player"), vec![e1]);
    assert_eq!(find_entities_by_tag(&world, "Heroic"), vec![e1]);
}

#[test]
fn test_prefab_instantiation() {
    let mut world = World::new();
    let prefab = Prefab {
        template: EntityData {
            name: "Template".to_string(),
            id: None,
            parent: None,
            components: std::collections::HashMap::new(),
            children: vec![],
            tags: vec!["prefab".to_string()],
        },
    };

    let instance = prefab.instantiate(&mut world);
    assert_eq!(world.get_component::<Name>(instance).unwrap().0, "Template");
    assert!(world
        .get_component::<Tag>(instance)
        .unwrap()
        .contains("prefab"));
}
