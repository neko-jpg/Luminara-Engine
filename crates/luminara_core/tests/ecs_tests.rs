use luminara_core::impl_component;
use luminara_core::world::World;
use luminara_core::Resource;

#[derive(Debug, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}
impl_component!(Position);

#[derive(Debug, PartialEq, Eq)]
struct Velocity {
    dx: i32,
    dy: i32,
}
impl_component!(Velocity);

#[derive(Debug, PartialEq, Eq)]
struct TestResource(u32);
impl Resource for TestResource {}

#[test]
fn test_world_spawn_and_components() {
    let mut world = World::new();
    let e1 = world.spawn();

    world.add_component(e1, Position { x: 10, y: 20 });

    {
        let pos = world
            .get_component::<Position>(e1)
            .expect("Position should exist");
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
    }

    world.add_component(e1, Velocity { dx: 1, dy: 2 });

    {
        let pos = world
            .get_component::<Position>(e1)
            .expect("Position should still exist");
        assert_eq!(pos.x, 10);
        let vel = world
            .get_component::<Velocity>(e1)
            .expect("Velocity should exist");
        assert_eq!(vel.dx, 1);
    }

    world.despawn(e1);
    assert!(world.get_component::<Position>(e1).is_none());
}

#[test]
fn test_world_resources() {
    let mut world = World::new();
    world.insert_resource(TestResource(100));

    assert_eq!(world.get_resource::<TestResource>().unwrap().0, 100);

    world.get_resource_mut::<TestResource>().unwrap().0 = 200;
    assert_eq!(world.get_resource::<TestResource>().unwrap().0, 200);
}

#[test]
fn test_world_events() {
    let mut world = World::new();
    struct MyEvent(u32);

    world.add_event(MyEvent(1));
    world.add_event(MyEvent(2));

    {
        let events = world.get_events::<MyEvent>().unwrap();
        let current: Vec<_> = events.iter_current().collect();
        assert_eq!(current.len(), 2);
    }

    world.get_events_mut::<MyEvent>().unwrap().update();

    {
        let events = world.get_events::<MyEvent>().unwrap();
        let previous: Vec<_> = events.iter_previous().collect();
        assert_eq!(previous.len(), 2);
        let current: Vec<_> = events.iter_current().collect();
        assert_eq!(current.len(), 0);
    }
}

#[test]
fn test_world_query() {
    use luminara_core::query::{Query, With, Without};

    let mut world = World::new();
    let e1 = world.spawn();
    world.add_component(e1, Position { x: 1, y: 1 });
    world.add_component(e1, Velocity { dx: 10, dy: 10 });

    let e2 = world.spawn();
    world.add_component(e2, Position { x: 2, y: 2 });

    let e3 = world.spawn();
    world.add_component(e3, Velocity { dx: 30, dy: 30 });

    // Query for Position
    {
        let query = Query::<&Position>::new(&world);
        let results: Vec<_> = query.iter().collect();
        assert_eq!(results.len(), 2);
    }

    // Query for (Position, Velocity)
    {
        let query = Query::<(&Position, &Velocity)>::new(&world);
        let results: Vec<_> = query.iter().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.x, 1);
        assert_eq!(results[0].1.dx, 10);
    }

    // Query with With<Velocity>
    {
        let query = Query::<&Position, With<Velocity>>::new(&world);
        let results: Vec<_> = query.iter().collect();
        assert_eq!(results.len(), 1);
    }

    // Query with Without<Velocity>
    {
        let query = Query::<&Position, Without<Velocity>>::new(&world);
        let results: Vec<_> = query.iter().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].x, 2);
    }

    // Query iter_mut
    {
        let mut query = Query::<&mut Position>::new(&world);
        for pos in query.iter_mut() {
            pos.x += 100;
        }
    }

    assert_eq!(world.get_component::<Position>(e1).unwrap().x, 101);
    assert_eq!(world.get_component::<Position>(e2).unwrap().x, 102);
}

#[test]
fn test_or_filter() {
    use luminara_core::entity::Entity;
    use luminara_core::query::{Or, Query, With};

    let mut world = World::new();
    let e1 = world.spawn();
    world.add_component(e1, Position { x: 1, y: 1 });

    let e2 = world.spawn();
    world.add_component(e2, Velocity { dx: 10, dy: 10 });

    let _e3 = world.spawn();
    // No components

    // Query for entities with either Position OR Velocity
    let query = Query::<Entity, Or<(With<Position>, With<Velocity>)>>::new(&world);
    let results: Vec<_> = query.iter().collect();
    assert_eq!(results.len(), 2);
}

