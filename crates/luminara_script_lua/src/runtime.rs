use crate::api::{input::LuaInput, world::LuaWorld};
use luminara_core::world::World;
use luminara_input::Input;
use luminara_script::{ScriptError, ScriptId};
use mlua::prelude::*;
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

    pub fn load_script(&mut self, path: &Path) -> Result<ScriptId, ScriptError> {
        let path_buf = path.to_path_buf();

        if let Some(&id) = self.path_to_id.get(&path_buf) {
            return Ok(id);
        }

        let source = std::fs::read_to_string(path).map_err(ScriptError::Io)?;

        let chunk = self.lua.load(&source);
        let func = chunk
            .into_function()
            .map_err(|e| ScriptError::Compilation(e.to_string()))?;
        let factory_key = self
            .lua
            .create_registry_value(func.clone())
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;

        let result: mlua::Value = func
            .call(())
            .map_err(|e| ScriptError::Runtime(format!("Error running script body: {}", e)))?;

        let instance_key = if let mlua::Value::Table(t) = result {
            Some(
                self.lua
                    .create_registry_value(t)
                    .map_err(|e| ScriptError::Runtime(e.to_string()))?,
            )
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
        let script = self
            .scripts
            .get_mut(&script_id)
            .ok_or_else(|| ScriptError::Runtime(format!("Script not found: {:?}", script_id)))?;

        let source = std::fs::read_to_string(&script.path).map_err(ScriptError::Io)?;

        let chunk = self.lua.load(&source);
        let func = match chunk.into_function() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to compile script on reload: {}", e);
                return Err(ScriptError::Compilation(e.to_string()));
            }
        };

        let result: mlua::Result<mlua::Value> = func.call(());
        let result = match result {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to execute script body on reload: {}", e);
                return Err(ScriptError::Runtime(e.to_string()));
            }
        };

        let new_instance_key = if let mlua::Value::Table(new_table) = result {
            if let Some(old_key) = &script.instance_key {
                let old_table: mlua::Table = self
                    .lua
                    .registry_value(old_key)
                    .map_err(|e| ScriptError::Runtime(e.to_string()))?;

                let saved_state: Option<mlua::Value> =
                    if let Ok(on_save) = old_table.get::<_, mlua::Function>("on_save") {
                        Some(
                            on_save
                                .call::<_, mlua::Value>(())
                                .map_err(|e| ScriptError::Runtime(e.to_string()))?,
                        )
                    } else {
                        None
                    };

                for pair in old_table.pairs::<mlua::Value, mlua::Value>() {
                    if let Ok((k, v)) = pair {
                        if !v.is_function() {
                            let _ = new_table.set(k, v);
                        }
                    }
                }

                if let Some(state) = saved_state {
                    if let Ok(on_restore) = new_table.get::<_, mlua::Function>("on_restore") {
                        let _ = on_restore.call::<_, ()>(state);
                    }
                }
            }

            Some(
                self.lua
                    .create_registry_value(new_table)
                    .map_err(|e| ScriptError::Runtime(e.to_string()))?,
            )
        } else {
            None
        };

        script.factory_key = self
            .lua
            .create_registry_value(func)
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;
        script.instance_key = new_instance_key;

        Ok(())
    }

    pub fn call_lifecycle(&self, script_id: ScriptId, hook: &str) -> Result<(), ScriptError> {
        let script = self
            .scripts
            .get(&script_id)
            .ok_or_else(|| ScriptError::Runtime(format!("Script not found: {:?}", script_id)))?;

        if let Some(key) = &script.instance_key {
            let table: mlua::Table = self
                .lua
                .registry_value(key)
                .map_err(|e| ScriptError::Runtime(e.to_string()))?;

            if let Ok(func) = table.get::<_, mlua::Function>(hook) {
                func.call::<_, ()>(()).map_err(|e| {
                    ScriptError::Runtime(format!("Error calling hook '{}': {}", hook, e))
                })?;
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
                            if let Err(e) =
                                func.call::<_, ()>((dt, input_ud.clone(), world_ud.clone()))
                            {
                                eprintln!("Error in script {:?} on_update: {}", script.id, e);
                            }
                        }
                    }
                }
            }
            Ok(())
        });

        result.map_err(|e| ScriptError::Runtime(e.to_string()))
    }
}
