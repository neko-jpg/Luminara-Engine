use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Failed to request adapter")]
    AdapterRequestFailed,
    #[error("Failed to request device: {0}")]
    DeviceRequestFailed(String),
    #[error("Failed to create surface: {0}")]
    SurfaceCreationFailed(String),
    #[error("Shader compilation error: {0}")]
    ShaderError(String),
}
