//! # Luminara Window
//!
//! Window management and event handling for the Luminara Engine.
//! Powered by `winit`.

pub mod cursor;
pub mod events;
pub mod monitor;
pub mod window;
pub mod window_plugin;

pub use events::*;
pub use window::*;
pub use window_plugin::*;

use crate::events::WindowEvent as LuminaraWindowEvent;
use luminara_core::shared_types::{App, AppInterface};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{WindowAttributes, WindowId};

struct LuminaraWinitHandler {
    app: App,
    window: Option<Arc<winit::window::Window>>,
}

impl ApplicationHandler for LuminaraWinitHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let descriptor = self
                .app
                .world
                .get_resource::<window::WindowDescriptor>()
                .cloned()
                .unwrap_or_default();

            // (4) Use .with_transparent(false) to hint the compositor that the
            //     window background should be opaque. This reduces the chance of
            //     a white flash if a frame is missed during resize.
            let attributes = WindowAttributes::default()
                .with_title(&descriptor.title)
                .with_inner_size(winit::dpi::LogicalSize::new(
                    descriptor.width,
                    descriptor.height,
                ))
                .with_resizable(descriptor.resizable)
                .with_transparent(false);

            let winit_window = Arc::new(event_loop.create_window(attributes).unwrap());
            let window = window::Window::new(winit_window.clone(), &descriptor);

            // Update app with the actual window resource
            self.app.insert_resource(window);
            self.window = Some(winit_window);

            // Run startup systems now that the window is available
            // (GPU context init, user setup systems, etc.)
            self.app.schedule.run_startup(&mut self.app.world);
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(lum_event) = luminara_window_event_from_winit(&event) {
            self.app
                .world
                .get_events_mut::<LuminaraWindowEvent>()
                .send(lum_event);
        }

        match event {
            winit::event::WindowEvent::Resized(size) => {
                // (1) The Resized event's physical_size is the authoritative value.
                //     On WSLg/Wayland, inner_size() may lag, so we trust the event.
                // (3) Guard against (0,0) which occurs on minimize and would crash wgpu.
                if size.width > 0 && size.height > 0 {
                    if let Some(window) = self.app.world.get_resource_mut::<window::Window>() {
                        window.resize(size.width, size.height);
                    }
                    // Run a full update cycle so window_resize_system picks up
                    // the new stored size, reconfigures the GPU surface, and
                    // render_system draws a frame at the correct dimensions.
                    self.app.update();
                    // (2) Request redraw immediately so the compositor gets a fresh
                    //     frame at the new size, preventing white/stale borders.
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            winit::event::WindowEvent::CloseRequested => {
                _event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.app.update();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn luminara_window_event_from_winit(
    event: &winit::event::WindowEvent,
) -> Option<LuminaraWindowEvent> {
    match event {
        winit::event::WindowEvent::Resized(size) => Some(LuminaraWindowEvent::Resized {
            width: size.width,
            height: size.height,
        }),
        winit::event::WindowEvent::CloseRequested => Some(LuminaraWindowEvent::CloseRequested),
        winit::event::WindowEvent::Focused(focused) => Some(LuminaraWindowEvent::Focused(*focused)),
        winit::event::WindowEvent::Moved(pos) => {
            Some(LuminaraWindowEvent::Moved { x: pos.x, y: pos.y })
        }
        winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            Some(LuminaraWindowEvent::ScaleFactorChanged {
                scale_factor: *scale_factor,
            })
        }
        winit::event::WindowEvent::CursorEntered { .. } => Some(LuminaraWindowEvent::CursorEntered),
        winit::event::WindowEvent::CursorLeft { .. } => Some(LuminaraWindowEvent::CursorLeft),
        winit::event::WindowEvent::DroppedFile(path) => {
            Some(LuminaraWindowEvent::DroppedFile(path.clone()))
        }
        _ => None,
    }
}

pub fn winit_runner(app: App) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut handler = LuminaraWinitHandler { app, window: None };
    event_loop.run_app(&mut handler).unwrap();
}
