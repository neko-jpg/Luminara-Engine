pub mod hot_reload;

pub trait ScriptRuntime: Send + Sync {
    fn load_script(&mut self, path: &std::path::Path) -> Result<ScriptId, ScriptError>;
    fn update(&mut self, dt: f32);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptId(pub u64);

/// Enhanced script error with detailed context and stack traces
#[derive(thiserror::Error, Debug)]
pub enum ScriptError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Compilation error in {script_path}:\n{message}\n{stack_trace}")]
    Compilation {
        script_path: String,
        message: String,
        stack_trace: String,
    },
    
    #[error("Runtime error in {script_path}:\n{message}\n{stack_trace}")]
    Runtime {
        script_path: String,
        message: String,
        stack_trace: String,
    },
    
    #[error("Script not found: {0}")]
    ScriptNotFound(String),
}
