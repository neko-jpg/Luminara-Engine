# WASM Scripting Guide

## Interface

Defined in `luminara-guest.wit`.

```wit
interface host {
    get-position: func(entity: u64) -> tuple<float32, float32, float32>;
    set-position: func(entity: u64, x: float32, y: float32, z: float32);
    log: func(msg: string);
}
```

## Rust Guest Example

```rust
#[no_mangle]
pub extern "C" fn on_update() {
    // Call host functions
}
```
