//! WASM plugin host implementation
//!
//! Provides safe, sandboxed plugin execution via wasmtime.

use crate::{PluginError, PluginManifest};
use std::collections::HashMap;
use std::path::Path;

/// Plugin permissions
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PluginPermissions {
    pub read_image: bool,
    pub write_image: bool,
    pub read_catalog: bool,
    pub write_catalog: bool,
    pub network: bool,
    pub filesystem: bool,
}

/// WASM plugin host
pub struct PluginHost {
    engine: wasmtime::Engine,
    plugins: HashMap<String, LoadedPlugin>,
}

/// A loaded plugin instance
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub store: wasmtime::Store<PluginState>,
    pub instance: wasmtime::Instance,
}

/// Plugin execution state
pub struct PluginState {
    pub plugin_id: String,
    pub permissions: PluginPermissions,
    pub input_image: Option<Vec<u8>>,
    pub output_image: Option<Vec<u8>>,
}

impl PluginHost {
    /// Create a new plugin host
    pub fn new() -> Result<Self, PluginError> {
        let config = wasmtime::Config::new();
        let engine = wasmtime::Engine::new(&config)
            .map_err(|e| PluginError::WasmError(e.to_string()))?;
        Ok(Self {
            engine,
            plugins: HashMap::new(),
        })
    }

    /// Load a plugin from a WASM file
    pub fn load_plugin(
        &mut self,
        manifest: PluginManifest,
        wasm_path: &Path,
    ) -> Result<(), PluginError> {
        let wasm_bytes = std::fs::read(wasm_path)?;
        let module = wasmtime::Module::new(&self.engine, &wasm_bytes)
            .map_err(|e| PluginError::WasmError(e.to_string()))?;

        let mut store = wasmtime::Store::new(
            &self.engine,
            PluginState {
                plugin_id: manifest.id.clone(),
                permissions: PluginPermissions::default(),
                input_image: None,
                output_image: None,
            },
        );

        // Create linker with host functions
        let linker = Self::create_linker(&self.engine, &manifest)?;
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| PluginError::WasmError(e.to_string()))?;

