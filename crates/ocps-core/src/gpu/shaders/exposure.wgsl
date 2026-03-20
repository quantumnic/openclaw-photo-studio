// Exposure adjustment compute shader
// Applies exposure compensation to linear RGB data

@group(0) @binding(0) var<storage, read> input: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;
@group(0) @binding(2) var<uniform> params: ExposureParams;

struct ExposureParams {
    exposure_ev: f32,
    width: u32,
    height: u32,
    _pad: u32,
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    let total_pixels = params.width * params.height;

    if idx >= total_pixels {
        return;
    }

    // Unpack u16 value from u32 storage (lower 16 bits)
    let val = input[idx] & 0xFFFFu;

    // Convert to normalized linear [0.0, 1.0]
    let linear = f32(val) / 65535.0;

    // Apply exposure: multiply by 2^ev
    let factor = pow(2.0, params.exposure_ev);
    let adjusted = clamp(linear * factor, 0.0, 1.0);

    // Convert back to u16 and store in u32
    let result = u32(adjusted * 65535.0);
    output[idx] = result;
}
