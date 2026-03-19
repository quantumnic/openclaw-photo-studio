//! GPU processing foundation (wgpu-based)
//! This module is behind the 'gpu' feature flag

#[cfg(feature = "gpu")]
pub mod context;

#[cfg(feature = "gpu")]
pub use context::GpuContext;
