use luminara_core::world::World;
use luminara_core::resource::{Res, ResMut};
use luminara_core::query::Query;
use luminara_core::system::IntoSystem;
use luminara_core::app::App;
use luminara_core::shared_types::{AppInterface, CoreStage};
use luminara_core::impl_component;

#[derive(Debug)]
struct Pos { x: f32 }
impl_component!(Pos);

struct MyRes(f32);

fn my_system(world: &World) {
    let res_val = world.get_resource::<MyRes>().unwrap().0;
    let mut query = Query::<&mut Pos>::new(world);
    for pos in query.iter_mut() {
        pos.x += res_val;
    }
    *world.get_resource_mut::<u32>().unwrap() += 1;
}

#[test]
fn test_system_basic() {
    let mut app = App::new();
    app.insert_resource(MyRes(10.0));
    app.insert_resource(0u32);

    let e = app.world.spawn();
    app.world.add_component(e, Pos { x: 1.0 });

    app.add_system(CoreStage::Update, IntoSystem::<luminara_core::system::WorldRefParam>::into_system(my_system));
    app.update();

    assert_eq!(app.world.get_component::<Pos>(e).unwrap().x, 11.0);
    assert_eq!(*app.world.get_resource::<u32>().unwrap(), 1);
}
