//! Basic editor example (Vizia version)
//!
//! This example demonstrates the initialization of the Vizia-based Luminara Editor
//! with a minimal engine setup and command-based keyboard input.

use luminara_asset::AssetServer;
use luminara_core::App;
use luminara_editor::{
    core::window::EditorWindowState, services::engine_bridge::EngineHandle, ui::theme::Theme,
};
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use vizia::prelude::*;

fn main() -> Result<(), ApplicationError> {
    let mut engine_app = App::new();

    use luminara_scene::hierarchy::{Children, Parent};
    use luminara_scene::scene::Name;

    let camera = engine_app.world.spawn();
    let _ = engine_app
        .world
        .add_component(camera, Name::new("Main Camera"));

    let light = engine_app.world.spawn();
    let _ = engine_app
        .world
        .add_component(light, Name::new("Directional Light"));

    let player = engine_app.world.spawn();
    let _ = engine_app
        .world
        .add_component(player, Name::new("Player Character"));

    let mesh = engine_app.world.spawn();
    let _ = engine_app.world.add_component(mesh, Name::new("Body Mesh"));
    let _ = engine_app.world.add_component(mesh, Parent(player));
    let _ = engine_app.world.add_component(player, Children(vec![mesh]));

    let world = Arc::new(RwLock::new(engine_app.world));

    let asset_server = Arc::new(AssetServer::new(PathBuf::from("assets")));
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let database = Arc::new(
        rt.block_on(luminara_editor::Database::new_memory())
            .expect("Failed to create database"),
    );
    drop(rt);

    let render_pipeline = Arc::new(RwLock::new(luminara_editor::RenderPipeline::mock()));

    let engine_handle = Arc::new(EngineHandle::new(
        world,
        asset_server,
        database,
        render_pipeline,
    ));

    let theme = Arc::new(Theme::default_dark());

    Application::new(move |cx| {
        EditorWindowState::new(theme.clone()).build(cx);
    })
    .title("Luminara Editor")
    .inner_size((1200, 800))
    .run()
}
