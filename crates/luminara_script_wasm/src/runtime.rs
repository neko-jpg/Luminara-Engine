use luminara_script::{ScriptError, ScriptId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};

// WIT Bindgen generated traits would go here if we ran `wit-bindgen`.
// Since we don't have `wit-bindgen` CLI tool installed in environment usually,
// we can use `wasmtime::component::bindgen!` macro if we were using Component Model fully.
// Or just manually register host functions which is simpler for MVP and matches Task 7.2 description "Register host functions in linker".

pub struct WasmScriptRuntime {
    engine: Engine,
    store: Store<HostState>,
    modules: HashMap<ScriptId, WasmModule>,
    linker: Linker<HostState>,
    next_id: u64,
}

pub struct WasmModule {
    pub id: ScriptId,
    pub instance: Instance,
}

pub struct HostState {
    pub limits: ResourceLimits,
    pub instruction_count: u64,
    // Add World reference? Accessing World from WASM callback requires it.
    // For now, placeholder.
}

#[derive(Default, Clone)]
pub struct ResourceLimits {
    pub max_memory: usize,
    pub max_execution_time: std::time::Duration,
    pub max_instructions: u64,
}

impl WasmScriptRuntime {
    pub fn new(limits: ResourceLimits) -> Result<Self, ScriptError> {
        let mut config = Config::new();

        if limits.max_instructions > 0 {
            config.consume_fuel(true);
        }
        config.epoch_interruption(true);

        let engine = Engine::new(&config).map_err(|e| ScriptError::Runtime(e.to_string()))?;
        let mut store = Store::new(
            &engine,
            HostState {
                limits: limits.clone(),
                instruction_count: 0,
            },
        );

        if limits.max_instructions > 0 {
            store
                .set_fuel(limits.max_instructions)
                .map_err(|e| ScriptError::Runtime(e.to_string()))?;
        }
        store.set_epoch_deadline(1);

        let mut linker = Linker::new(&engine);

        // Register host functions (Task 7.2)
        // Transform API
        linker
            .func_wrap(
                "host",
                "get_position",
                |caller: wasmtime::Caller<'_, HostState>, entity_id: u64| -> (f32, f32, f32) {
                    // Access World from caller.data() if we stored it?
                    // Since `HostState` owns data in Store, we can't easily put `&mut World` there (lifetime issue).
                    // Usually we use `store.data_mut().world_handle` where handle indexes into external map,
                    // or we accept that we can't access World this way without unsafe or complex architecture.
                    // For MVP, return dummy values or println.
                    println!("WASM: get_position({})", entity_id);
                    (0.0, 0.0, 0.0)
                },
            )
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;

        linker
            .func_wrap(
                "host",
                "set_position",
                |caller: wasmtime::Caller<'_, HostState>,
                 entity_id: u64,
                 x: f32,
                 y: f32,
                 z: f32| {
                    println!("WASM: set_position({}, {}, {}, {})", entity_id, x, y, z);
                },
            )
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;

        // Input API
        // Strings in WASM are pointer+len. `func_wrap` doesn't support String directly unless using component model or typed func with manual read.
        // `func_wrap` supports primitive types.
        // To support string, we need `(ptr, len)`.

