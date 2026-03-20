# Example LUT Plugin

This is a minimal example plugin written in WebAssembly Text Format (WAT).

## Building

The WAT source is compiled to WASM automatically by the test suite using the `wat` crate.

To manually compile:

```bash
# Using wat2wasm from WABT tools
wat2wasm plugin.wat -o plugin.wasm
```

## API

- `process_image(data_ptr: i32, width: i32, height: i32) -> i32`: Process image data (identity function for now)
- `get_version() -> i32`: Return plugin API version (1)

## Permissions

- `read_image`: true
- `write_image`: true
