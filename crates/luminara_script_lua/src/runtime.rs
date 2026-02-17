use crate::api::{input::LuaInput, world::LuaWorld};
use luminara_core::world::World;
use luminara_input::Input;
use luminara_script::{ScriptError, ScriptId};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct LuaScriptRuntime {
    pub(crate) lua: mlua::Lua,
    scripts: HashMap<ScriptId, LoadedScript>,
    path_to_id: HashMap<PathBuf, ScriptId>,
    next_id: u64,
}

pub struct LoadedScript {
    pub id: ScriptId,
    pub path: PathBuf,
    pub factory_key: mlua::RegistryKey,
    pub instance_key: Option<mlua::RegistryKey>,
}

impl LuaScriptRuntime {
    pub fn new() -> Result<Self, ScriptError> {
        // Use default standard libraries for now (excludes debug/io/os if configured properly via new_with if we had the flags)
        // Since StdLib::BASE etc are not found, we use safe defaults or new()
        let lua = mlua::Lua::new();

        // Security: Set instruction limit to prevent infinite loops (e.g. 100,000 instructions)
        // lua.set_interrupt is not available in standard mlua 0.9 without features or changed API.
        // TODO: Re-enable instruction limit/sandbox once API is clarified.
        // lua.set_interrupt(|_| {
        //     Ok(mlua::VmState::Continue)
        // });

        Ok(Self {
            lua,
            scripts: HashMap::new(),
            path_to_id: HashMap::new(),
            next_id: 0,
        })
    }

    pub fn get_lua(&self) -> &mlua::Lua {
        &self.lua
    }

    /// Extract detailed stack trace from Lua error
    fn extract_stack_trace(&self, error: &mlua::Error) -> String {
        match error {
            mlua::Error::CallbackError { traceback, cause } => {
                format!("Callback error:\n{}\nCause: {}", traceback, cause)
            }
            mlua::Error::RuntimeError(msg) => {
                // Try to get debug traceback
                if let Ok(debug_traceback) = self.lua.load("debug.traceback()").eval::<String>() {
                    format!("{}\n\nStack trace:\n{}", msg, debug_traceback)
                } else {
                    msg.clone()
                }
            }
            mlua::Error::SyntaxError { message, .. } => message.clone(),
            _ => error.to_string(),
        }
    }

