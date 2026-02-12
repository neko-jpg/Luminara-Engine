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

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
        }
    }

    /// Returns the current physical pixel dimensions.
    /// Uses the stored values from the last Resized event, which are the
    /// authoritative source of truth. On WSLg/Wayland, inner_size() can
    /// lag behind the actual compositor size, so we must NOT query it here.
    pub fn physical_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Queries the underlying winit window for the current inner size.
    /// This is used as a **self-healing fallback** in the render loop:
    /// if the Resized event was missed or filtered, this will still return
    /// the actual size once the compositor has finished the resize.
    pub fn inner_size(&self) -> (u32, u32) {
        let size = self.winit_window.inner_size();
        (size.width, size.height)
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
