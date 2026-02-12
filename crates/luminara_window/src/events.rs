use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WindowEvent {
    Resized { width: u32, height: u32 },
    CloseRequested,
    Focused(bool),
    Moved { x: i32, y: i32 },
    ScaleFactorChanged { scale_factor: f64 },
    CursorEntered,
    CursorLeft,
    DroppedFile(PathBuf),
}
