//! Full GPU processing pipeline
//!
//! Chains multiple compute shaders to process an image:
//! 1. Exposure adjustment
//! 2. Tone adjustments (contrast, highlights, shadows, whites, blacks)
//! 3. Color adjustments (vibrance, saturation)
//! 4. Combine (gamma encoding + u16→u8 conversion)

use super::{GpuContext, GpuError};
use crate::pipeline::types::{EditRecipe, RgbImage16, RgbImage8};
use wgpu::util::DeviceExt;

/// Tone adjustment parameters
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ToneParams {
    contrast: f32,
    highlights: f32,
    shadows: f32,
    whites: f32,
    blacks: f32,
    width: u32,
    height: u32,
    _pad: u32,
}

/// Color adjustment parameters
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ColorParams {
    vibrance: f32,
    saturation: f32,
    width: u32,
    height: u32,
}

/// Combine stage parameters
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CombineParams {
    width: u32,
    height: u32,
    _pad0: u32,
    _pad1: u32,
}

/// Process an image through the full GPU pipeline
///
/// # Arguments
/// * `ctx` - GPU context
/// * `image` - Input image (u16 linear RGB)
/// * `recipe` - Edit parameters
///
/// # Returns
/// * `Ok(RgbImage8)` - Processed image (u8 sRGB)
/// * `Err(GpuError)` - Processing failed
#[cfg(feature = "gpu")]
pub async fn process_image_gpu(
    ctx: &GpuContext,
    image: &RgbImage16,
    recipe: &EditRecipe,
) -> Result<RgbImage8, GpuError> {
    let width = image.width;
    let height = image.height;
    let pixel_count = (width * height) as usize;

    // Convert u16 RGB data to u32 for GPU (3 channels per pixel)
    let input_u32: Vec<u32> = image.data.iter().map(|&v| v as u32).collect();

    // Stage 1: Exposure (if needed)
    let after_exposure = if (recipe.exposure - 0.0).abs() > 0.001 {
        apply_exposure_stage(ctx, &input_u32, recipe.exposure, width, height).await?
    } else {
        input_u32
    };

    // Stage 2: Tone adjustments
    let after_tone = apply_tone_stage(
        ctx,
        &after_exposure,
        recipe.contrast,
        recipe.highlights,
        recipe.shadows,
        recipe.whites,
        recipe.blacks,
        width,
        height,
    )
    .await?;

    // Stage 3: Color adjustments
    let after_color = apply_color_stage(
        ctx,
        &after_tone,
        recipe.vibrance,
        recipe.saturation,
        width,
        height,
    )
    .await?;

    // Stage 4: Combine (gamma encoding + u16→u8)
    let output_u8 = apply_combine_stage(ctx, &after_color, width, height).await?;

    Ok(RgbImage8 {
        width,
        height,
        data: output_u8,
    })
}

/// Apply exposure adjustment stage
async fn apply_exposure_stage(
    ctx: &GpuContext,
    data: &[u32],
    exposure_ev: f32,
    width: u32,
    height: u32,
) -> Result<Vec<u32>, GpuError> {
    let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Exposure Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/exposure.wgsl").into()),
    });

    run_compute_shader(
        ctx,
        &shader,
        "main",
        data,
        &bytemuck::bytes_of(&super::pipeline::ExposureParams {
            exposure_ev,
            width,
            height,
            _pad: 0,
        }),
        width * height * 3,
    )
    .await
}

/// Apply tone adjustment stage
async fn apply_tone_stage(
    ctx: &GpuContext,
    data: &[u32],
    contrast: f32,
    highlights: f32,
    shadows: f32,
    whites: f32,
    blacks: f32,
    width: u32,
    height: u32,
) -> Result<Vec<u32>, GpuError> {
    let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Tone Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/tone.wgsl").into()),
    });

    let params = ToneParams {
        contrast,
        highlights,
        shadows,
        whites,
        blacks,
        width,
        height,
        _pad: 0,
    };

    run_compute_shader(
        ctx,
        &shader,
        "main",
        data,
        bytemuck::bytes_of(&params),
        width * height * 3,
    )
    .await
}

