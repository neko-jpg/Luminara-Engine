use bevy::prelude::*;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureUsages, CommandEncoderDescriptor,
    ImageCopyTexture, ImageDataLayout, Origin3d, TextureAspect, BufferDescriptor, BufferUsages, MapMode, Maintain,
};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::ImageCopyBuffer;
use bevy::render::{RenderApp, Render, RenderSet};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::view::RenderLayers;
use bevy::core_pipeline::core_3d::Camera3dBundle;
use bevy::render::camera::RenderTarget;
use bevy::render::render_asset::RenderAssets;
use bevy::render::texture::GpuImage;
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use luminara_core::{LuminaraCorePlugin, Motor};

// BevyBridge struct to be used by EngineHandle
pub struct BevyBridge {
    pub command_sender: Sender<Box<dyn FnOnce(&mut World) + Send + Sync>>,
    pub image_receiver: Receiver<Vec<u8>>,
}

impl BevyBridge {
    pub fn new() -> (Self, std::thread::JoinHandle<()>) {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (img_tx, img_rx) = crossbeam_channel::unbounded();

        let bridge = Self {
            command_sender: cmd_tx,
            image_receiver: img_rx,
        };

        let handle = std::thread::spawn(move || {
            start_bevy_runtime(cmd_rx, img_tx);
        });

        (bridge, handle)
    }
}

// Resource to hold the channel in the render world
#[derive(Resource)]
struct ImageSender(Sender<Vec<u8>>);

// Resource to hold the target image handle
#[derive(Resource, Clone)]
struct TargetImage(Handle<Image>);

fn start_bevy_runtime(
    cmd_rx: Receiver<Box<dyn FnOnce(&mut World) + Send + Sync>>,
    img_tx: Sender<Vec<u8>>
) {
    let mut app = App::new();

    // Configure Bevy plugins for headless execution
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: bevy::window::ExitCondition::DontExit,
                close_when_requested: false,
            })
            .disable::<bevy::log::LogPlugin>(),
    );

    // Add Luminara Core
    app.add_plugins(LuminaraCorePlugin);

    // Setup Headless Rendering
    app.insert_resource(ImageSender(img_tx));
    app.add_plugins(HeadlessRenderPlugin);

    // Add rotator system
    app.add_systems(Update, rotator_system);

    // Setup Custom Runner to handle commands and loop
    app.set_runner(move |mut app| {
        loop {
            // Process commands from Editor
            while let Ok(cmd) = cmd_rx.try_recv() {
                cmd(&mut app.world_mut());
            }

            // Update App
            app.update();

            // Throttle to ~60 FPS
            std::thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
        }
    });

    app.run();
}

struct HeadlessRenderPlugin;

impl Plugin for HeadlessRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_headless_scene);

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(Render, extract_texture_system.after(RenderSet::Render));
        }
    }

    fn finish(&self, app: &mut App) {
        if let Some(sender) = app.world().get_resource::<ImageSender>() {
            let sender_clone = ImageSender(sender.0.clone());
            if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
                render_app.insert_resource(sender_clone);
            }
        }

        app.add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<TargetImage>::default());
    }
}

impl bevy::render::extract_resource::ExtractResource for TargetImage {
    type Source = TargetImage;
    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

fn setup_headless_scene(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 1. Create Target Image
    let size = Extent3d {
        width: 800,
        height: 600,
        depth_or_array_layers: 1,
    };

    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_SRC |
        TextureUsages::RENDER_ATTACHMENT |
        TextureUsages::TEXTURE_BINDING;

    let image_handle = images.add(image);
    commands.insert_resource(TargetImage(image_handle.clone()));

    // 2. Setup Camera rendering to this image
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                target: RenderTarget::Image(image_handle),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
    ));

    // 3. Setup Scene (Cube + Light)
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.7, 0.6),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Motor::default(), // Uses Luminara Motor
        RotatorTest,      // Tag for rotation system
    ));
}

#[derive(Component)]
struct RotatorTest;

fn rotator_system(mut query: Query<(&mut Motor, &Transform), With<RotatorTest>>, time: Res<Time>) {
    for (mut motor, _transform) in query.iter_mut() {
        let angle = time.delta_seconds() * 1.0;
        let axis = luminara_math::algebra::Vector3::new(0.0, 1.0, 0.0);
        let rotor = luminara_math::algebra::Motor::from_axis_angle(axis, angle);

        motor.0 = motor.0.geometric_product(&rotor);
        motor.0.normalize();
    }
}

// Render System to extract texture
fn extract_texture_system(
    image_sender: Res<ImageSender>,
    target_image: Res<TargetImage>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    if let Some(gpu_image) = gpu_images.get(&target_image.0) {
        let size = gpu_image.size; // UVec3
        let width = size.x;
        let height = size.y;
        let bytes_per_pixel = 4; // Rgba8UnormSrgb
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = 256;
        let padded_bytes_per_row = ((unpadded_bytes_per_row + align - 1) / align) * align;
        let total_size = padded_bytes_per_row * height;

        let buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("readback_buffer"),
            size: total_size as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor { label: Some("readback_encoder") });

        encoder.copy_texture_to_buffer(
            ImageCopyTexture {
                texture: &gpu_image.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            ImageCopyBuffer {
                buffer: &buffer,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: None,
                },
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        render_queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = buffer.slice(..);
        let (tx, rx) = crossbeam_channel::bounded(1);

        buffer_slice.map_async(MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        render_device.poll(Maintain::Wait);

        if let Ok(Ok(())) = rx.recv() {
            let data = buffer_slice.get_mapped_range();
            let mut result_data = Vec::with_capacity((width * height * 4) as usize);

            // Unpad rows
            for i in 0..height {
                let start = (i * padded_bytes_per_row) as usize;
                let end = start + unpadded_bytes_per_row as usize;
                result_data.extend_from_slice(&data[start..end]);
            }

            let _ = image_sender.0.send(result_data);
        }
    }
}
