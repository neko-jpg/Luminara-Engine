//! Fluid visualization demo controls and settings.

use luminara::input::{ActionMap, Input, InputExt};
use luminara::input::keyboard::Key;
use luminara::input::input_map::{ActionBinding, InputSource};
use luminara::render::{FluidRenderer, FluidVisualizationMode};
use luminara::core::shared_types::{Query, Res, ResMut};
use log::info;

/// Actions available in the fluid demo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FluidAction {
    /// Switch to velocity magnitude visualization
    ModeVelocityMagnitude,
    /// Switch to velocity direction visualization
    ModeVelocityDirection,
    /// Switch to vorticity visualization
    ModeVorticity,
    /// Switch to pressure visualization
    ModePressure,
    /// Switch to streamlines visualization
    ModeStreamlines,
    /// Increase viscosity
    IncreaseViscosity,
    /// Decrease viscosity
    DecreaseViscosity,
    /// Pause/Resume simulation
    TogglePause,
    /// Reset simulation
    Reset,
}

/// Settings for the fluid demo
#[derive(Debug, Clone)]
pub struct FluidDemoSettings {
    /// Whether the simulation is paused
    pub paused: bool,
    /// Current viscosity value
    pub viscosity: f32,
    /// Minimum viscosity
    pub min_viscosity: f32,
    /// Maximum viscosity
    pub max_viscosity: f32,
    /// Viscosity adjustment step
    pub viscosity_step: f32,
}

impl Default for FluidDemoSettings {
    fn default() -> Self {
        Self {
            paused: false,
            viscosity: 0.001,
            min_viscosity: 0.0001,
            max_viscosity: 0.1,
            viscosity_step: 0.0005,
        }
    }
}

impl luminara::core::Resource for FluidDemoSettings {}

/// Setup input bindings for fluid controls
pub fn setup_fluid_input(world: &mut luminara::core::World) {
    let mut action_map = ActionMap::<FluidAction>::new();

    // Visualization mode keys
    action_map.bind(FluidAction::ModeVelocityMagnitude, ActionBinding {
        inputs: vec![InputSource::Key(Key::Num1)],
    });
    action_map.bind(FluidAction::ModeVelocityDirection, ActionBinding {
        inputs: vec![InputSource::Key(Key::Num2)],
    });
    action_map.bind(FluidAction::ModeVorticity, ActionBinding {
        inputs: vec![InputSource::Key(Key::Num3)],
    });
    action_map.bind(FluidAction::ModePressure, ActionBinding {
        inputs: vec![InputSource::Key(Key::Num4)],
    });
    action_map.bind(FluidAction::ModeStreamlines, ActionBinding {
        inputs: vec![InputSource::Key(Key::Num5)],
    });

    // Viscosity controls
    action_map.bind(FluidAction::IncreaseViscosity, ActionBinding {
        inputs: vec![InputSource::Key(Key::Equal)],
    });
    action_map.bind(FluidAction::DecreaseViscosity, ActionBinding {
        inputs: vec![InputSource::Key(Key::Minus)],
    });

    // Simulation controls
    action_map.bind(FluidAction::TogglePause, ActionBinding {
        inputs: vec![InputSource::Key(Key::Space)],
    });
    action_map.bind(FluidAction::Reset, ActionBinding {
        inputs: vec![InputSource::Key(Key::R)],
    });

    world.insert_resource(action_map);
    info!("Fluid input bindings configured");
}

/// System that handles fluid control inputs
pub fn fluid_control_system(
    input: Res<Input>,
    action_map: Res<ActionMap<FluidAction>>,
    mut settings: ResMut<FluidDemoSettings>,
    mut fluid_query: Query<&mut FluidRenderer>,
) {
    // Check for visualization mode changes
    if input.action_just_pressed(FluidAction::ModeVelocityMagnitude, &*action_map) {
        for mut renderer in fluid_query.iter_mut() {
            renderer.visualization_mode = FluidVisualizationMode::VelocityMagnitude;
            info!("Switched to Velocity Magnitude visualization");
        }
    }

    if input.action_just_pressed(FluidAction::ModeVelocityDirection, &*action_map) {
        for mut renderer in fluid_query.iter_mut() {
            renderer.visualization_mode = FluidVisualizationMode::VelocityDirection;
            info!("Switched to Velocity Direction visualization");
        }
    }

    if input.action_just_pressed(FluidAction::ModeVorticity, &*action_map) {
        for mut renderer in fluid_query.iter_mut() {
            renderer.visualization_mode = FluidVisualizationMode::Vorticity;
            info!("Switched to Vorticity visualization");
        }
    }

    if input.action_just_pressed(FluidAction::ModePressure, &*action_map) {
        for mut renderer in fluid_query.iter_mut() {
            renderer.visualization_mode = FluidVisualizationMode::Pressure;
            info!("Switched to Pressure visualization");
        }
    }

    if input.action_just_pressed(FluidAction::ModeStreamlines, &*action_map) {
        for mut renderer in fluid_query.iter_mut() {
            renderer.visualization_mode = FluidVisualizationMode::Streamlines;
            info!("Switched to Streamlines visualization");
        }
    }

    // Check for viscosity adjustments
    if input.action_just_pressed(FluidAction::IncreaseViscosity, &*action_map) {
        settings.viscosity = (settings.viscosity + settings.viscosity_step)
            .min(settings.max_viscosity);
        
        for mut renderer in fluid_query.iter_mut() {
            renderer.viscosity = settings.viscosity;
        }
        
        info!("Increased viscosity to {:.6}", settings.viscosity);
    }

    if input.action_just_pressed(FluidAction::DecreaseViscosity, &*action_map) {
        settings.viscosity = (settings.viscosity - settings.viscosity_step)
            .max(settings.min_viscosity);
        
        for mut renderer in fluid_query.iter_mut() {
            renderer.viscosity = settings.viscosity;
        }
        
        info!("Decreased viscosity to {:.6}", settings.viscosity);
    }

    // Check for pause toggle
    if input.action_just_pressed(FluidAction::TogglePause, &*action_map) {
        settings.paused = !settings.paused;
        info!("Simulation {}", if settings.paused { "paused" } else { "resumed" });
    }

    // Check for reset
    if input.action_just_pressed(FluidAction::Reset, &*action_map) {
        // Reset will be handled by recreating the solver
        // This is done by the fluid systems when they detect parameter changes
        info!("Resetting fluid simulation...");
        
        // Reset settings to default
        settings.viscosity = 0.001;
        settings.paused = false;
        
        for mut renderer in fluid_query.iter_mut() {
            renderer.viscosity = settings.viscosity;
        }
    }
}
