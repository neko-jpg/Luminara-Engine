pub mod hot_reload;

pub trait ScriptRuntime: Send + Sync {
    fn load_script(&mut self, path: &std::path::Path) -> Result<ScriptId, ScriptError>;
    fn update(&mut self, dt: f32);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptId(pub u64);

#[derive(thiserror::Error, Debug)]
pub enum ScriptError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Compilation error: {0}")]
    Compilation(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
}
