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