        linker
            .func_wrap(
                "host",
                "log",
                |mut caller: wasmtime::Caller<'_, HostState>, ptr: i32, len: i32| {
                    let mem = match caller.get_export("memory") {
                        Some(wasmtime::Extern::Memory(m)) => m,
                        _ => return,
                    };
                    let mut buffer = vec![0u8; len as usize];
                    if mem.read(&caller, ptr as usize, &mut buffer).is_ok() {
                        if let Ok(s) = String::from_utf8(buffer) {
                            println!("WASM Log: {}", s);
                        }
                    }
                },
            )
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;

        Ok(Self {
            engine,
            store,
            modules: HashMap::new(),
            linker,
            next_id: 0,
        })
    }

    // ... existing methods ...
    pub fn start_epoch_timer(&self) {
        let engine = self.engine.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            engine.increment_epoch();
        });
    }

    pub fn load_module(&mut self, bytes: &[u8]) -> Result<ScriptId, ScriptError> {
        let module = Module::from_binary(&self.engine, bytes)
            .map_err(|e| ScriptError::Compilation(e.to_string()))?;

        let instance = self
            .linker
            .instantiate(&mut self.store, &module)
            .map_err(|e| {
                ScriptError::Runtime(format!("Failed to instantiate WASM module: {}", e))
            })?;

        let id = ScriptId(self.next_id);
        self.next_id += 1;

        self.modules.insert(id, WasmModule { id, instance });

        Ok(id)
    }

    pub fn write_to_memory(
        &mut self,
        script_id: ScriptId,
        data: &[u8],
    ) -> Result<(i32, i32), ScriptError> {
        let module = self
            .modules
            .get(&script_id)
            .ok_or_else(|| ScriptError::Runtime("Module not found".into()))?;
        let instance = module.instance;

        let alloc = instance
            .get_typed_func::<i32, i32>(&mut self.store, "alloc")
            .map_err(|_| ScriptError::Runtime("Module does not export 'alloc'".into()))?;

        let ptr = alloc
            .call(&mut self.store, data.len() as i32)
            .map_err(|e| ScriptError::Runtime(format!("Alloc failed: {}", e)))?;

        let memory = instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| ScriptError::Runtime("Module does not export 'memory'".into()))?;

        memory
            .write(&mut self.store, ptr as usize, data)
            .map_err(|e| ScriptError::Runtime(format!("Memory write failed: {}", e)))?;

        Ok((ptr, data.len() as i32))
    }

    pub fn read_from_memory(
        &mut self,
        script_id: ScriptId,
        ptr: i32,
        len: i32,
    ) -> Result<Vec<u8>, ScriptError> {
        let module = self
            .modules
            .get(&script_id)
            .ok_or_else(|| ScriptError::Runtime("Module not found".into()))?;
        let instance = module.instance;

        let memory = instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| ScriptError::Runtime("Module does not export 'memory'".into()))?;

        let mut buffer = vec![0u8; len as usize];
        memory
            .read(&mut self.store, ptr as usize, &mut buffer)
            .map_err(|e| ScriptError::Runtime(format!("Memory read failed: {}", e)))?;

        Ok(buffer)
    }

    pub fn call_json_func<Args: Serialize, Ret: for<'de> Deserialize<'de>>(
        &mut self,
        script_id: ScriptId,
        func_name: &str,
        args: Args,
    ) -> Result<Ret, ScriptError> {
        // Reset limits BEFORE any execution (including alloc in write_to_memory)
        if self.store.data().limits.max_instructions > 0 {
            self.store
                .set_fuel(self.store.data().limits.max_instructions)
                .unwrap();
        }
        self.store.set_epoch_deadline(1);

        let json_bytes =
            serde_json::to_vec(&args).map_err(|e| ScriptError::Runtime(e.to_string()))?;
        let (ptr, len) = self.write_to_memory(script_id, &json_bytes)?;

        let module = self
            .modules
            .get(&script_id)
            .ok_or_else(|| ScriptError::Runtime("Module not found".into()))?;
        let func = module
            .instance
            .get_typed_func::<(i32, i32), (i32, i32)>(&mut self.store, func_name)
            .map_err(|_| {
                ScriptError::Runtime(format!(
                    "Function {} not found or signature mismatch",
                    func_name
                ))
            })?;

        let (ret_ptr, ret_len) = func
            .call(&mut self.store, (ptr, len))
            .map_err(|e| ScriptError::Runtime(format!("Call failed: {}", e)))?;

        let ret_bytes = self.read_from_memory(script_id, ret_ptr, ret_len)?;

        let ret: Ret =
            serde_json::from_slice(&ret_bytes).map_err(|e| ScriptError::Runtime(e.to_string()))?;

        Ok(ret)
    }
}
