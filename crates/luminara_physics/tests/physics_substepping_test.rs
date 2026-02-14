use luminara_core::{App, CoreStage, AppInterface, Res, ResMut};
use luminara_core::time::Time;
use luminara_physics::physics3d::{PhysicsWorld3D, physics_step_system};
use rapier3d::prelude::*;

fn run_simulation_with_variable_dt(steps: usize, dt_per_step: f32) -> Vector<f32> {
    let mut app = App::new();

    // Setup PhysicsWorld3D manually to control initialization
    let mut physics_world = PhysicsWorld3D::default();
    physics_world.timestep = 1.0 / 120.0; // Ensure known timestep

    // Add a rigid body
    let rigid_body = RigidBodyBuilder::dynamic()
        .translation(vector![0.0, 10.0, 0.0])
        .build();
    let body_handle = physics_world.rigid_body_set.insert(rigid_body);

    app.insert_resource(physics_world);
    app.insert_resource(Time::default());

    // Register system
    app.add_system::<(
        luminara_core::system::FunctionMarker,
        ResMut<'static, PhysicsWorld3D>,
        Res<'static, Time>,
        luminara_core::Query<'static, (luminara_core::Entity, &mut luminara_physics::components::PreviousTransform)>
    )>(CoreStage::Update, physics_step_system);

    // Simulate
    for _ in 0..steps {
        // Update time
        {
            let mut time = app.world.get_resource_mut::<Time>().unwrap();
            time.update_manual(dt_per_step);
        }

        // Run physics step system
        app.update();
    }

    // Get result
    let physics_world = app.world.get_resource::<PhysicsWorld3D>().unwrap();
    let body = physics_world.rigid_body_set.get(body_handle).unwrap();
    *body.translation()
}

#[test]
fn test_substepping_determinism() {
    // Simulation target: 1.0 second of physics time.
    // Physics timestep is 1/120s (0.008333...)

    // Case 1: 120 FPS (dt = 1/120)
    // 120 frames * (1/120) = 1.0s
    // Should result in exactly 1 physics step per frame -> 120 steps total.
    let dt1 = 1.0 / 120.0;
    let frames1 = 120;
    let pos1 = run_simulation_with_variable_dt(frames1, dt1);

    // Case 2: 60 FPS (dt = 1/60)
    // 60 frames * (1/60) = 1.0s
    // Should result in exactly 2 physics steps per frame -> 120 steps total.
    let dt2 = 1.0 / 60.0;
    let frames2 = 60;
    let pos2 = run_simulation_with_variable_dt(frames2, dt2);

    // Case 3: 30 FPS (dt = 1/30)
    // 30 frames * (1/30) = 1.0s
    // Should result in exactly 4 physics steps per frame -> 120 steps total.
    let dt3 = 1.0 / 30.0;
    let frames3 = 30;
    let pos3 = run_simulation_with_variable_dt(frames3, dt3);

    println!("Pos1 (120FPS): {:?}", pos1);
    println!("Pos2 (60FPS):  {:?}", pos2);
    println!("Pos3 (30FPS):  {:?}", pos3);

    // Verify positions are identical
    // Rapier is deterministic for same sequence of steps.
    // Our substepping logic ensures the same sequence of steps (120 steps of size 1/120).
    // Floating point accumulation on `accumulator` might cause slight drift on *when* the step fires if not careful,
    // but for 1.0s total time with these clean fractions, it should match.

    let epsilon = 0.0001;
    assert!((pos1.y - pos2.y).abs() < epsilon, "120FPS vs 60FPS mismatch");
    assert!((pos1.y - pos3.y).abs() < epsilon, "120FPS vs 30FPS mismatch");
}
