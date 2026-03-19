//! Text watermark engine with bitmap font rendering

use crate::ExportError;

/// Watermark position on the image
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum WatermarkPosition {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    Center,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// Text watermark settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TextWatermark {
    pub text: String,
    pub font_size: f32,           // relative to image height, e.g. 0.05 = 5%
    pub opacity: f32,             // 0-1
    pub position: WatermarkPosition,
    pub color: [u8; 3],          // RGB
    pub inset: f32,              // margin from edge, relative to image size
}

impl Default for TextWatermark {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_size: 0.03,              // 3% of image height
            opacity: 0.5,
            position: WatermarkPosition::BottomRight,
            color: [255, 255, 255],       // White
            inset: 0.02,                  // 2% margin
        }
    }
}

// Simple 8x8 bitmap font for ASCII 32-126
// Each character is represented as 8 bytes (8x8 pixels)
// 1 = pixel on, 0 = pixel off
const BITMAP_FONT: &[&[u8; 8]] = &[
    // Space (32)
    &[0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000],
    // ! (33)
    &[0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00000000, 0b00011000, 0b00000000],
    // " (34)
    &[0b01100110, 0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000],
    // # (35)
    &[0b01100110, 0b01100110, 0b11111111, 0b01100110, 0b11111111, 0b01100110, 0b01100110, 0b00000000],
    // $ (36)
    &[0b00011000, 0b00111110, 0b01011000, 0b00111100, 0b00011010, 0b01111100, 0b00011000, 0b00000000],
    // % (37)
    &[0b01100010, 0b01100100, 0b00001000, 0b00010000, 0b00100000, 0b01001100, 0b10001100, 0b00000000],
    // & (38)
    &[0b00110000, 0b01001000, 0b01001000, 0b00110000, 0b01001010, 0b01000100, 0b00111010, 0b00000000],
    // ' (39)
    &[0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000],
    // ( (40)
    &[0b00001100, 0b00011000, 0b00110000, 0b00110000, 0b00110000, 0b00011000, 0b00001100, 0b00000000],
    // ) (41)
    &[0b00110000, 0b00011000, 0b00001100, 0b00001100, 0b00001100, 0b00011000, 0b00110000, 0b00000000],
    // * (42)
    &[0b00000000, 0b01100110, 0b00111100, 0b11111111, 0b00111100, 0b01100110, 0b00000000, 0b00000000],
    // + (43)
    &[0b00000000, 0b00011000, 0b00011000, 0b01111110, 0b00011000, 0b00011000, 0b00000000, 0b00000000],
    // , (44)
    &[0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00011000, 0b00011000, 0b00110000],
    // - (45)
    &[0b00000000, 0b00000000, 0b00000000, 0b01111110, 0b00000000, 0b00000000, 0b00000000, 0b00000000],
    // . (46)
    &[0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00011000, 0b00011000, 0b00000000],
    // / (47)
    &[0b00000010, 0b00000100, 0b00001000, 0b00010000, 0b00100000, 0b01000000, 0b10000000, 0b00000000],
    // 0 (48)
    &[0b00111100, 0b01100110, 0b01101110, 0b01110110, 0b01100110, 0b01100110, 0b00111100, 0b00000000],
    // 1 (49)
    &[0b00011000, 0b00111000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b01111110, 0b00000000],
    // 2 (50)
    &[0b00111100, 0b01100110, 0b00000110, 0b00001100, 0b00110000, 0b01100000, 0b01111110, 0b00000000],
    // 3 (51)
    &[0b00111100, 0b01100110, 0b00000110, 0b00011100, 0b00000110, 0b01100110, 0b00111100, 0b00000000],
    // 4 (52)
    &[0b00001100, 0b00011100, 0b00101100, 0b01001100, 0b01111110, 0b00001100, 0b00001100, 0b00000000],
    // 5 (53)
    &[0b01111110, 0b01100000, 0b01111100, 0b00000110, 0b00000110, 0b01100110, 0b00111100, 0b00000000],
    // 6 (54)
    &[0b00111100, 0b01100000, 0b01111100, 0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00000000],
    // 7 (55)
    &[0b01111110, 0b00000110, 0b00001100, 0b00011000, 0b00110000, 0b00110000, 0b00110000, 0b00000000],
    // 8 (56)
    &[0b00111100, 0b01100110, 0b01100110, 0b00111100, 0b01100110, 0b01100110, 0b00111100, 0b00000000],
    // 9 (57)
    &[0b00111100, 0b01100110, 0b01100110, 0b00111110, 0b00000110, 0b00001100, 0b00111000, 0b00000000],
    // : (58)
    &[0b00000000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00011000, 0b00011000, 0b00000000],
    // ; (59)
    &[0b00000000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00011000, 0b00011000, 0b00110000],
    // < (60)
    &[0b00000110, 0b00001100, 0b00011000, 0b00110000, 0b00011000, 0b00001100, 0b00000110, 0b00000000],
    // = (61)
    &[0b00000000, 0b00000000, 0b01111110, 0b00000000, 0b01111110, 0b00000000, 0b00000000, 0b00000000],
    // > (62)
    &[0b01100000, 0b00110000, 0b00011000, 0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b00000000],
    // ? (63)
    &[0b00111100, 0b01100110, 0b00000110, 0b00001100, 0b00011000, 0b00000000, 0b00011000, 0b00000000],
    // @ (64)
    &[0b00111100, 0b01100110, 0b01101110, 0b01101110, 0b01100000, 0b01100010, 0b00111100, 0b00000000],
    // A (65)
    &[0b00011000, 0b00111100, 0b01100110, 0b01100110, 0b01111110, 0b01100110, 0b01100110, 0b00000000],
    // B (66)
    &[0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b00000000],
    // C (67)
    &[0b00111100, 0b01100110, 0b01100000, 0b01100000, 0b01100000, 0b01100110, 0b00111100, 0b00000000],
    // D (68)
    &[0b01111000, 0b01101100, 0b01100110, 0b01100110, 0b01100110, 0b01101100, 0b01111000, 0b00000000],
    // E (69)
    &[0b01111110, 0b01100000, 0b01100000, 0b01111100, 0b01100000, 0b01100000, 0b01111110, 0b00000000],
    // F (70)
    &[0b01111110, 0b01100000, 0b01100000, 0b01111100, 0b01100000, 0b01100000, 0b01100000, 0b00000000],
    // G (71)
    &[0b00111100, 0b01100110, 0b01100000, 0b01101110, 0b01100110, 0b01100110, 0b00111100, 0b00000000],
    // H (72)
    &[0b01100110, 0b01100110, 0b01100110, 0b01111110, 0b01100110, 0b01100110, 0b01100110, 0b00000000],
    // I (73)
    &[0b01111110, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b01111110, 0b00000000],
    // J (74)
    &[0b00000110, 0b00000110, 0b00000110, 0b00000110, 0b00000110, 0b01100110, 0b00111100, 0b00000000],
    // K (75)
    &[0b01100110, 0b01101100, 0b01111000, 0b01110000, 0b01111000, 0b01101100, 0b01100110, 0b00000000],
    // L (76)
    &[0b01100000, 0b01100000, 0b01100000, 0b01100000, 0b01100000, 0b01100000, 0b01111110, 0b00000000],
    // M (77)
    &[0b01100011, 0b01110111, 0b01111111, 0b01101011, 0b01100011, 0b01100011, 0b01100011, 0b00000000],
    // N (78)
    &[0b01100110, 0b01110110, 0b01111110, 0b01111110, 0b01101110, 0b01100110, 0b01100110, 0b00000000],
    // O (79)
    &[0b00111100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00000000],
    // P (80)
    &[0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01100000, 0b01100000, 0b01100000, 0b00000000],
    // Q (81)
    &[0b00111100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00001110, 0b00000000],
    // R (82)
    &[0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01111000, 0b01101100, 0b01100110, 0b00000000],
    // S (83)
    &[0b00111100, 0b01100110, 0b01100000, 0b00111100, 0b00000110, 0b01100110, 0b00111100, 0b00000000],
    // T (84)
    &[0b01111110, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00000000],
    // U (85)
    &[0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00000000],
    // V (86)
    &[0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00011000, 0b00000000],
    // W (87)
    &[0b01100011, 0b01100011, 0b01100011, 0b01101011, 0b01111111, 0b01110111, 0b01100011, 0b00000000],
    // X (88)
    &[0b01100110, 0b01100110, 0b00111100, 0b00011000, 0b00111100, 0b01100110, 0b01100110, 0b00000000],
    // Y (89)
    &[0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00011000, 0b00011000, 0b00011000, 0b00000000],
    // Z (90)
    &[0b01111110, 0b00000110, 0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b01111110, 0b00000000],
];

