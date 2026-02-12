use luminara_core::app::App;
use luminara_core::shared_types::{AppInterface, CoreStage};
use luminara_core::system::{FunctionMarker, IntoSystem, SystemAccess};
use luminara_core::world::World;
use luminara_core::Resource;
use std::any::TypeId;

#[derive(Debug, PartialEq, Eq)]
struct ResA(u32);
impl Resource for ResA {}

#[derive(Debug, PartialEq, Eq)]
struct ResB(i32);
impl Resource for ResB {}

#[test]
fn test_parallel_systems() {
    let mut app = App::new();
    app.insert_resource(ResA(0));
    app.insert_resource(ResB(0));

    fn sys_a(world: &World) {
        world.get_resource_mut::<ResA>().unwrap().0 += 1;
    }

    fn sys_b(world: &World) {
        world.get_resource_mut::<ResB>().unwrap().0 += 1;
    }

    let mut access_a = SystemAccess::default();
    access_a.resources_write.insert(TypeId::of::<ResA>());

    let mut access_b = SystemAccess::default();
    access_b.resources_write.insert(TypeId::of::<ResB>());

    app.add_system(
        CoreStage::Update,
        IntoSystem::<(FunctionMarker, World)>::into_system(sys_a as fn(&World)).with_access(access_a),
    );
    app.add_system(
        CoreStage::Update,
        IntoSystem::<(FunctionMarker, World)>::into_system(sys_b as fn(&World)).with_access(access_b),
    );

    app.update();

    assert_eq!(app.world.get_resource::<ResA>().unwrap().0, 1);
    assert_eq!(app.world.get_resource::<ResB>().unwrap().0, 1);
}

#[test]
fn test_query_par_for_each() {
    use luminara_core::impl_component;
    use luminara_core::query::Query;
    use luminara_core::world::World;

    #[derive(Debug)]
    struct C1(u32);
    impl_component!(C1);

    let mut world = World::new();
    for i in 0..1000 {
        let e = world.spawn();
        world.add_component(e, C1(i));
    }

    let query = Query::<&C1>::new(&world);
    let count = std::sync::atomic::AtomicU32::new(0);
    query.par_for_each(|c| {
        count.fetch_add(c.0, std::sync::atomic::Ordering::Relaxed);
    });

    assert_eq!(
        count.load(std::sync::atomic::Ordering::Relaxed),
        (0..1000).sum()
    );
}
