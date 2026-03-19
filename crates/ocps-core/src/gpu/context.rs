//! GPU context initialization and management

#[cfg(feature = "gpu")]
use wgpu;

/// GPU context for accelerated image processing
#[cfg(feature = "gpu")]
pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: wgpu::AdapterInfo,
}

#[cfg(feature = "gpu")]
impl GpuContext {
    /// Initialize GPU context asynchronously
    /// Returns None if no suitable GPU adapter is found
    pub async fn new() -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await?;

        let info = adapter.get_info();
        log::info!(
            "GPU adapter found: {} ({:?}) - Backend: {:?}",
            info.name,
            info.device_type,
            info.backend
        );

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("OCPS GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .ok()?;

        Some(Self {
            device,
            queue,
            adapter_info: info,
        })
    }

    /// Get the name of the GPU adapter
    pub fn adapter_name(&self) -> &str {
        &self.adapter_info.name
    }

    /// Get the backend type (Metal, Vulkan, DX12, etc.)
    pub fn backend(&self) -> &str {
        match self.adapter_info.backend {
            wgpu::Backend::Vulkan => "Vulkan",
            wgpu::Backend::Metal => "Metal",
            wgpu::Backend::Dx12 => "DirectX 12",
            wgpu::Backend::Gl => "OpenGL",
            wgpu::Backend::BrowserWebGpu => "WebGPU",
            _ => "Unknown",
        }
    }

    /// Get the device type (Integrated, Discrete, etc.)
    pub fn device_type(&self) -> &str {
        match self.adapter_info.device_type {
            wgpu::DeviceType::DiscreteGpu => "Discrete GPU",
            wgpu::DeviceType::IntegratedGpu => "Integrated GPU",
            wgpu::DeviceType::VirtualGpu => "Virtual GPU",
            wgpu::DeviceType::Cpu => "CPU",
            wgpu::DeviceType::Other => "Other",
        }
    }
}

#[cfg(test)]
#[cfg(feature = "gpu")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_context_initialization() {
        // This test may fail on CI without GPU
        // It's OK to skip if no GPU is available
        if let Some(ctx) = GpuContext::new().await {
            println!("GPU Context initialized:");
            println!("  Adapter: {}", ctx.adapter_name());
            println!("  Backend: {}", ctx.backend());
            println!("  Type: {}", ctx.device_type());
            assert!(!ctx.adapter_name().is_empty());
        } else {
            println!("No GPU available - skipping test");
        }
    }
}
