use luminara::prelude::*;
use luminara::input::{ActionMap, InputExt, input_map::{ActionBinding, InputSource}};
use luminara::input::keyboard::Key;
use luminara::input::mouse::MouseButton;
use luminara::asset::AssetServer;
use luminara::render::{Mesh, OverlayRenderer};
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DemoAction {
    IncreaseGravity,
    DecreaseGravity,
    IncreaseTimeScale,
    DecreaseTimeScale,
    SpawnSphere,
    SpawnCube,
    ResetScene,
    ToggleDebugGizmos,
    ToggleCrosshair,
    Shoot,
}

pub fn setup_demo_input(world: &mut World) {
    let mut map = ActionMap::<DemoAction>::new();

    map.bind(DemoAction::IncreaseGravity, ActionBinding {
        inputs: vec![InputSource::Key(Key::Plus), InputSource::Key(Key::Equals)],
    });
    map.bind(DemoAction::DecreaseGravity, ActionBinding {
        inputs: vec![InputSource::Key(Key::Minus)],
    });
    map.bind(DemoAction::IncreaseTimeScale, ActionBinding {
        inputs: vec![InputSource::Key(Key::RBracket)],
    });
    map.bind(DemoAction::DecreaseTimeScale, ActionBinding {
        inputs: vec![InputSource::Key(Key::LBracket)],
    });
    map.bind(DemoAction::SpawnSphere, ActionBinding {
        inputs: vec![InputSource::Key(Key::Num1)],
    });
    map.bind(DemoAction::SpawnCube, ActionBinding {
        inputs: vec![InputSource::Key(Key::Num2)],
    });
    map.bind(DemoAction::ResetScene, ActionBinding {
        inputs: vec![InputSource::Key(Key::R)],
    });
    map.bind(DemoAction::ToggleDebugGizmos, ActionBinding {
        inputs: vec![InputSource::Key(Key::G)],
    });
    map.bind(DemoAction::ToggleCrosshair, ActionBinding {
        inputs: vec![InputSource::Key(Key::H)],
    });
    map.bind(DemoAction::Shoot, ActionBinding {
        inputs: vec![InputSource::MouseButton(MouseButton::Left)],
    });

    world.insert_resource(map);
}

#[derive(Debug, Clone, Copy)]
pub struct DemoSettings {
    pub gravity_scale: f32,
    pub time_scale: f32,
    pub show_debug_gizmos: bool,
    pub spawn_counter: u32,
}

impl Resource for DemoSettings {}

impl Default for DemoSettings {
    fn default() -> Self {
        Self {
            gravity_scale: 1.0,
            time_scale: 1.0,
            show_debug_gizmos: false,
            spawn_counter: 0,
        }
    }
}

pub fn demo_interaction_system(
    input: Res<Input>,
    map: Res<ActionMap<DemoAction>>,
    mut settings: ResMut<DemoSettings>,
    mut time: ResMut<luminara::core::Time>,
) {
    // Gravity control
    if InputExt::action_just_pressed(&*input, DemoAction::IncreaseGravity, &map) {
        settings.gravity_scale += 0.1;
        println!("Gravity scale: {:.2}", settings.gravity_scale);
    }
    if InputExt::action_just_pressed(&*input, DemoAction::DecreaseGravity, &map) {
        settings.gravity_scale = (settings.gravity_scale - 0.1).max(0.0);
        println!("Gravity scale: {:.2}", settings.gravity_scale);
    }

    // Time scale control
    if InputExt::action_just_pressed(&*input, DemoAction::IncreaseTimeScale, &map) {
        settings.time_scale = (settings.time_scale + 0.1).min(2.0);
        time.set_time_scale(settings.time_scale);
        println!("Time scale: {:.2}", settings.time_scale);
    }
    if InputExt::action_just_pressed(&*input, DemoAction::DecreaseTimeScale, &map) {
        settings.time_scale = (settings.time_scale - 0.1).max(0.1);
        time.set_time_scale(settings.time_scale);
        println!("Time scale: {:.2}", settings.time_scale);
    }

    // Debug gizmos toggle
    if InputExt::action_just_pressed(&*input, DemoAction::ToggleDebugGizmos, &map) {
        settings.show_debug_gizmos = !settings.show_debug_gizmos;
        println!("Debug gizmos: {}", if settings.show_debug_gizmos { "ON" } else { "OFF" });
    }

    // Note: Spawn and reset actions need exclusive world access
    // These will be handled in a separate exclusive system
}

#[derive(Debug, Clone, Copy)]
enum ObjectType {
    Sphere,
    Cube,
}

// Marker component for spawned objects
pub struct SpawnedObject;

impl Component for SpawnedObject {
    fn type_name() -> &'static str {
        "SpawnedObject"
    }
}

// Animation system for rotating objects
pub struct RotatingObject {
    pub axis: Vec3,
    pub speed: f32,
}

impl Component for RotatingObject {
    fn type_name() -> &'static str {
        "RotatingObject"
    }
}

pub fn rotation_animation_system(
    time: Res<luminara::core::Time>,
    mut query: Query<(&mut Transform, &RotatingObject)>,
) {
    let dt = time.delta_seconds();

    for (mut transform, rotating) in query.iter_mut() {
        let rotation = Quat::from_axis_angle(rotating.axis.normalize(), rotating.speed * dt);
        transform.rotation = transform.rotation * rotation;
    }
}

