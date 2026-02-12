use luminara_core::app::App;
use luminara_core::impl_component;
use luminara_core::shared_types::{AppInterface, CoreStage};
use luminara_core::system::{IntoSystem, SystemAccess, FunctionMarker};
use luminara_core::world::World;
use std::any::TypeId;

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}
impl_component!(Position);

#[derive(Debug)]
struct Velocity {
    dx: f32,
    dy: f32,
}
impl_component!(Velocity);

#[derive(Debug)]
struct Health(f32);
impl_component!(Health);

fn movement_system(world: &World) {
    let query = luminara_core::query::Query::<(&mut Position, &Velocity)>::new(world);
    query.par_for_each(|(pos, vel)| {
        pos.x += vel.dx;
        pos.y += vel.dy;
    });
}

fn health_system(world: &World) {
    let query = luminara_core::query::Query::<&mut Health>::new(world);
    query.par_for_each(|health| {
        health.0 -= 0.1;
    });
}

#[test]
fn stress_test_game_simulation() {
    let mut app = App::new();

    // Declarative access for parallel execution
    let mut move_access = SystemAccess::default();
    move_access
        .components_write
        .insert(TypeId::of::<Position>());
    move_access.components_read.insert(TypeId::of::<Velocity>());

    let mut health_access = SystemAccess::default();
    health_access
        .components_write
        .insert(TypeId::of::<Health>());

    app.add_system(
        CoreStage::Update,
        IntoSystem::<(FunctionMarker, World)>::into_system(movement_system as fn(&World)).with_access(move_access),
    );
    app.add_system(
        CoreStage::Update,
        IntoSystem::<(FunctionMarker, World)>::into_system(health_system as fn(&World)).with_access(health_access),
    );

    // Spawn 10,000 entities with bundles
    for _ in 0..10_000 {
        app.world.spawn_bundle((
            Position { x: 0.0, y: 0.0 },
            Velocity { dx: 1.0, dy: 1.0 },
            Health(100.0),
        ));
    }

    let start = std::time::Instant::now();
    for _ in 0..100 {
        app.update();
    }
    let elapsed = start.elapsed();
    println!("Simulated 100 frames with 10k entities in {:?}", elapsed);

    // Verify one entity
    let query = luminara_core::query::Query::<(&Position, &Health)>::new(&app.world);
    for (pos, health) in query.iter().take(1) {
        assert!(pos.x > 90.0);
        assert!(health.0 < 95.0);
    }
}
