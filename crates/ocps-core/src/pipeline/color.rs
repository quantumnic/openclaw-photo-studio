//! Color space conversion and gamma encoding/decoding

/// Convert linear RGB to sRGB gamma-encoded value
pub fn gamma_encode(linear: f32) -> f32 {
    if linear <= 0.0031308 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

/// Convert sRGB gamma-encoded value to linear RGB
pub fn gamma_decode(srgb: f32) -> f32 {
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert u16 linear to u8 sRGB
#[inline]
pub fn u16_linear_to_u8_srgb(value: u16, white_level: u16) -> u8 {
    let linear = (value as f32) / (white_level as f32);
    let clamped = linear.clamp(0.0, 1.0);
    let encoded = gamma_encode(clamped);
    (encoded * 255.0).round() as u8
}

/// Convert RGB to HSV
/// r, g, b in range [0.0, 1.0]
/// Returns (h, s, v) where h is in [0.0, 360.0], s and v in [0.0, 1.0]
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;

    if delta < 1e-6 {
        return (0.0, 0.0, v);
    }

    let s = if max > 0.0 { delta / max } else { 0.0 };

    let h = if (r - max).abs() < 1e-6 {
        // Red is max
        60.0 * (((g - b) / delta) % 6.0)
    } else if (g - max).abs() < 1e-6 {
        // Green is max
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        // Blue is max
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    (h, s, v)
}

/// Convert HSV to RGB
/// h in [0.0, 360.0], s and v in [0.0, 1.0]
/// Returns (r, g, b) in [0.0, 1.0]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    if s < 1e-6 {
        return (v, v, v);
    }

    let h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r + m, g + m, b + m)
}