// Floating animation system
pub struct FloatingObject {
    pub amplitude: f32,
    pub frequency: f32,
    pub phase: f32,
    pub initial_y: f32,
}

impl Component for FloatingObject {
    fn type_name() -> &'static str {
        "FloatingObject"
    }
}

pub fn floating_animation_system(
    time: Res<luminara::core::Time>,
    mut query: Query<(&mut Transform, &FloatingObject)>,
) {
    let elapsed = time.elapsed_seconds();

    for (mut transform, floating) in query.iter_mut() {
        let offset = (elapsed * floating.frequency + floating.phase).sin() * floating.amplitude;
        transform.translation.y = floating.initial_y + offset;
    }
}


// Exclusive system for spawning objects (needs full world access)
pub fn demo_spawn_system(world: &mut World) {
    let input = world.resource::<Input>().clone();
    let map = world.resource::<ActionMap<DemoAction>>().clone();
    
    let mut should_spawn_sphere = false;
    let mut should_spawn_cube = false;
    let mut should_reset = false;

    // Check input
    if InputExt::action_just_pressed(&input, DemoAction::SpawnSphere, &map) {
        should_spawn_sphere = true;
    }
    if InputExt::action_just_pressed(&input, DemoAction::SpawnCube, &map) {
        should_spawn_cube = true;
    }
    if InputExt::action_just_pressed(&input, DemoAction::ResetScene, &map) {
        should_reset = true;
    }

    // Execute actions
    if should_spawn_sphere {
        let mut settings = world.resource_mut::<DemoSettings>();
        let gravity_scale = settings.gravity_scale;
        settings.spawn_counter += 1;
        let counter = settings.spawn_counter;
        drop(settings);
        
        spawn_dynamic_object_impl(world, ObjectType::Sphere, counter, gravity_scale);
    }
    
    if should_spawn_cube {
        let mut settings = world.resource_mut::<DemoSettings>();
        let gravity_scale = settings.gravity_scale;
        settings.spawn_counter += 1;
        let counter = settings.spawn_counter;
        drop(settings);
        
        spawn_dynamic_object_impl(world, ObjectType::Cube, counter, gravity_scale);
    }
    
    if should_reset {
        reset_dynamic_objects(world);
    }
}

fn spawn_dynamic_object_impl(world: &mut World, obj_type: ObjectType, counter: u32, gravity_scale: f32) {
    let name = format!("Spawned_{:?}_{}", obj_type, counter);

    world.resource_scope::<AssetServer, _, _>(|world, asset_server| {
        let entity = world.spawn();
        world.add_component(entity, Name::new(name.clone()));

        // Random position above the scene
        let mut rng = rand::thread_rng();
        let x = (rng.gen::<f32>() - 0.5) * 8.0;
        let z = (rng.gen::<f32>() - 0.5) * 8.0;
        let y = 8.0 + rng.gen::<f32>() * 4.0;

        world.add_component(
            entity,
            Transform {
                translation: Vec3::new(x, y, z),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
        );

        // Add mesh
        let mesh = match obj_type {
            ObjectType::Sphere => Mesh::sphere(0.5, 16),
            ObjectType::Cube => Mesh::cube(1.0),
        };
        let handle = asset_server.add(mesh);
        world.add_component(entity, handle);

        // Random color
        let r = rng.gen::<f32>() * 0.5 + 0.5;
        let g = rng.gen::<f32>() * 0.5 + 0.5;
        let b = rng.gen::<f32>() * 0.5 + 0.5;

        world.add_component(
            entity,
            luminara::render::PbrMaterial {
                albedo: Color::rgb(r, g, b),
                albedo_texture: None,
                normal_texture: None,
                metallic: rng.gen::<f32>() * 0.5 + 0.3,
                roughness: rng.gen::<f32>() * 0.5 + 0.3,
                metallic_roughness_texture: None,
                emissive: Color::BLACK,
            },
        );

        // Add physics
        let collider = match obj_type {
            ObjectType::Sphere => Collider {
                shape: ColliderShape::Sphere { radius: 0.5 },
                friction: 0.3,
                restitution: 0.6,
                is_sensor: false,
            },
            ObjectType::Cube => Collider {
                shape: ColliderShape::Box {
                    half_extents: Vec3::new(0.5, 0.5, 0.5),
                },
                friction: 0.4,
                restitution: 0.4,
                is_sensor: false,
            },
        };

        world.add_component(entity, collider);
        world.add_component(
            entity,
            RigidBody {
                body_type: RigidBodyType::Dynamic,
                mass: 1.0,
                linear_damping: 0.1,
                angular_damping: 0.1,
                gravity_scale,
            },
        );

        world.add_component(entity, SpawnedObject);
    });

    println!("Spawned {:?}", obj_type);
}

fn reset_dynamic_objects(world: &mut World) {
    // Remove all spawned objects
    let mut to_remove = Vec::new();
    
    // Collect entities with SpawnedObject marker
    for entity in world.entities() {
        if world.has_component::<SpawnedObject>(entity) {
            to_remove.push(entity);
        }
    }

    let count = to_remove.len();
    for entity in to_remove {
        world.despawn(entity);
    }

    // Reset settings
    let mut settings = world.resource_mut::<DemoSettings>();
    settings.spawn_counter = 0;
    settings.gravity_scale = 1.0;
    settings.time_scale = 1.0;

    println!("Scene reset: removed {} spawned objects", count);
}
