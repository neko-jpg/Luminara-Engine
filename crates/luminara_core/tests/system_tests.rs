use luminara_core::app::App;
use luminara_core::impl_component;
use luminara_core::query::Query;
use luminara_core::shared_types::{AppInterface, CoreStage};
use luminara_core::system::FunctionMarker;
use luminara_core::world::World;
use luminara_core::Resource;

#[derive(Debug)]
struct Pos {
    x: f32,
}
impl_component!(Pos);

struct MyRes(f32);
impl Resource for MyRes {}

#[derive(Debug, PartialEq, Eq)]
struct TestCounter(u32);
impl Resource for TestCounter {}

fn my_system(world: &World) {
    let res_val = world.get_resource::<MyRes>().unwrap().0;
    let mut query = Query::<&mut Pos>::new(world);
    for pos in query.iter_mut() {
        pos.x += res_val;
    }
    world.get_resource_mut::<TestCounter>().unwrap().0 += 1;
}

#[test]
fn test_system_basic() {
    let mut app = App::new();
    app.insert_resource(MyRes(10.0));
    app.insert_resource(TestCounter(0));

    let e = app.world.spawn();
    app.world.add_component(e, Pos { x: 1.0 });

    app.add_system::<(FunctionMarker, World)>(CoreStage::Update, my_system as fn(&World));
    app.update();

    assert_eq!(app.world.get_component::<Pos>(e).unwrap().x, 11.0);
    assert_eq!(app.world.get_resource::<TestCounter>().unwrap().0, 1);
}
