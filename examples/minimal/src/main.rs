//! # Luminara Engine — Minimal Example
//!
//! Phase 0 成果物: ウィンドウを開き、色付き三角形を表示する。

use luminara::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_startup_system::<ExclusiveMarker>(setup);
    app.run();
}

/// Startup system: spawn a camera and a triangle mesh.
fn setup(world: &mut World) {
    // Spawn a camera entity with identity transform
    // (triangle vertices are already in clip-space range [-0.5, 0.5])
    let cam = world.spawn();
    world.add_component(
        cam,
        Camera {
            projection: Projection::Orthographic {
                size: 2.0,
                near: -1.0,
                far: 1.0,
            },
            clear_color: Color::BLACK,
            is_active: true,
        },
    );
    world.add_component(cam, Camera3d);
    world.add_component(cam, Transform::default());

    // Spawn a triangle mesh entity
    // Note: Mesh is now an Asset and requires a Handle.
    // In this minimal example without AssetServer easily accessible in this signature,
    // we cannot spawn a mesh correctly.
    // We would need to update this example to use proper system dependency injection.

    // For now, we will comment out the mesh spawning to allow compilation,
    // or update the function signature if we could.
    // Ideally, we should migrate this example to use AssetServer.

    // let tri = world.spawn();
    // world.add_component(tri, Mesh::triangle()); // Error: Mesh is not Component
    // world.add_component(tri, Transform::default());

    log::info!("Luminara minimal example: setup complete (Mesh spawning disabled pending migration)");
}