// Continue with more ASCII characters (91-126) - simplified subset
const BITMAP_FONT_EXTRA: &[&[u8; 8]] = &[
    // [ to ` (91-96) - using simple placeholders
    &[0b00000000; 8], &[0b00000000; 8], &[0b00000000; 8], &[0b00000000; 8], &[0b00000000; 8], &[0b00000000; 8],
    // a (97)
    &[0b00000000, 0b00000000, 0b00111100, 0b00000110, 0b00111110, 0b01100110, 0b00111110, 0b00000000],
    // b (98)
    &[0b01100000, 0b01100000, 0b01111100, 0b01100110, 0b01100110, 0b01100110, 0b01111100, 0b00000000],
    // c (99)
    &[0b00000000, 0b00000000, 0b00111100, 0b01100000, 0b01100000, 0b01100000, 0b00111100, 0b00000000],
    // d (100)
    &[0b00000110, 0b00000110, 0b00111110, 0b01100110, 0b01100110, 0b01100110, 0b00111110, 0b00000000],
    // e (101)
    &[0b00000000, 0b00000000, 0b00111100, 0b01100110, 0b01111110, 0b01100000, 0b00111100, 0b00000000],
    // f (102)
    &[0b00011100, 0b00110000, 0b00110000, 0b01111100, 0b00110000, 0b00110000, 0b00110000, 0b00000000],
    // g (103)
    &[0b00000000, 0b00000000, 0b00111110, 0b01100110, 0b01100110, 0b00111110, 0b00000110, 0b00111100],
    // h (104)
    &[0b01100000, 0b01100000, 0b01111100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00000000],
    // i (105)
    &[0b00011000, 0b00000000, 0b00111000, 0b00011000, 0b00011000, 0b00011000, 0b00111100, 0b00000000],
    // j (106)
    &[0b00000110, 0b00000000, 0b00000110, 0b00000110, 0b00000110, 0b00000110, 0b01100110, 0b00111100],
    // k (107)
    &[0b01100000, 0b01100000, 0b01100110, 0b01101100, 0b01111000, 0b01101100, 0b01100110, 0b00000000],
    // l (108)
    &[0b00111000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00111100, 0b00000000],
    // m (109)
    &[0b00000000, 0b00000000, 0b01100110, 0b01111111, 0b01111111, 0b01101011, 0b01100011, 0b00000000],
    // n (110)
    &[0b00000000, 0b00000000, 0b01111100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00000000],
    // o (111)
    &[0b00000000, 0b00000000, 0b00111100, 0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00000000],
    // p (112)
    &[0b00000000, 0b00000000, 0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01100000, 0b01100000],
    // q (113)
    &[0b00000000, 0b00000000, 0b00111110, 0b01100110, 0b01100110, 0b00111110, 0b00000110, 0b00000110],
    // r (114)
    &[0b00000000, 0b00000000, 0b01111100, 0b01100110, 0b01100000, 0b01100000, 0b01100000, 0b00000000],
    // s (115)
    &[0b00000000, 0b00000000, 0b00111110, 0b01100000, 0b00111100, 0b00000110, 0b01111100, 0b00000000],
    // t (116)
    &[0b00110000, 0b00110000, 0b01111100, 0b00110000, 0b00110000, 0b00110000, 0b00011100, 0b00000000],
    // u (117)
    &[0b00000000, 0b00000000, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111110, 0b00000000],
    // v (118)
    &[0b00000000, 0b00000000, 0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00011000, 0b00000000],
    // w (119)
    &[0b00000000, 0b00000000, 0b01100011, 0b01101011, 0b01111111, 0b00111110, 0b00110110, 0b00000000],
    // x (120)
    &[0b00000000, 0b00000000, 0b01100110, 0b00111100, 0b00011000, 0b00111100, 0b01100110, 0b00000000],
    // y (121)
    &[0b00000000, 0b00000000, 0b01100110, 0b01100110, 0b01100110, 0b00111110, 0b00000110, 0b00111100],
    // z (122)
    &[0b00000000, 0b00000000, 0b01111110, 0b00001100, 0b00011000, 0b00110000, 0b01111110, 0b00000000],
    // { to ~ (123-126) - using simple placeholders
    &[0b00000000; 8], &[0b00000000; 8], &[0b00000000; 8], &[0b00000000; 8],
];

