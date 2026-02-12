use luminara_core::shared_types::Resource;
use luminara_math::IVec2;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowMode {
    Windowed,
    BorderlessFullscreen,
    Fullscreen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CursorGrab {
    None,
    Confined,
    Locked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowDescriptor {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub mode: WindowMode,
    pub vsync: bool,
    pub resizable: bool,
}

impl Default for WindowDescriptor {
    fn default() -> Self {
        Self {
            title: "Luminara Game".to_string(),
            width: 1280,
            height: 720,
            mode: WindowMode::Windowed,
            vsync: true,
            resizable: true,
        }
    }
}

impl Resource for WindowDescriptor {}

pub struct Window {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub position: Option<IVec2>,
    pub mode: WindowMode,
    pub vsync: bool,
    pub resizable: bool,
    pub(crate) winit_window: Arc<winit::window::Window>,
}

impl Window {
    pub fn new(winit_window: Arc<winit::window::Window>, descriptor: &WindowDescriptor) -> Self {
        let size = winit_window.inner_size();
        let pos = winit_window
            .outer_position()
            .ok()
            .map(|p| IVec2::new(p.x, p.y));
        Self {
            title: descriptor.title.clone(),
            width: size.width,
            height: size.height,
            position: pos,
            mode: descriptor.mode,
            vsync: descriptor.vsync,
            resizable: descriptor.resizable,
            winit_window,
        }
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        self.winit_window.set_title(title);
    }

    pub fn request_redraw(&self) {
        self.winit_window.request_redraw();
    }
}

impl Resource for Window {}

impl HasWindowHandle for Window {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.winit_window.window_handle()
    }
}

impl HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        self.winit_window.display_handle()
    }
}

// SAFETY: winit::window::Window is generally not thread-safe and must be
// accessed from the main thread. We implement Send and Sync to allow
// storing the Window in the ECS Resources, but care must be taken to
// only call its methods from the thread it was created on (usually the main thread).
unsafe impl Send for Window {}
unsafe impl Sync for Window {}
