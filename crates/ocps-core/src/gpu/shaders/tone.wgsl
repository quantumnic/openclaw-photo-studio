// Tone adjustment compute shader
// Applies contrast, highlights, shadows, whites, blacks adjustments

@group(0) @binding(0) var<storage, read> input: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;
@group(0) @binding(2) var<uniform> params: ToneParams;

struct ToneParams {
    contrast: f32,        // -100 to +100
    highlights: f32,      // -100 to +100
    shadows: f32,         // -100 to +100
    whites: f32,          // -100 to +100
    blacks: f32,          // -100 to +100
    width: u32,
    height: u32,
    _pad: u32,
}

// S-curve for contrast
fn apply_contrast(val: f32, contrast: f32) -> f32 {
    if abs(contrast) < 0.001 {
        return val;
    }

    // Normalize contrast from -100..100 to -1..1
    let c = contrast / 100.0;

    // S-curve centered at 0.5
    let centered = val - 0.5;
    let adjusted = centered * (1.0 + c);
    return clamp(adjusted + 0.5, 0.0, 1.0);
}

// Range compression for highlights/shadows
fn apply_tone_range(val: f32, highlights: f32, shadows: f32) -> f32 {
    var result = val;

    // Normalize from -100..100 to -1..1
    let h = highlights / 100.0;
    let s = shadows / 100.0;

    // Highlights affect upper range (0.5-1.0)
    if val > 0.5 && abs(h) > 0.001 {
        let range = val - 0.5;
        let adjustment = range * h * 0.5; // Scale down adjustment
        result = result - adjustment;
    }

    // Shadows affect lower range (0.0-0.5)
    if val < 0.5 && abs(s) > 0.001 {
        let range = 0.5 - val;
        let adjustment = range * s * 0.5;
        result = result + adjustment;
    }

    return clamp(result, 0.0, 1.0);
}

// Endpoint adjustment for whites/blacks
fn apply_endpoints(val: f32, whites: f32, blacks: f32) -> f32 {
    var result = val;

    // Normalize from -100..100 to -1..1
    let w = whites / 100.0;
    let b = blacks / 100.0;

    // Whites push highlights up or down
    if abs(w) > 0.001 {
        let factor = 1.0 + (w * 0.3); // Scale adjustment
        result = pow(result, 1.0 / factor);
    }

    // Blacks push shadows up or down
    if abs(b) > 0.001 {
        let lift = b * 0.1; // Scale adjustment
        result = result + lift;
    }

    return clamp(result, 0.0, 1.0);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    let total_pixels = params.width * params.height * 3u; // RGB channels

    if idx >= total_pixels {
        return;
    }

    // Unpack u16 value from u32 storage
    let val = input[idx] & 0xFFFFu;

    // Convert to normalized linear [0.0, 1.0]
    var linear = f32(val) / 65535.0;

    // Apply adjustments in order:
    // 1. Contrast (S-curve)
    linear = apply_contrast(linear, params.contrast);

    // 2. Highlights/Shadows (range compression)
    linear = apply_tone_range(linear, params.highlights, params.shadows);

    // 3. Whites/Blacks (endpoint adjustment)
    linear = apply_endpoints(linear, params.whites, params.blacks);

    // Convert back to u16 and store
    let result = u32(clamp(linear, 0.0, 1.0) * 65535.0);
    output[idx] = result;
}