        self.plugins.insert(
            manifest.id.clone(),
            LoadedPlugin {
                manifest,
                store,
                instance,
            },
        );
        Ok(())
    }

    /// Create linker with host functions that plugins can call
    fn create_linker(
        engine: &wasmtime::Engine,
        _manifest: &PluginManifest,
    ) -> Result<wasmtime::Linker<PluginState>, PluginError> {
        let mut linker = wasmtime::Linker::new(engine);

        // Host function: ocps_log(msg_ptr, msg_len)
        // Allows plugins to log messages
        linker
            .func_wrap(
                "env",
                "ocps_log",
                |mut caller: wasmtime::Caller<PluginState>, ptr: u32, len: u32| {
                    if let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                        let data = mem.data(&caller);
                        if let Some(bytes) = data.get(ptr as usize..(ptr + len) as usize) {
                            if let Ok(msg) = std::str::from_utf8(bytes) {
                                println!("[plugin:{}] {}", caller.data().plugin_id, msg);
                            }
                        }
                    }
                },
            )
            .map_err(|e| PluginError::WasmError(e.to_string()))?;

        Ok(linker)
    }

    /// Call a function exported by a plugin
    pub fn call_plugin_function(
        &mut self,
        plugin_id: &str,
        function: &str,
        args: &[wasmtime::Val],
    ) -> Result<Vec<wasmtime::Val>, PluginError> {
        let plugin = self
            .plugins
            .get_mut(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;

        let func = plugin
            .instance
            .get_func(&mut plugin.store, function)
            .ok_or_else(|| PluginError::FunctionNotFound(function.to_string()))?;

        let mut results = vec![wasmtime::Val::I32(0); func.ty(&plugin.store).results().len()];
        func.call(&mut plugin.store, args, &mut results)
            .map_err(|e| PluginError::WasmError(e.to_string()))?;

        Ok(results)
    }

    /// Get list of loaded plugin IDs
    pub fn list_plugins(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a plugin is loaded
    pub fn has_plugin(&self, plugin_id: &str) -> bool {
        self.plugins.contains_key(plugin_id)
    }

    /// Unload a plugin
    pub fn unload_plugin(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        self.plugins
            .remove(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;
        Ok(())
    }
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new().expect("Failed to create default PluginHost")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_host_new() {
        let host = PluginHost::new();
        assert!(host.is_ok());
        let host = host.unwrap();
        assert_eq!(host.list_plugins().len(), 0);
    }

    #[test]
    fn test_load_wat_module() {
        let mut host = PluginHost::new().unwrap();

        // Compile WAT to WASM bytes
        let wat = r#"
            (module
              (import "env" "ocps_log" (func $log (param i32 i32)))
              (memory (export "memory") 1)

              (func (export "get_version") (result i32)
                (i32.const 1)
              )

              (data (i32.const 0) "test-plugin")
            )
        "#;

        let wasm_bytes = wat::parse_str(wat).expect("Failed to compile WAT");

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let wasm_path = temp_dir.join("test_plugin.wasm");
        std::fs::write(&wasm_path, wasm_bytes).unwrap();

        let manifest = PluginManifest {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            api_version: "1".to_string(),
            plugin_type: "image_filter".to_string(),
            author: "Test".to_string(),
            description: "Test plugin".to_string(),
            entry_point: "test_plugin.wasm".to_string(),
        };

        let result = host.load_plugin(manifest, &wasm_path);
        assert!(result.is_ok());
        assert!(host.has_plugin("test-plugin"));

        // Clean up
        std::fs::remove_file(wasm_path).ok();
    }

    #[test]
    fn test_call_get_version() {
        let mut host = PluginHost::new().unwrap();

        let wat = r#"
            (module
              (import "env" "ocps_log" (func $log (param i32 i32)))
              (memory (export "memory") 1)

              (func (export "get_version") (result i32)
                (i32.const 42)
              )
            )
        "#;

        let wasm_bytes = wat::parse_str(wat).unwrap();
        let temp_dir = std::env::temp_dir();
        let wasm_path = temp_dir.join("version_test.wasm");
        std::fs::write(&wasm_path, wasm_bytes).unwrap();

        let manifest = PluginManifest {
            id: "version-test".to_string(),
            name: "Version Test".to_string(),
            version: "1.0.0".to_string(),
            api_version: "1".to_string(),
            plugin_type: "image_filter".to_string(),
            author: "Test".to_string(),
            description: "Version test".to_string(),
            entry_point: "version_test.wasm".to_string(),
        };

        host.load_plugin(manifest, &wasm_path).unwrap();

        let results = host.call_plugin_function("version-test", "get_version", &[]);
        assert!(results.is_ok());

        let results = results.unwrap();
        assert_eq!(results.len(), 1);
        if let wasmtime::Val::I32(v) = results[0] {
            assert_eq!(v, 42);
        } else {
            panic!("Expected I32 result");
        }

        std::fs::remove_file(wasm_path).ok();
    }

    #[test]
    fn test_plugin_not_found_error() {
        let mut host = PluginHost::new().unwrap();
        let result = host.call_plugin_function("nonexistent", "test", &[]);
        assert!(result.is_err());
        match result {
            Err(PluginError::NotFound(id)) => assert_eq!(id, "nonexistent"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_function_not_found_error() {
        let mut host = PluginHost::new().unwrap();

        let wat = r#"
            (module
              (import "env" "ocps_log" (func $log (param i32 i32)))
              (memory (export "memory") 1)

              (func (export "existing_func") (result i32)
                (i32.const 1)
              )
            )
        "#;

        let wasm_bytes = wat::parse_str(wat).unwrap();
        let temp_dir = std::env::temp_dir();
        let wasm_path = temp_dir.join("func_test.wasm");
        std::fs::write(&wasm_path, wasm_bytes).unwrap();

        let manifest = PluginManifest {
            id: "func-test".to_string(),
            name: "Func Test".to_string(),
            version: "1.0.0".to_string(),
            api_version: "1".to_string(),
            plugin_type: "image_filter".to_string(),
            author: "Test".to_string(),
            description: "Function test".to_string(),
            entry_point: "func_test.wasm".to_string(),
        };

        host.load_plugin(manifest, &wasm_path).unwrap();

        let result = host.call_plugin_function("func-test", "nonexistent_func", &[]);
        assert!(result.is_err());
        match result {
            Err(PluginError::FunctionNotFound(name)) => {
                assert_eq!(name, "nonexistent_func")
            }
            _ => panic!("Expected FunctionNotFound error"),
        }

        std::fs::remove_file(wasm_path).ok();
    }
}