    /// Safely execute a Lua function with error isolation
    fn safe_call<'lua, A, R>(
        &'lua self,
        func: mlua::Function<'lua>,
        args: A,
        script_path: &str,
    ) -> Result<R, ScriptError>
    where
        A: mlua::IntoLuaMulti<'lua>,
        R: mlua::FromLuaMulti<'lua>,
    {
        match func.call::<A, R>(args) {
            Ok(result) => Ok(result),
            Err(e) => {
                let stack_trace = self.extract_stack_trace(&e);
                Err(ScriptError::Runtime {
                    script_path: script_path.to_string(),
                    message: e.to_string(),
                    stack_trace,
                })
            }
        }
    }

    pub fn load_script(&mut self, path: &Path) -> Result<ScriptId, ScriptError> {
        let path_buf = path.to_path_buf();

        if let Some(&id) = self.path_to_id.get(&path_buf) {
            return Ok(id);
        }

        let source = std::fs::read_to_string(path).map_err(ScriptError::Io)?;
        let path_str = path.display().to_string();

        let chunk = self.lua.load(&source);
        let func = chunk.into_function().map_err(|e| {
            let stack_trace = self.extract_stack_trace(&e);
            ScriptError::Compilation {
                script_path: path_str.clone(),
                message: e.to_string(),
                stack_trace,
            }
        })?;

        let factory_key = self.lua.create_registry_value(func.clone()).map_err(|e| {
            let stack_trace = self.extract_stack_trace(&e);
            ScriptError::Runtime {
                script_path: path_str.clone(),
                message: e.to_string(),
                stack_trace,
            }
        })?;

        // Execute script body - need to call directly to avoid borrow issues
        let result: mlua::Value = match func.call(()) {
            Ok(v) => v,
            Err(e) => {
                let stack_trace = self.extract_stack_trace(&e);
                return Err(ScriptError::Runtime {
                    script_path: path_str,
                    message: e.to_string(),
                    stack_trace,
                });
            }
        };

        let instance_key = if let mlua::Value::Table(t) = result {
            Some(self.lua.create_registry_value(t).map_err(|e| {
                let stack_trace = self.extract_stack_trace(&e);
                ScriptError::Runtime {
                    script_path: path_str.clone(),
                    message: e.to_string(),
                    stack_trace,
                }
            })?)
        } else {
            None
        };

        let id = ScriptId(self.next_id);
        self.next_id += 1;

        let script = LoadedScript {
            id,
            path: path_buf.clone(),
            factory_key,
            instance_key,
        };

        self.scripts.insert(id, script);
        self.path_to_id.insert(path_buf, id);

        Ok(id)
    }

    pub fn reload_script(&mut self, script_id: ScriptId) -> Result<(), ScriptError> {
        // Get script path first to avoid borrow issues
        let path_str = self
            .scripts
            .get(&script_id)
            .ok_or_else(|| ScriptError::ScriptNotFound(format!("Script ID: {:?}", script_id)))?
            .path
            .display()
            .to_string();

        let path_buf = self.scripts.get(&script_id).unwrap().path.clone();
        let source = std::fs::read_to_string(&path_buf).map_err(ScriptError::Io)?;

        let chunk = self.lua.load(&source);
        let func = match chunk.into_function() {
            Ok(f) => f,
            Err(e) => {
                let stack_trace = self.extract_stack_trace(&e);
                eprintln!(
                    "Failed to compile script on reload: {}\nStack trace:\n{}",
                    e, stack_trace
                );
                return Err(ScriptError::Compilation {
                    script_path: path_str,
                    message: e.to_string(),
                    stack_trace,
                });
            }
        };

        // Execute script body
        let result = match func.call::<_, mlua::Value>(()) {
            Ok(v) => v,
            Err(e) => {
                let stack_trace = self.extract_stack_trace(&e);
                eprintln!(
                    "Failed to execute script body on reload: {}\nStack trace:\n{}",
                    e, stack_trace
                );
                return Err(ScriptError::Runtime {
                    script_path: path_str,
                    message: e.to_string(),
                    stack_trace,
                });
            }
        };

        let new_instance_key = if let mlua::Value::Table(new_table) = result {
            // Check if we have an old instance to preserve state from
            let has_old_instance = self
                .scripts
                .get(&script_id)
                .and_then(|s| s.instance_key.as_ref())
                .is_some();

            if has_old_instance {
                let old_key = self.scripts.get(&script_id).unwrap().instance_key.as_ref().unwrap();
                // Try to preserve state across reload
                match self.lua.registry_value::<mlua::Table>(old_key) {
                    Ok(old_table) => {
                        // Try to save state
                        let saved_state: Option<mlua::Value> =
                            if let Ok(on_save) = old_table.get::<_, mlua::Function>("on_save") {
                                match on_save.call::<_, mlua::Value>(()) {
                                    Ok(state) => Some(state),
                                    Err(e) => {
                                        let stack_trace = self.extract_stack_trace(&e);
                                        eprintln!(
                                            "Error calling on_save during reload:\n{}\nStack trace:\n{}",
                                            e, stack_trace
                                        );
                                        None
                                    }
                                }
                            } else {
                                None
                            };

                        // Copy non-function fields
                        for pair in old_table.pairs::<mlua::Value, mlua::Value>() {
                            if let Ok((k, v)) = pair {
                                if !v.is_function() {
                                    let _ = new_table.set(k, v);
                                }
                            }
                        }

                        // Try to restore state
                        if let Some(state) = saved_state {
                            if let Ok(on_restore) = new_table.get::<_, mlua::Function>("on_restore")
                            {
                                if let Err(e) = on_restore.call::<_, ()>(state) {
                                    let stack_trace = self.extract_stack_trace(&e);
                                    eprintln!(
                                        "Error calling on_restore during reload:\n{}\nStack trace:\n{}",
                                        e, stack_trace
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to get old table during reload: {}", e);
                    }
                }
            }

            match self.lua.create_registry_value(new_table) {
                Ok(key) => Some(key),
                Err(e) => {
                    let stack_trace = self.extract_stack_trace(&e);
                    return Err(ScriptError::Runtime {
                        script_path: path_str.clone(),
                        message: e.to_string(),
                        stack_trace,
                    });
                }
            }
        } else {
            None
        };

        let factory_key = match self.lua.create_registry_value(func) {
            Ok(key) => key,
            Err(e) => {
                let stack_trace = self.extract_stack_trace(&e);
                return Err(ScriptError::Runtime {
                    script_path: path_str,
                    message: e.to_string(),
                    stack_trace,
                });
            }
        };

        // Now update the script
        let script = self.scripts.get_mut(&script_id).unwrap();
        script.factory_key = factory_key;
        script.instance_key = new_instance_key;

        Ok(())
    }

    pub fn call_lifecycle(&self, script_id: ScriptId, hook: &str) -> Result<(), ScriptError> {
        let script = self.scripts.get(&script_id).ok_or_else(|| {
            ScriptError::ScriptNotFound(format!("Script ID: {:?}", script_id))
        })?;

        let path_str = script.path.display().to_string();

        if let Some(key) = &script.instance_key {
            let table: mlua::Table = self.lua.registry_value(key).map_err(|e| {
                let stack_trace = self.extract_stack_trace(&e);
                ScriptError::Runtime {
                    script_path: path_str.clone(),
                    message: format!("Failed to get script instance: {}", e),
                    stack_trace,
                }
            })?;

            if let Ok(func) = table.get::<_, mlua::Function>(hook) {
                // Call directly with explicit type annotation
                match func.call::<_, ()>(()) {
                    Ok(_) => {}
                    Err(e) => {
                        let stack_trace = self.extract_stack_trace(&e);
                        return Err(ScriptError::Runtime {
                            script_path: path_str,
                            message: e.to_string(),
                            stack_trace,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    pub fn update(&mut self, dt: f32, world: &mut World, input: &Input) -> Result<(), ScriptError> {
        let lua_world = LuaWorld(world as *mut _);

        let result: mlua::Result<()> = self.lua.scope(|scope| {
            // Pass references directly. Scope ensures they don't outlive the function call.
            // We cast to raw pointer to bypass static lifetime requirement, relying on scope safety
            let input_ud = scope.create_userdata(LuaInput(input as *const _))?;
            let world_ud = scope.create_userdata(lua_world)?;

            for script in self.scripts.values() {
                if let Some(key) = &script.instance_key {
                    if let Ok(table) = self.lua.registry_value::<mlua::Table>(key) {
                        if let Ok(func) = table.get::<_, mlua::Function>("on_update") {
                            // Isolate errors per script - don't let one script crash others
                            if let Err(e) = func.call::<_, ()>((dt, input_ud.clone(), world_ud.clone())) {
                                let stack_trace = self.extract_stack_trace(&e);
                                eprintln!(
                                    "Error in script {:?} ({}) on_update:\n{}\n\nStack trace:\n{}",
                                    script.id,
                                    script.path.display(),
                                    e,
                                    stack_trace
                                );
                                // Continue processing other scripts instead of propagating error
                            }
                        }
                    }
                }
            }
            Ok(())
        });

        result.map_err(|e| {
            let stack_trace = self.extract_stack_trace(&e);
            ScriptError::Runtime {
                script_path: "update loop".to_string(),
                message: e.to_string(),
                stack_trace,
            }
        })
    }
}
