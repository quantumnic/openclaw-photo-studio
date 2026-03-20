// Color adjustment compute shader
// Applies vibrance and saturation adjustments via HSV conversion

@group(0) @binding(0) var<storage, read> input: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;
@group(0) @binding(2) var<uniform> params: ColorParams;

struct ColorParams {
    vibrance: f32,    // -100 to +100
    saturation: f32,  // -100 to +100
    width: u32,
    height: u32,
}

// RGB to HSV conversion
fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let r = rgb.r;
    let g = rgb.g;
    let b = rgb.b;

    let max_val = max(max(r, g), b);
    let min_val = min(min(r, g), b);
    let delta = max_val - min_val;

    var h: f32 = 0.0;
    var s: f32 = 0.0;
    let v: f32 = max_val;

    if delta > 0.00001 {
        s = delta / max_val;

        if r >= max_val {
            h = (g - b) / delta;
        } else if g >= max_val {
            h = 2.0 + (b - r) / delta;
        } else {
            h = 4.0 + (r - g) / delta;
        }

        h = h * 60.0;
        if h < 0.0 {
            h = h + 360.0;
        }
    }

    return vec3<f32>(h, s, v);
}

// HSV to RGB conversion
fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h = hsv.x;
    let s = hsv.y;
    let v = hsv.z;

    if s <= 0.00001 {
        return vec3<f32>(v, v, v);
    }

    let h_sector = h / 60.0;
    let sector = u32(floor(h_sector));
    let frac = h_sector - f32(sector);

    let p = v * (1.0 - s);
    let q = v * (1.0 - s * frac);
    let t = v * (1.0 - s * (1.0 - frac));

    switch sector {
        case 0u: { return vec3<f32>(v, t, p); }
        case 1u: { return vec3<f32>(q, v, p); }
        case 2u: { return vec3<f32>(p, v, t); }
        case 3u: { return vec3<f32>(p, q, v); }
        case 4u: { return vec3<f32>(t, p, v); }
        default: { return vec3<f32>(v, p, q); }
    }
}

// Apply vibrance (smart saturation - affects low-saturated colors more)
fn apply_vibrance(hsv: vec3<f32>, vibrance: f32) -> vec3<f32> {
    if abs(vibrance) < 0.001 {
        return hsv;
    }

    let v = vibrance / 100.0;
    let current_sat = hsv.y;

    // Vibrance affects low-saturated colors more
    let weight = 1.0 - current_sat;
    let adjustment = v * weight;

    return vec3<f32>(hsv.x, clamp(current_sat + adjustment, 0.0, 1.0), hsv.z);
}

// Apply saturation (uniform across all colors)
fn apply_saturation(hsv: vec3<f32>, saturation: f32) -> vec3<f32> {
    if abs(saturation) < 0.001 {
        return hsv;
    }

    let s = saturation / 100.0;
    let current_sat = hsv.y;

    // Linear saturation adjustment
    let new_sat = clamp(current_sat * (1.0 + s), 0.0, 1.0);

    return vec3<f32>(hsv.x, new_sat, hsv.z);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let pixel_idx = id.x;
    let total_pixels = params.width * params.height;

    if pixel_idx >= total_pixels {
        return;
    }

    // Read RGB triplet (3 consecutive u32 values per pixel)
    let base_idx = pixel_idx * 3u;

    let r_val = input[base_idx] & 0xFFFFu;
    let g_val = input[base_idx + 1u] & 0xFFFFu;
    let b_val = input[base_idx + 2u] & 0xFFFFu;

    // Convert to normalized linear RGB [0.0, 1.0]
    var rgb = vec3<f32>(
        f32(r_val) / 65535.0,
        f32(g_val) / 65535.0,
        f32(b_val) / 65535.0
    );

    // Convert to HSV
    var hsv = rgb_to_hsv(rgb);

    // Apply vibrance
    hsv = apply_vibrance(hsv, params.vibrance);

    // Apply saturation
    hsv = apply_saturation(hsv, params.saturation);

    // Convert back to RGB
    rgb = hsv_to_rgb(hsv);

    // Clamp and convert back to u16
    let r_out = u32(clamp(rgb.r, 0.0, 1.0) * 65535.0);
    let g_out = u32(clamp(rgb.g, 0.0, 1.0) * 65535.0);
    let b_out = u32(clamp(rgb.b, 0.0, 1.0) * 65535.0);

    // Write back
    output[base_idx] = r_out;
    output[base_idx + 1u] = g_out;
    output[base_idx + 2u] = b_out;
}
