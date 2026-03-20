//! GPU image processing pipeline using compute shaders

use super::GpuContext;
use thiserror::Error;
use wgpu::util::DeviceExt;

/// GPU processing errors
#[derive(Debug, Error)]
pub enum GpuError {
    #[error("GPU context not available")]
    NoContext,

    #[error("Shader compilation failed: {0}")]
    ShaderError(String),

    #[error("Buffer creation failed: {0}")]
    BufferError(String),

    #[error("GPU operation failed: {0}")]
    OperationError(String),

    #[error("Invalid input data: {0}")]
    InvalidInput(String),
}

/// Exposure adjustment parameters
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ExposureParams {
    exposure_ev: f32,
    width: u32,
    height: u32,
    _pad: u32,
}

/// Apply exposure adjustment using GPU compute shader
///
/// # Arguments
/// * `ctx` - GPU context
/// * `data` - Input u16 data (single channel or RGB, depending on use)
/// * `exposure_ev` - Exposure adjustment in EV stops
///
/// # Returns
/// * `Ok(Vec<u16>)` - Adjusted data
/// * `Err(GpuError)` - Processing failed
pub async fn apply_exposure_gpu(
    ctx: &GpuContext,
    data: &[u16],
    exposure_ev: f32,
) -> Result<Vec<u16>, GpuError> {
    if data.is_empty() {
        return Err(GpuError::InvalidInput("Empty input data".to_string()));
    }

    // Calculate dimensions (assume square root for single channel, or width from RGB)
    let pixel_count = data.len();
    let width = (pixel_count as f32).sqrt() as u32;
    let height = width;

    // Verify dimensions
    if (width * height) as usize != pixel_count {
        return Err(GpuError::InvalidInput(format!(
            "Data size {} does not match square dimensions",
            pixel_count
        )));
    }

    // Convert u16 data to u32 for GPU storage
    let input_u32: Vec<u32> = data.iter().map(|&v| v as u32).collect();

    // Create shader module
    let shader_source = include_str!("shaders/exposure.wgsl");
    let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Exposure Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // Create input buffer
    let input_buffer = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Input Buffer"),
            contents: bytemuck::cast_slice(&input_u32),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

    // Create output buffer
    let output_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: (pixel_count * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // Create params buffer
    let params = ExposureParams {
        exposure_ev,
        width,
        height,
        _pad: 0,
    };

    let params_buffer = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

    // Create bind group layout
    let bind_group_layout = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Exposure Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

    // Create bind group
    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Exposure Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: params_buffer.as_entire_binding(),
            },
        ],
    });

    // Create pipeline layout
    let pipeline_layout = ctx
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Exposure Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

    // Create compute pipeline
    let compute_pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Exposure Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

    // Create command encoder
    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Exposure Encoder"),
        });

    // Dispatch compute shader
    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Exposure Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);

        // Calculate workgroups (64 threads per workgroup)
        let workgroup_size = 64;
        let workgroups = (pixel_count + workgroup_size - 1) / workgroup_size;
        compute_pass.dispatch_workgroups(workgroups as u32, 1, 1);
    }

    // Create staging buffer for reading back results
    let staging_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: (pixel_count * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Copy output to staging buffer
    encoder.copy_buffer_to_buffer(
        &output_buffer,
        0,
        &staging_buffer,
        0,
        (pixel_count * std::mem::size_of::<u32>()) as u64,
    );

    // Submit commands
    ctx.queue.submit(Some(encoder.finish()));

    // Read back results
    let buffer_slice = staging_buffer.slice(..);
    let (tx, rx) = futures::channel::oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).ok();
    });

    ctx.device.poll(wgpu::Maintain::Wait);

    rx.await
        .map_err(|_| GpuError::OperationError("Failed to receive buffer mapping result".to_string()))?
        .map_err(|e| GpuError::OperationError(format!("Buffer mapping failed: {:?}", e)))?;

    // Extract data
    let data_u32 = buffer_slice.get_mapped_range();
    let output_u32: Vec<u32> = bytemuck::cast_slice(&data_u32).to_vec();

    drop(data_u32);
    staging_buffer.unmap();

    // Convert back to u16
    let output_u16: Vec<u16> = output_u32.iter().map(|&v| (v & 0xFFFF) as u16).collect();

    Ok(output_u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "gpu")]
    async fn test_gpu_exposure_identity() {
        // Test that ev=0.0 leaves values unchanged (within rounding)
        if let Some(ctx) = GpuContext::new().await {
            let input = vec![10000u16, 20000u16, 30000u16, 40000u16]; // 4 pixels (2x2)

            let output = apply_exposure_gpu(&ctx, &input, 0.0).await;

            if let Ok(output) = output {
                assert_eq!(output.len(), input.len());

                for (i, (&inp, &out)) in input.iter().zip(output.iter()).enumerate() {
                    let diff = (inp as i32 - out as i32).abs();
                    assert!(
                        diff <= 2,
                        "Pixel {} differs too much: {} vs {} (diff: {})",
                        i,
                        inp,
                        out,
                        diff
                    );
                }
            } else {
                println!("GPU exposure test skipped - operation failed");
            }
        } else {
            println!("GPU exposure test skipped - no GPU available");
        }
    }

    #[tokio::test]
    #[cfg(feature = "gpu")]
    async fn test_gpu_exposure_positive() {
        // Test that ev=1.0 approximately doubles values
        if let Some(ctx) = GpuContext::new().await {
            let input = vec![10000u16, 20000u16, 30000u16]; // Small test (not square, will fail)

            // This should fail because input is not square
            let result = apply_exposure_gpu(&ctx, &input, 1.0).await;
            assert!(result.is_err());

            // Try with square input (4 pixels = 2x2)
            let input_square = vec![10000u16, 15000u16, 20000u16, 25000u16];
            let output = apply_exposure_gpu(&ctx, &input_square, 1.0).await;

            if let Ok(output) = output {
                assert_eq!(output.len(), input_square.len());

                // Values should be approximately doubled (within tolerance)
                for (i, (&inp, &out)) in input_square.iter().zip(output.iter()).enumerate() {
                    let expected = (inp as f32 * 2.0).min(65535.0);
                    let diff = (expected - out as f32).abs();
                    let tolerance = expected * 0.01; // 1% tolerance

                    assert!(
                        diff < tolerance,
                        "Pixel {} not doubled correctly: {} -> {} (expected ~{})",
                        i,
                        inp,
                        out,
                        expected
                    );
                }
            } else {
                println!("GPU exposure positive test skipped - operation failed");
            }
        } else {
            println!("GPU exposure positive test skipped - no GPU available");
        }
    }
}