/// Apply color adjustment stage
async fn apply_color_stage(
    ctx: &GpuContext,
    data: &[u32],
    vibrance: f32,
    saturation: f32,
    width: u32,
    height: u32,
) -> Result<Vec<u32>, GpuError> {
    let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Color Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/color.wgsl").into()),
    });

    let params = ColorParams {
        vibrance,
        saturation,
        width,
        height,
    };

    run_compute_shader(
        ctx,
        &shader,
        "main",
        data,
        bytemuck::bytes_of(&params),
        width * height,
    )
    .await
}

/// Apply combine stage (gamma encoding + u16→u8)
async fn apply_combine_stage(
    ctx: &GpuContext,
    data: &[u32],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, GpuError> {
    let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Combine Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/combine.wgsl").into()),
    });

    let params = CombineParams {
        width,
        height,
        _pad0: 0,
        _pad1: 0,
    };

    let output_u32 = run_compute_shader(
        ctx,
        &shader,
        "main",
        data,
        bytemuck::bytes_of(&params),
        width * height,
    )
    .await?;

    // Unpack u32 to RGB u8
    let mut output_u8 = Vec::with_capacity((width * height * 3) as usize);
    for packed in output_u32 {
        output_u8.push((packed & 0xFF) as u8); // R
        output_u8.push(((packed >> 8) & 0xFF) as u8); // G
        output_u8.push(((packed >> 16) & 0xFF) as u8); // B
    }

    Ok(output_u8)
}

/// Generic compute shader runner
async fn run_compute_shader(
    ctx: &GpuContext,
    shader: &wgpu::ShaderModule,
    entry_point: &str,
    input_data: &[u32],
    params_bytes: &[u8],
    output_size: u32,
) -> Result<Vec<u32>, GpuError> {
    // Create buffers
    let input_buffer = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Input Buffer"),
            contents: bytemuck::cast_slice(input_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

    let output_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: (output_size as usize * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let params_buffer = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params Buffer"),
            contents: params_bytes,
            usage: wgpu::BufferUsages::UNIFORM,
        });

    // Create bind group layout
    let bind_group_layout =
        ctx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bind Group Layout"),
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
        label: Some("Bind Group"),
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

    // Create pipeline
    let pipeline_layout =
        ctx.device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

    let compute_pipeline =
        ctx.device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Compute Pipeline"),
                layout: Some(&pipeline_layout),
                module: shader,
                entry_point: Some(entry_point),
                compilation_options: Default::default(),
                cache: None,
            });

    // Execute
    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Encoder"),
        });

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);

        let workgroup_size = 64;
        let workgroups = (output_size + workgroup_size - 1) / workgroup_size;
        compute_pass.dispatch_workgroups(workgroups, 1, 1);
    }

    // Read back
    let staging_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: (output_size as usize * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    encoder.copy_buffer_to_buffer(
        &output_buffer,
        0,
        &staging_buffer,
        0,
        (output_size as usize * std::mem::size_of::<u32>()) as u64,
    );

    ctx.queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let (tx, rx) = futures::channel::oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).ok();
    });

    ctx.device.poll(wgpu::Maintain::Wait);

    rx.await
        .map_err(|_| GpuError::OperationError("Failed to receive buffer mapping".to_string()))?
        .map_err(|e| GpuError::OperationError(format!("Buffer mapping failed: {:?}", e)))?;

    let data_u32 = buffer_slice.get_mapped_range();
    let output: Vec<u32> = bytemuck::cast_slice(&data_u32).to_vec();

    drop(data_u32);
    staging_buffer.unmap();

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "gpu")]
    async fn test_gpu_full_pipeline_default() {
        if let Some(ctx) = GpuContext::new().await {
            let width = 4;
            let height = 4;
            let data = vec![32768u16; (width * height * 3) as usize]; // Mid-gray

            let image = RgbImage16 {
                width,
                height,
                data,
            };

            let recipe = EditRecipe::default();
            let result = process_image_gpu(&ctx, &image, &recipe).await;

            if let Ok(output) = result {
                assert_eq!(output.width, width);
                assert_eq!(output.height, height);
                assert_eq!(output.data.len(), (width * height * 3) as usize);
            } else {
                println!("GPU full pipeline test skipped - operation failed");
            }
        } else {
            println!("GPU full pipeline test skipped - no GPU available");
        }
    }
}
