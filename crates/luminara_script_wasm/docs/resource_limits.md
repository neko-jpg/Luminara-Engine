# WASM Resource Limits

## Overview

The WASM script runtime enforces resource limits to prevent runaway execution and ensure system stability. This document describes the resource limit configuration and enforcement mechanisms.

## Resource Limit Types

### 1. Memory Limits

**Purpose**: Prevent excessive memory allocation that could exhaust system resources.

**Configuration**: `ResourceLimits::max_memory` (in bytes)

**Enforcement**: 
- Wasmtime enforces memory limits at the module level
- Memory growth operations (`memory.grow`) are restricted
- Attempts to exceed the limit will fail gracefully

**Default**: No limit (0 = unlimited)

**Recommended Values**:
- Development: 64 MB (1024 * 1024 * 64)
- Production: 16 MB (1024 * 1024 * 16)
- Strict sandbox: 1 MB (1024 * 1024)

### 2. Instruction Count Limits

**Purpose**: Prevent infinite loops and runaway computation.

**Configuration**: `ResourceLimits::max_instructions` (instruction count)

**Enforcement**:
- Uses Wasmtime's "fuel" mechanism
- Each WASM instruction consumes fuel
- When fuel is exhausted, execution is terminated with an error
- Fuel is reset before each function call

**Default**: No limit (0 = unlimited)

**Recommended Values**:
- Development: 10,000,000 instructions
- Production: 1,000,000 instructions
- Strict sandbox: 100,000 instructions

**Note**: Instruction count is approximate and depends on WASM instruction complexity.

### 3. Execution Time Limits

**Purpose**: Timeout long-running scripts to prevent blocking.

**Configuration**: `ResourceLimits::max_execution_time` (Duration)

**Enforcement**:
- Uses Wasmtime's epoch-based interruption
- A background thread increments the epoch counter every 100ms
- Scripts are interrupted when the epoch deadline is exceeded
- Requires calling `runtime.start_epoch_timer()` to enable

**Default**: No limit (Duration::ZERO = unlimited)

**Recommended Values**:
- Development: 5 seconds
- Production: 1 second
- Strict sandbox: 100 milliseconds

**Important**: The epoch timer must be started explicitly:
```rust
runtime.start_epoch_timer();
```

## Configuration Examples

### Development Configuration (Permissive)

```rust
use luminara_script_wasm::ResourceLimits;
use std::time::Duration;

let limits = ResourceLimits {
    max_instructions: 10_000_000,
    max_memory: 1024 * 1024 * 64, // 64 MB
    max_execution_time: Duration::from_secs(5),
};
```

### Production Configuration (Balanced)

```rust
let limits = ResourceLimits {
    max_instructions: 1_000_000,
    max_memory: 1024 * 1024 * 16, // 16 MB
    max_execution_time: Duration::from_secs(1),
};
```

### Strict Sandbox Configuration (Restrictive)

```rust
let limits = ResourceLimits {
    max_instructions: 100_000,
    max_memory: 1024 * 1024, // 1 MB
    max_execution_time: Duration::from_millis(100),
};
```

### No Limits (Testing Only)

```rust
let limits = ResourceLimits::default();
// All limits disabled - use only for testing!
```

## Usage

### Basic Setup

```rust
use luminara_script_wasm::{ResourceLimits, WasmScriptRuntime};
use std::time::Duration;

// Configure limits
let limits = ResourceLimits {
    max_instructions: 1_000_000,
    max_memory: 1024 * 1024 * 16,
    max_execution_time: Duration::from_secs(1),
};

// Create runtime with limits
let mut runtime = WasmScriptRuntime::new(limits)?;

// Start epoch timer for time-based limits
runtime.start_epoch_timer();

// Load and execute scripts
let script_id = runtime.load_module(&wasm_bytes)?;
let result: String = runtime.call_json_func(script_id, "my_function", "input")?;
```

### Handling Limit Violations

When a resource limit is exceeded, the runtime returns a `ScriptError::Runtime` error:

```rust
match runtime.call_json_func(script_id, "function", "input") {
    Ok(result) => {
        // Script executed successfully
        println!("Result: {:?}", result);
    }
    Err(e) => {
        // Script exceeded limits or encountered an error
        eprintln!("Script error: {}", e);
        
        // Runtime remains functional - can execute other scripts
        // or retry with different parameters
    }
}
```

## Limit Enforcement Details

### Memory Limit Enforcement

- Enforced at WASM module instantiation time
- Wasmtime configures the maximum memory size
- `memory.grow` operations fail when limit is reached
- No runtime overhead - enforced by the WASM runtime

