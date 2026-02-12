use luminara_core::impl_component;
use luminara_core::query::Query;
use luminara_core::world::World;

#[derive(Debug)]
struct Position {
    x: i32,
    y: i32,
}
impl_component!(Position);

#[test]
fn bench_spawn_100k() {
    let mut world = World::new();
    let start = std::time::Instant::now();
    for _ in 0..100_000 {
        let e = world.spawn();
        world.add_component(e, Position { x: 0, y: 0 });
    }
    let elapsed = start.elapsed();
    println!("Spawned 100k entities in {:?}", elapsed);
}

#[test]
fn bench_query_1m() {
    let mut world = World::new();
    // To speed up setup, we can use a more efficient way if available,
    // but here we just want to measure iteration.
    for _ in 0..1_000_000 {
        let e = world.spawn();
        world.add_component(e, Position { x: 0, y: 0 });
    }

    let start = std::time::Instant::now();
    let query = Query::<&Position>::new(&world);
    let mut count = 0;
    for _ in query.iter() {
        count += 1;
    }
    let elapsed = start.elapsed();
    println!("Iterated over {} entities in {:?}", count, elapsed);

    // In release mode it should be < 1ms.
    // In debug mode it might be slower.
}