/// Get bitmap for a character (ASCII 32-126)
fn get_char_bitmap(c: char) -> Option<&'static [u8; 8]> {
    let code = c as usize;
    if (32..=90).contains(&code) {
        Some(BITMAP_FONT[code - 32])
    } else if (91..=126).contains(&code) {
        Some(BITMAP_FONT_EXTRA[code - 91])
    } else {
        None
    }
}

/// Character rendering parameters
#[derive(Clone)]
struct RenderParams {
    x: i32,
    y: i32,
    scale: u32,
    color: [u8; 3],
    opacity: f32,
}

/// Render a single character onto the image at specified position
fn render_char(
    rgb_data: &mut [u8],
    width: u32,
    height: u32,
    c: char,
    params: &RenderParams,
) {
    let bitmap = match get_char_bitmap(c) {
        Some(b) => b,
        None => return, // Skip unsupported characters
    };

    // Render each pixel of the 8x8 bitmap
    for (row, &byte) in bitmap.iter().enumerate() {
        let row = row as u32;
        for col in 0..8u32 {
            let bit = (byte >> (7 - col)) & 1;
            if bit == 1 {
                // Draw scaled pixel
                for dy in 0..params.scale {
                    for dx in 0..params.scale {
                        let px = params.x + (col * params.scale + dx) as i32;
                        let py = params.y + (row * params.scale + dy) as i32;

                        // Bounds check
                        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                            let idx = ((py as u32 * width + px as u32) * 3) as usize;
                            if idx + 2 < rgb_data.len() {
                                // Blend with existing pixel using opacity
                                let bg_r = rgb_data[idx] as f32;
                                let bg_g = rgb_data[idx + 1] as f32;
                                let bg_b = rgb_data[idx + 2] as f32;

                                rgb_data[idx] = (bg_r * (1.0 - params.opacity) + params.color[0] as f32 * params.opacity) as u8;
                                rgb_data[idx + 1] = (bg_g * (1.0 - params.opacity) + params.color[1] as f32 * params.opacity) as u8;
                                rgb_data[idx + 2] = (bg_b * (1.0 - params.opacity) + params.color[2] as f32 * params.opacity) as u8;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Apply text watermark to RGB image
///
/// # Arguments
/// * `rgb_data` - Mutable 8-bit RGB data to modify
/// * `width` - Image width
/// * `height` - Image height
/// * `watermark` - Watermark settings
///
/// # Returns
/// Ok(()) on success, or ExportError if invalid
pub fn apply_text_watermark(
    rgb_data: &mut [u8],
    width: u32,
    height: u32,
    watermark: &TextWatermark,
) -> Result<(), ExportError> {
    // Validate input
    let expected_size = (width * height * 3) as usize;
    if rgb_data.len() != expected_size {
        return Err(ExportError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid RGB data size",
        )));
    }

    // Skip if text is empty or opacity is zero
    if watermark.text.is_empty() || watermark.opacity <= 0.0 {
        return Ok(());
    }

    // Calculate font scale based on image height
    let font_height_pixels = (height as f32 * watermark.font_size).max(8.0) as u32;
    let scale = (font_height_pixels / 8).max(1); // Each char is 8px tall

    // Calculate text dimensions
    let char_width = 8 * scale;
    let char_height = 8 * scale;
    let text_width = watermark.text.len() as u32 * char_width;
    let text_height = char_height;

    // Calculate inset in pixels
    let inset_x = (width as f32 * watermark.inset) as i32;
    let inset_y = (height as f32 * watermark.inset) as i32;

    // Calculate starting position based on alignment
    let (start_x, start_y) = match watermark.position {
        WatermarkPosition::TopLeft => (inset_x, inset_y),
        WatermarkPosition::TopCenter => ((width as i32 - text_width as i32) / 2, inset_y),
        WatermarkPosition::TopRight => (width as i32 - text_width as i32 - inset_x, inset_y),
        WatermarkPosition::MiddleLeft => (inset_x, (height as i32 - text_height as i32) / 2),
        WatermarkPosition::Center => (
            (width as i32 - text_width as i32) / 2,
            (height as i32 - text_height as i32) / 2,
        ),
        WatermarkPosition::MiddleRight => (
            width as i32 - text_width as i32 - inset_x,
            (height as i32 - text_height as i32) / 2,
        ),
        WatermarkPosition::BottomLeft => (inset_x, height as i32 - text_height as i32 - inset_y),
        WatermarkPosition::BottomCenter => (
            (width as i32 - text_width as i32) / 2,
            height as i32 - text_height as i32 - inset_y,
        ),
        WatermarkPosition::BottomRight => (
            width as i32 - text_width as i32 - inset_x,
            height as i32 - text_height as i32 - inset_y,
        ),
    };

    // Render each character
    let params = RenderParams {
        x: 0,
        y: start_y,
        scale,
        color: watermark.color,
        opacity: watermark.opacity.clamp(0.0, 1.0),
    };

    for (i, c) in watermark.text.chars().enumerate() {
        let mut char_params = params.clone();
        char_params.x = start_x + (i as i32 * char_width as i32);
        render_char(rgb_data, width, height, c, &char_params);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_char_bitmap() {
        assert!(get_char_bitmap('A').is_some());
        assert!(get_char_bitmap('z').is_some());
        assert!(get_char_bitmap('0').is_some());
        assert!(get_char_bitmap(' ').is_some());
    }

    #[test]
    fn test_watermark_modifies_pixels() {
        let width = 200;
        let height = 200;
        let mut rgb_data = vec![100u8; (width * height * 3) as usize];
        let original = rgb_data.clone();

        let watermark = TextWatermark {
            text: "TEST".to_string(),
            font_size: 0.1,
            opacity: 1.0,
            position: WatermarkPosition::Center,
            color: [255, 255, 255],
            inset: 0.05,
        };

        let result = apply_text_watermark(&mut rgb_data, width, height, &watermark);
        assert!(result.is_ok());

        // Should have modified some pixels
        assert_ne!(rgb_data, original);

        // Check that some pixels are white (watermark color)
        let has_white = rgb_data.chunks(3).any(|rgb| rgb[0] > 200 && rgb[1] > 200 && rgb[2] > 200);
        assert!(has_white, "Watermark should have added white pixels");
    }

    #[test]
    fn test_watermark_opacity_zero() {
        let width = 100;
        let height = 100;
        let mut rgb_data = vec![100u8; (width * height * 3) as usize];
        let original = rgb_data.clone();

        let watermark = TextWatermark {
            text: "TEST".to_string(),
            font_size: 0.05,
            opacity: 0.0, // Zero opacity
            position: WatermarkPosition::Center,
            color: [255, 255, 255],
            inset: 0.02,
        };

        let result = apply_text_watermark(&mut rgb_data, width, height, &watermark);
        assert!(result.is_ok());

        // Should not have modified pixels (opacity is zero)
        assert_eq!(rgb_data, original);
    }

    #[test]
    fn test_watermark_bottom_right() {
        let width = 200;
        let height = 200;
        let mut rgb_data = vec![50u8; (width * height * 3) as usize];

        let watermark = TextWatermark {
            text: "C".to_string(), // Use simple ASCII instead of ©
            font_size: 0.05,
            opacity: 0.7,
            position: WatermarkPosition::BottomRight,
            color: [200, 200, 200],
            inset: 0.02,
        };

        let result = apply_text_watermark(&mut rgb_data, width, height, &watermark);
        assert!(result.is_ok());

        // Check that pixels in the bottom-right quadrant are modified
        let bottom_right_offset = ((height * 3 / 4) * width * 3) as usize;
        let quadrant_data = &rgb_data[bottom_right_offset..];

        let has_modified = quadrant_data.chunks(3).any(|rgb| {
            rgb[0] > 100 || rgb[1] > 100 || rgb[2] > 100
        });
        assert!(has_modified, "Watermark should modify bottom-right quadrant");
    }

    #[test]
    fn test_watermark_empty_text() {
        let width = 100;
        let height = 100;
        let mut rgb_data = vec![100u8; (width * height * 3) as usize];
        let original = rgb_data.clone();

        let watermark = TextWatermark {
            text: "".to_string(), // Empty text
            font_size: 0.05,
            opacity: 1.0,
            position: WatermarkPosition::Center,
            color: [255, 255, 255],
            inset: 0.02,
        };

        let result = apply_text_watermark(&mut rgb_data, width, height, &watermark);
        assert!(result.is_ok());

        // Should not modify anything
        assert_eq!(rgb_data, original);
    }

    #[test]
    fn test_watermark_invalid_data_size() {
        let width = 100;
        let height = 100;
        let mut rgb_data = vec![100u8; 50]; // Wrong size

        let watermark = TextWatermark::default();

        let result = apply_text_watermark(&mut rgb_data, width, height, &watermark);
        assert!(result.is_err());
    }
}
