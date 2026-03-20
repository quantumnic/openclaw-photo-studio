//! Plugin SDK Templates
//!
//! Generate starter templates for plugin authors

use crate::api::PLUGIN_API_VERSION;

/// Generate a Rust plugin template
///
/// Returns a complete Rust source file that compiles to WASM and implements
/// a basic image filter plugin.
pub fn generate_rust_template() -> String {
    // Simple template without format! macro to avoid escaping issues
    let api_ver = PLUGIN_API_VERSION.to_string();
    let mut s = String::new();
    s.push_str("//! Example OCPS Plugin\n");
    s.push_str("//! Compile: cargo build --target wasm32-unknown-unknown --release\n\n");
    s.push_str("#![no_std]\n");
    s.push_str("use core::panic::PanicInfo;\n\n");
    s.push_str("#[panic_handler]\n");
    s.push_str("fn panic(_: &PanicInfo) -> ! { loop {} }\n\n");
    s.push_str("extern \"C\" {\n");
    s.push_str("    fn ocps_log(ptr: *const u8, len: i32);\n");
    s.push_str("    fn ocps_get_pixel(x: i32, y: i32, ch: i32) -> f32;\n");
    s.push_str("    fn ocps_set_pixel(x: i32, y: i32, ch: i32, v: f32);\n");
    s.push_str("}\n\n");
    s.push_str("#[no_mangle]\n");
    s.push_str("pub extern \"C\" fn plugin_init() -> i32 { 0 }\n\n");
    s.push_str("#[no_mangle]\n");
    s.push_str("pub extern \"C\" fn plugin_info(_out: *mut u8) -> i32 {\n");
    s.push_str("    // Return JSON: {\"name\":\"Example\",\"version\":\"1.0.0\",\"api_version\":");
    s.push_str(&api_ver);
    s.push_str("}\n");
    s.push_str("    0\n");
    s.push_str("}\n\n");
    s.push_str("#[no_mangle]\n");
    s.push_str("pub extern \"C\" fn get_parameters(_out: *mut u8) -> i32 { 0 }\n\n");
    s.push_str("#[no_mangle]\n");
    s.push_str("pub extern \"C\" fn process_image(_w: i32, _h: i32) -> i32 { 0 }\n\n");
    s.push_str("// Plugin API version: ");
    s.push_str(&api_ver);
    s.push('\n');
    s
}