### Instruction Limit Enforcement

- Fuel is set before each function call
- Each WASM instruction consumes fuel
- When fuel reaches zero, execution is trapped
- Small runtime overhead (~5-10% depending on workload)

### Time Limit Enforcement

- Epoch counter is incremented every 100ms by background thread
- Scripts check epoch deadline at loop back-edges and function calls
- When deadline is exceeded, execution is trapped
- Minimal runtime overhead (~1-2%)

## Isolation and Safety

### Script Isolation

- Each script module has independent memory
- Resource limits are enforced per-call, not per-module
- One script exceeding limits does not affect others
- Runtime remains functional after limit violations

### Sandbox Restrictions

WASM scripts are sandboxed and cannot:
- Access the file system directly
- Make network requests directly
- Access system resources directly
- Execute arbitrary native code

Scripts can only:
- Perform computation within their memory
- Call host functions explicitly provided by the runtime
- Allocate memory within configured limits

### Host Function Safety

Host functions (provided by the engine) should:
- Validate all parameters
- Enforce their own resource limits
- Handle errors gracefully
- Not expose unsafe operations

## Performance Considerations

### Overhead

- Memory limits: No overhead (enforced by WASM runtime)
- Instruction limits: ~5-10% overhead (fuel consumption)
- Time limits: ~1-2% overhead (epoch checking)

### Optimization Tips

1. **Tune instruction limits**: Profile your scripts to find appropriate limits
2. **Use time limits as backup**: Instruction limits are more precise
3. **Batch operations**: Reduce function call overhead by batching work
4. **Minimize host calls**: Host function calls have overhead

### Benchmarking

To measure the overhead of resource limits:

```rust
// Without limits
let limits_none = ResourceLimits::default();
let mut runtime_none = WasmScriptRuntime::new(limits_none)?;

// With limits
let limits_full = ResourceLimits {
    max_instructions: 1_000_000,
    max_memory: 1024 * 1024 * 16,
    max_execution_time: Duration::from_secs(1),
};
let mut runtime_full = WasmScriptRuntime::new(limits_full)?;
runtime_full.start_epoch_timer();

// Compare execution times
```

## Testing

The resource limit enforcement is tested in `tests/resource_limits_test.rs`:

- `test_memory_limit_enforcement`: Verifies memory limits prevent excessive allocation
- `test_instruction_limit_enforcement`: Verifies instruction limits prevent infinite loops
- `test_execution_time_limit_enforcement`: Verifies time limits timeout long-running scripts
- `test_sandbox_restrictions_basic`: Verifies basic sandbox restrictions
- `test_script_termination_on_limit_exceeded`: Verifies proper termination and recovery
- `test_multiple_scripts_isolation`: Verifies script isolation
- `test_resource_limit_configuration`: Verifies configuration options

Run tests with:
```bash
cargo test -p luminara_script_wasm
```

## Troubleshooting

### Script Fails with "out of fuel" Error

**Cause**: Script exceeded instruction count limit

**Solutions**:
- Increase `max_instructions` limit
- Optimize script to use fewer instructions
- Break work into smaller chunks

### Script Fails with Timeout Error

**Cause**: Script exceeded execution time limit

**Solutions**:
- Increase `max_execution_time` limit
- Optimize script performance
- Ensure epoch timer is started: `runtime.start_epoch_timer()`

### Memory Allocation Fails

**Cause**: Script exceeded memory limit

**Solutions**:
- Increase `max_memory` limit
- Optimize script memory usage
- Use streaming or chunked processing

### Epoch Timer Not Working

**Cause**: Epoch timer not started

**Solution**: Call `runtime.start_epoch_timer()` after creating the runtime

## Future Enhancements

Potential improvements to resource limit enforcement:

1. **Per-module memory limits**: Currently memory limits are global
2. **CPU time limits**: More accurate than instruction counting
3. **I/O operation limits**: Limit host function calls
4. **Dynamic limit adjustment**: Adjust limits based on system load
5. **Limit profiles**: Predefined limit configurations for common scenarios
6. **Monitoring and metrics**: Track resource usage over time

## References

- [Wasmtime Fuel Documentation](https://docs.wasmtime.dev/api/wasmtime/struct.Store.html#method.add_fuel)
- [Wasmtime Epoch Interruption](https://docs.wasmtime.dev/api/wasmtime/struct.Store.html#method.set_epoch_deadline)
- [WebAssembly Memory Model](https://webassembly.github.io/spec/core/syntax/modules.html#memories)
