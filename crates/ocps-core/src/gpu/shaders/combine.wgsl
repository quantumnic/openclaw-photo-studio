// Final combine shader
// Applies gamma encoding (sRGB) and converts u16 linear to u8 sRGB

@group(0) @binding(0) var<storage, read> input: array<u32>;  // u16 linear RGB
@group(0) @binding(1) var<storage, read_write> output: array<u32>;  // u8 sRGB (packed)
@group(0) @binding(2) var<uniform> params: CombineParams;

struct CombineParams {
    width: u32,
    height: u32,
    _pad0: u32,
    _pad1: u32,
}

// sRGB gamma encoding (proper piecewise function)
fn linear_to_srgb(linear: f32) -> f32 {
    if linear <= 0.0031308 {
        return linear * 12.92;
    } else {
        return 1.055 * pow(linear, 1.0 / 2.4) - 0.055;
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let pixel_idx = id.x;
    let total_pixels = params.width * params.height;

    if pixel_idx >= total_pixels {
        return;
    }

    // Read RGB triplet from u16 linear
    let base_idx = pixel_idx * 3u;

    let r_val = input[base_idx] & 0xFFFFu;
    let g_val = input[base_idx + 1u] & 0xFFFFu;
    let b_val = input[base_idx + 2u] & 0xFFFFu;

    // Convert to normalized linear [0.0, 1.0]
    let r_linear = f32(r_val) / 65535.0;
    let g_linear = f32(g_val) / 65535.0;
    let b_linear = f32(b_val) / 65535.0;

    // Apply sRGB gamma encoding
    let r_srgb = linear_to_srgb(r_linear);
    let g_srgb = linear_to_srgb(g_linear);
    let b_srgb = linear_to_srgb(b_linear);

    // Convert to u8 [0, 255]
    let r_u8 = u32(clamp(r_srgb * 255.0, 0.0, 255.0));
    let g_u8 = u32(clamp(g_srgb * 255.0, 0.0, 255.0));
    let b_u8 = u32(clamp(b_srgb * 255.0, 0.0, 255.0));

    // Pack RGB into single u32 (R in bits 0-7, G in bits 8-15, B in bits 16-23)
    let packed = r_u8 | (g_u8 << 8u) | (b_u8 << 16u);

    // Write to output
    output[pixel_idx] = packed;
}