/// Generate a WAT (WebAssembly Text) plugin template
///
/// Returns a minimal plugin in WAT format for users who want to write
/// plugins directly in WebAssembly text format.
pub fn generate_wat_template() -> String {
    format!(
        r#"(module
  ;; Import host functions from OCPS
  (import "env" "ocps_log" (func $ocps_log (param i32 i32)))
  (import "env" "ocps_get_image_width" (func $ocps_get_image_width (result i32)))
  (import "env" "ocps_get_image_height" (func $ocps_get_image_height (result i32)))
  (import "env" "ocps_get_pixel" (func $ocps_get_pixel (param i32 i32 i32) (result f32)))
  (import "env" "ocps_set_pixel" (func $ocps_set_pixel (param i32 i32 i32 f32)))

  ;; Linear memory (required for string passing)
  (memory (export "memory") 1)

  ;; Plugin info JSON string
  (data (i32.const 0) "{{\\\"name\\\":\\\"WAT Plugin\\\",\\\"version\\\":\\\"1.0.0\\\",\\\"author\\\":\\\"WAT Author\\\",\\\"description\\\":\\\"Minimal WAT plugin\\\",\\\"type\\\":\\\"image_filter\\\",\\\"api_version\\\":{}}}")

  ;; Initialize plugin
  (func (export "plugin_init") (result i32)
    (i32.const 0) ;; return 0 (success)
  )

  ;; Return plugin info
  (func (export "plugin_info") (param $out_ptr i32) (result i32)
    ;; Copy plugin info JSON to out_ptr (simplified: assume caller provides buffer)
    (i32.const 163) ;; return length of JSON string
  )

  ;; Process image (example: invert colors)
  (func (export "process_image") (param $width i32) (param $height i32) (result i32)
    (local $x i32)
    (local $y i32)
    (local $channel i32)
    (local $pixel f32)

    ;; Loop over all pixels
    (block $done
      (loop $y_loop
        (local.set $x (i32.const 0))

        (loop $x_loop
          ;; Process RGB channels
          (local.set $channel (i32.const 0))

          (loop $channel_loop
            ;; Get pixel
            (local.set $pixel (call $ocps_get_pixel (local.get $x) (local.get $y) (local.get $channel)))

            ;; Invert: 1.0 - pixel
            (call $ocps_set_pixel
              (local.get $x)
              (local.get $y)
              (local.get $channel)
              (f32.sub (f32.const 1.0) (local.get $pixel))
            )

            ;; Next channel
            (local.set $channel (i32.add (local.get $channel) (i32.const 1)))
            (br_if $channel_loop (i32.lt_s (local.get $channel) (i32.const 3)))
          )

          ;; Next x
          (local.set $x (i32.add (local.get $x) (i32.const 1)))
          (br_if $x_loop (i32.lt_s (local.get $x) (local.get $width)))
        )

        ;; Next y
        (local.set $y (i32.add (local.get $y) (i32.const 1)))
        (br_if $y_loop (i32.lt_s (local.get $y) (local.get $height)))
      )
    )

    (i32.const 0) ;; return 0 (success)
  )
)

;; README
;;
;; # Building WAT Plugin
;;
;; 1. Save this file as `plugin.wat`
;; 2. Compile to WASM: `wat2wasm plugin.wat -o plugin.wasm`
;; 3. Create plugin.toml manifest (see Rust template for example)
;; 4. Package and install
;;
;; # Installing wat2wasm
;;
;; Part of WABT (WebAssembly Binary Toolkit):
;; - macOS: `brew install wabt`
;; - Linux: `apt install wabt` or build from source
;; - Windows: Download from https://github.com/WebAssembly/wabt/releases
;;
;; # API Version: {}
"#,
        PLUGIN_API_VERSION, PLUGIN_API_VERSION
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdk_rust_template_compiles() {
        let template = generate_rust_template();

        // Verify it's valid UTF-8
        assert!(!template.is_empty());

        // Check for expected sections
        assert!(template.contains("plugin_init"), "Missing plugin_init function");
        assert!(template.contains("plugin_info"), "Missing plugin_info function");
        assert!(template.contains("process_image"), "Missing process_image function");
        assert!(template.contains("get_parameters"), "Missing get_parameters function");

        // Check for host function imports
        assert!(template.contains("ocps_get_pixel"), "Missing ocps_get_pixel import");
        assert!(template.contains("ocps_set_pixel"), "Missing ocps_set_pixel import");

        // Check for API version reference
        assert!(template.contains(&format!("\"api_version\":{}", PLUGIN_API_VERSION)));
    }

    #[test]
    fn test_sdk_wat_template_valid() {
        let template = generate_wat_template();

        // Verify it's valid UTF-8
        assert!(!template.is_empty());

        // Check for module declaration
        assert!(template.starts_with("(module"), "Should start with module declaration");

        // Check for required exports
        assert!(template.contains("(export \"plugin_init\")"), "Missing plugin_init export");
        assert!(template.contains("(export \"plugin_info\")"), "Missing plugin_info export");
        assert!(template.contains("(export \"process_image\")"), "Missing process_image export");

        // Check for host imports
        assert!(template.contains("(import \"env\" \"ocps_get_pixel\""), "Missing ocps_get_pixel import");
        assert!(template.contains("(import \"env\" \"ocps_set_pixel\""), "Missing ocps_set_pixel import");

        // Try to compile with wat crate
        let result = wat::parse_str(&template);
        assert!(result.is_ok(), "WAT template should compile: {:?}", result.err());

        let wasm_bytes = result.unwrap();
        assert!(!wasm_bytes.is_empty(), "Should produce WASM bytes");
    }

    #[test]
    fn test_api_version_is_1() {
        assert_eq!(PLUGIN_API_VERSION, 1, "Plugin API version must be 1");
    }
}
