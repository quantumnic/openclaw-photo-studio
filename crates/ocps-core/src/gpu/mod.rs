//! GPU processing foundation (wgpu-based)
//! This module is behind the 'gpu' feature flag

#[cfg(feature = "gpu")]
pub mod context;

#[cfg(feature = "gpu")]
pub mod pipeline;

#[cfg(feature = "gpu")]
pub use context::GpuContext;

#[cfg(feature = "gpu")]
pub use pipeline::{apply_exposure_gpu, GpuError};
