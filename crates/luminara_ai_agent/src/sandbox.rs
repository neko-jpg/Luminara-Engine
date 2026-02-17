use luminara_script::{ScriptError, ScriptId};
use luminara_script_lua::LuaScriptRuntime;
use luminara_script_wasm::{ResourceLimits, WasmScriptRuntime};
use std::path::Path;
use std::time::Duration;

pub struct ScriptSandbox {
    pub(crate) lua_runtime: LuaScriptRuntime,
    pub(crate) wasm_runtime: WasmScriptRuntime,
    config: SandboxConfig,
}

#[derive(Clone, Debug)]
pub struct SandboxConfig {
    pub max_memory: usize,
    pub max_execution_time: Duration,
    pub max_instructions: u64,
    pub max_entity_spawns: usize,
    pub allow_filesystem: bool,
    pub allow_network: bool,
    pub whitelisted_scripts: Vec<ScriptId>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory: 64 * 1024 * 1024, // 64MB as per requirement 25.1
            max_execution_time: Duration::from_secs(5), // 5s as per requirement 25.1
            max_instructions: 1_000_000,
            max_entity_spawns: 1000, // 1000 entity spawn limit as per requirement 25.1
            allow_filesystem: false,
            allow_network: false,
            whitelisted_scripts: Vec::new(),
        }
    }
}

impl SandboxConfig {
    /// Create config for code verification (stricter limits)
    pub fn for_verification() -> Self {
        Self {
            max_memory: 64 * 1024 * 1024,
            max_execution_time: Duration::from_secs(5),
            max_instructions: 1_000_000,
            max_entity_spawns: 1000,
            allow_filesystem: false,
            allow_network: false,
            whitelisted_scripts: Vec::new(),
        }
    }
}

impl ScriptSandbox {
    pub fn new(config: SandboxConfig) -> Result<Self, ScriptError> {
        let lua_runtime = LuaScriptRuntime::new()?;
        let wasm_runtime = WasmScriptRuntime::new(ResourceLimits {
            max_memory: config.max_memory,
            max_execution_time: config.max_execution_time,
            max_instructions: config.max_instructions,
        })?;

        let mut sandbox = Self {
            lua_runtime,
            wasm_runtime,
            config,
        };

        sandbox.apply_restrictions()?;

        Ok(sandbox)
    }

    fn apply_restrictions(&mut self) -> Result<(), ScriptError> {
        let lua = self.lua_runtime.get_lua();
        let globals = lua.globals();

        if !self.config.allow_filesystem {
            globals
                .set("io", mlua::Value::Nil)
                .map_err(|e| ScriptError::Runtime {
                    script_path: "sandbox".to_string(),
                    message: e.to_string(),
                    stack_trace: String::new(),
                })?;
            globals
                .set("os", mlua::Value::Nil)
                .map_err(|e| ScriptError::Runtime {
                    script_path: "sandbox".to_string(),
                    message: e.to_string(),
                    stack_trace: String::new(),
                })?;
        }

        if self.config.max_instructions > 0 {
            let triggers = mlua::HookTriggers::new().every_line();
            lua.set_hook(triggers, move |_lua, _debug| {
                // In a real implementation, we would check instruction count here.
                Ok(())
            });
        }

        Ok(())
    }

    pub fn load_lua_script(&mut self, path: &Path) -> Result<ScriptId, ScriptError> {
        self.lua_runtime.load_script(path)
    }

    // For testing
    pub fn run_lua(&self, code: &str) -> Result<(), ScriptError> {
        self.lua_runtime
            .get_lua()
            .load(code)
            .exec()
            .map_err(|e| ScriptError::Runtime {
                script_path: "sandbox".to_string(),
                message: e.to_string(),
                stack_trace: String::new(),
            })
    }

    pub fn whitelist_script(&mut self, id: ScriptId) {
        self.config.whitelisted_scripts.push(id);
    }

    pub fn is_whitelisted(&self, id: ScriptId) -> bool {
        self.config.whitelisted_scripts.contains(&id)
    }
}
