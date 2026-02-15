use luminara_script_wasm::{ResourceLimits, WasmScriptRuntime};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[quickcheck]
fn test_wasm_validation(invalid_bytes: Vec<u8>) -> TestResult {
    if invalid_bytes.starts_with(b"\0asm") {
        return TestResult::discard();
    }

    let mut runtime = WasmScriptRuntime::new(ResourceLimits::default()).unwrap();
    let result = runtime.load_module(&invalid_bytes);

    TestResult::from_bool(result.is_err())
}

#[test]
fn test_marshaling_round_trip() {
    let limits = ResourceLimits {
        max_instructions: 10000,
        max_memory: 1024 * 64,
        max_execution_time: std::time::Duration::from_secs(1),
    };
    let mut runtime = WasmScriptRuntime::new(limits).unwrap();

    let wat = r#"
        (module
            (memory (export "memory") 1)
            (func (export "alloc") (param i32) (result i32)
                i32.const 0
            )
            (func (export "echo_json") (param i32 i32) (result i32 i32)
                local.get 0
                local.get 1
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let id = runtime.load_module(&wasm).unwrap();

    let input = "Hello WASM";
    let output: String = runtime.call_json_func(id, "echo_json", input).unwrap();

    assert_eq!(input, output);
}

#[test]
fn test_wasm_error_isolation() {
    let mut runtime = WasmScriptRuntime::new(ResourceLimits::default()).unwrap();

    let wat_trap = r#"
        (module
            (memory (export "memory") 1)
            (func (export "alloc") (param i32) (result i32) i32.const 0)
            (func (export "do_trap") (param i32 i32) (result i32 i32)
                unreachable
            )
        )
    "#;
    let wasm_trap = wat::parse_str(wat_trap).unwrap();
    let id_trap = runtime.load_module(&wasm_trap).unwrap();

    let result: Result<String, _> = runtime.call_json_func(id_trap, "do_trap", "input");

    assert!(result.is_err());

    let wat_echo = r#"
        (module
            (memory (export "memory") 1)
            (func (export "alloc") (param i32) (result i32) i32.const 0)
            (func (export "echo") (param i32 i32) (result i32 i32) local.get 0 local.get 1)
        )
    "#;
    let wasm_echo = wat::parse_str(wat_echo).unwrap();
    let id_echo = runtime.load_module(&wasm_echo).unwrap();
    let res: String = runtime.call_json_func(id_echo, "echo", "ok").unwrap();
    assert_eq!(res, "ok");
}
