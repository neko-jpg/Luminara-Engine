use luminara_core::shared_types::Resource;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, DisplayHandle, WindowHandle, RawDisplayHandle, RawWindowHandle, HandleError};

pub struct Window {
    pub width: u32,
    pub height: u32,
    display_handle: RawDisplayHandle,
    window_handle: RawWindowHandle,
}

impl Window {
    pub fn new(width: u32, height: u32, display_handle: RawDisplayHandle, window_handle: RawWindowHandle) -> Self {
        Self {
            width,
            height,
            display_handle,
            window_handle,
        }
    }
}

// SAFETY: Window handles are generally safe to pass between threads on most modern platforms,
// but wgpu and other libraries handle the actual synchronization.
unsafe impl Send for Window {}
unsafe impl Sync for Window {}

impl Resource for Window {}

impl HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        unsafe { Ok(DisplayHandle::borrow_raw(self.display_handle)) }
    }
}

impl HasWindowHandle for Window {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        unsafe { Ok(WindowHandle::borrow_raw(self.window_handle)) }
    }
}
pub mod window;
pub mod window_plugin;
pub mod events;
pub mod monitor;
pub mod cursor;

pub use window::*;
pub use window_plugin::*;
pub use events::*;

use luminara_core::shared_types::{App, AppInterface, Events};
use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{WindowId, WindowAttributes};
use crate::events::WindowEvent as LuminaraWindowEvent;
use std::sync::Arc;

struct LuminaraWinitHandler {
    app: App,
    window: Option<Arc<winit::window::Window>>,
}

impl ApplicationHandler for LuminaraWinitHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            // In a real implementation, we would get this from app resources.
            // For now, we use default or a placeholder.
            let descriptor = window::WindowDescriptor::default();

            let attributes = WindowAttributes::default()
                .with_title(&descriptor.title)
                .with_inner_size(winit::dpi::LogicalSize::new(descriptor.width, descriptor.height))
                .with_resizable(descriptor.resizable);

            let winit_window = Arc::new(event_loop.create_window(attributes).unwrap());
            let window = window::Window::new(winit_window.clone(), &descriptor);

            // Update app with the actual window resource
            self.app.insert_resource(window);
            self.window = Some(winit_window);
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _window_id: WindowId, event: winit::event::WindowEvent) {
        if let Some(lum_event) = luminara_window_event_from_winit(&event) {
            if let Some(events) = self.app.get_resource_mut::<Events<LuminaraWindowEvent>>() {
                events.send(lum_event);
            }
        }

        match event {
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

fn luminara_window_event_from_winit(event: &winit::event::WindowEvent) -> Option<LuminaraWindowEvent> {
    match event {
        winit::event::WindowEvent::Resized(size) => Some(LuminaraWindowEvent::Resized {
            width: size.width,
            height: size.height,
        }),
        winit::event::WindowEvent::CloseRequested => Some(LuminaraWindowEvent::CloseRequested),
        winit::event::WindowEvent::Focused(focused) => Some(LuminaraWindowEvent::Focused(*focused)),
        winit::event::WindowEvent::Moved(pos) => Some(LuminaraWindowEvent::Moved {
            x: pos.x,
            y: pos.y,
        }),
        winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            Some(LuminaraWindowEvent::ScaleFactorChanged { scale_factor: *scale_factor })
        }
        winit::event::WindowEvent::CursorEntered { .. } => Some(LuminaraWindowEvent::CursorEntered),
        winit::event::WindowEvent::CursorLeft { .. } => Some(LuminaraWindowEvent::CursorLeft),
        winit::event::WindowEvent::DroppedFile(path) => Some(LuminaraWindowEvent::DroppedFile(path.clone())),
        _ => None,
    }
}

pub fn winit_runner(app: App) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut handler = LuminaraWinitHandler { app, window: None };
    event_loop.run_app(&mut handler).unwrap();
}