/// Calculate white balance multipliers from temperature and tint
/// Returns [r_mult, g_mult, b_mult]
pub fn calculate_wb_multipliers(temperature: u32, tint: i32) -> [f32; 3] {
    // Simplified white balance using Planckian locus approximation
    // Temperature in Kelvin (2000-50000)
    // Tint in -150 to +150

    let temp = (temperature as f32).clamp(2000.0, 50000.0);
    let tint_factor = (tint as f32) / 150.0; // Normalize to -1.0 to 1.0

    // Calculate red multiplier based on temperature
    let r = if temp <= 6600.0 {
        1.0
    } else {
        let t = (temp - 6000.0) / 1000.0;
        (1.0 + 0.18 * t).min(2.0)
    };

    // Calculate blue multiplier based on temperature
    let b = if temp >= 6600.0 {
        1.0
    } else {
        let t = (6600.0 - temp) / 1000.0;
        (1.0 + 0.18 * t).min(2.0)
    };

    // Green is affected by tint
    // Positive tint = more green (magenta -> green)
    // Negative tint = less green (green -> magenta)
    // We apply tint before normalization
    let g = 1.0 + (tint_factor * 0.3);

    // Normalize so that the minimum is 1.0
    // This ensures we never multiply down (only up)
    let min_mult = r.min(g).min(b);
    [r / min_mult, g / min_mult, b / min_mult]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamma_encode_decode_roundtrip() {
        let values = [0.0, 0.1, 0.5, 0.8, 1.0];
        for &val in &values {
            let encoded = gamma_encode(val);
            let decoded = gamma_decode(encoded);
            assert!((decoded - val).abs() < 1e-6, "Failed for {}", val);
        }
    }

    #[test]
    fn test_gamma_encode_boundaries() {
        assert_eq!(gamma_encode(0.0), 0.0);
        assert!((gamma_encode(1.0) - 1.0).abs() < 1e-6);

        // Test transition point
        let transition = 0.0031308;
        let encoded = gamma_encode(transition);
        assert!(encoded > 0.0 && encoded < 1.0);
    }

    #[test]
    fn test_u16_to_u8_conversion() {
        // Black
        assert_eq!(u16_linear_to_u8_srgb(0, 65535), 0);

        // White
        assert_eq!(u16_linear_to_u8_srgb(65535, 65535), 255);

        // Mid-gray (18% gray)
        let mid = (65535.0 * 0.18) as u16;
        let result = u16_linear_to_u8_srgb(mid, 65535);
        // sRGB 18% gray should be around 117 (0.46 * 255)
        assert!(result > 100 && result < 130);
    }

    #[test]
    fn test_rgb_to_hsv_pure_colors() {
        // Red
        let (h, s, v) = rgb_to_hsv(1.0, 0.0, 0.0);
        assert!((h - 0.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 1e-5);
        assert!((v - 1.0).abs() < 1e-5);

        // Green
        let (h, s, v) = rgb_to_hsv(0.0, 1.0, 0.0);
        assert!((h - 120.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 1e-5);
        assert!((v - 1.0).abs() < 1e-5);

        // Blue
        let (h, s, v) = rgb_to_hsv(0.0, 0.0, 1.0);
        assert!((h - 240.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 1e-5);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_rgb_to_hsv_gray() {
        let (h, s, v) = rgb_to_hsv(0.5, 0.5, 0.5);
        assert!((s - 0.0).abs() < 1e-5);
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_hsv_to_rgb_roundtrip() {
        let test_cases = [
            (1.0, 0.0, 0.0),   // Red
            (0.0, 1.0, 0.0),   // Green
            (0.0, 0.0, 1.0),   // Blue
            (0.5, 0.5, 0.5),   // Gray
            (0.8, 0.3, 0.6),   // Random color
        ];

        for &(r, g, b) in &test_cases {
            let (h, s, v) = rgb_to_hsv(r, g, b);
            let (r2, g2, b2) = hsv_to_rgb(h, s, v);
            assert!((r - r2).abs() < 1e-5, "R: {} vs {}", r, r2);
            assert!((g - g2).abs() < 1e-5, "G: {} vs {}", g, g2);
            assert!((b - b2).abs() < 1e-5, "B: {} vs {}", b, b2);
        }
    }

    #[test]
    fn test_wb_multipliers_neutral() {
        let wb = calculate_wb_multipliers(5500, 0);
        // At neutral temp and tint, multipliers should be close to [1, 1, 1]
        assert!((wb[0] - 1.0).abs() < 0.2);
        assert!((wb[1] - 1.0).abs() < 0.2);
        assert!((wb[2] - 1.0).abs() < 0.2);
    }

    #[test]
    fn test_wb_multipliers_warm() {
        let wb = calculate_wb_multipliers(8000, 0);
        // Warm temp should increase red multiplier
        assert!(wb[0] > 1.0, "Red should be > 1.0 for warm temp");
        assert!(wb[2] <= 1.0, "Blue should be <= 1.0 for warm temp");
    }

    #[test]
    fn test_wb_multipliers_cool() {
        let wb = calculate_wb_multipliers(3000, 0);
        // Cool temp should increase blue multiplier
        assert!(wb[2] > 1.0, "Blue should be > 1.0 for cool temp");
        assert!(wb[0] <= 1.0, "Red should be <= 1.0 for cool temp");
    }

    #[test]
    fn test_wb_multipliers_tint() {
        let wb_neutral = calculate_wb_multipliers(5500, 0);
        let wb_pos = calculate_wb_multipliers(5500, 100);
        let wb_neg = calculate_wb_multipliers(5500, -100);

        // Positive tint should increase green relative to neutral
        assert!(wb_pos[1] > wb_neutral[1], "Positive tint should increase green");

        // Negative tint reduces green, but after normalization it affects the whole balance
        // The key is that the color balance changes - we can verify by checking
        // that the ratios are different
        let neutral_ratio = wb_neutral[1] / wb_neutral[0];
        let neg_ratio = wb_neg[1] / wb_neg[0];
        assert!(neg_ratio < neutral_ratio, "Negative tint should reduce green/red ratio");
    }
}
