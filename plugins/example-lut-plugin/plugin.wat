(module
  (import "env" "ocps_log" (func $log (param i32 i32)))
  (memory (export "memory") 1)

  ;; Simple identity filter: returns 0 (success)
  (func (export "process_image") (param i32 i32 i32) (result i32)
    ;; params: data_ptr, width, height
    ;; Returns 0 = success
    (i32.const 0)
  )

  ;; Get plugin version
  (func (export "get_version") (result i32)
    (i32.const 1)
  )

  ;; Store "example-lut v1.0.0" in memory at offset 0
  (data (i32.const 0) "example-lut v1.0.0")
)
