//! Type definitions for the image processing pipeline

/// 16-bit RGB image (linear color space)
#[derive(Debug, Clone)]
pub struct RgbImage16 {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u16>, // RGB interleaved
}

impl RgbImage16 {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 3) as usize;
        Self {
            width,
            height,
            data: vec![0; size],
        }
    }

    pub fn from_data(width: u32, height: u32, data: Vec<u16>) -> Self {
        assert_eq!(data.len(), (width * height * 3) as usize);
        Self {
            width,
            height,
            data,
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> [u16; 3] {
        let idx = ((y * self.width + x) * 3) as usize;
        [self.data[idx], self.data[idx + 1], self.data[idx + 2]]
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, rgb: [u16; 3]) {
        let idx = ((y * self.width + x) * 3) as usize;
        self.data[idx] = rgb[0];
        self.data[idx + 1] = rgb[1];
        self.data[idx + 2] = rgb[2];
    }
}

/// 8-bit RGB image (sRGB gamma-encoded)
#[derive(Debug, Clone)]
pub struct RgbImage8 {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGB interleaved, sRGB gamma
}

impl RgbImage8 {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 3) as usize;
        Self {
            width,
            height,
            data: vec![0; size],
        }
    }

    pub fn from_data(width: u32, height: u32, data: Vec<u8>) -> Self {
        assert_eq!(data.len(), (width * height * 3) as usize);
        Self {
            width,
            height,
            data,
        }
    }
}

/// White balance settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WhiteBalance {
    pub temperature: u32, // Kelvin 2000-50000
    pub tint: i32,        // -150 to +150
}

impl Default for WhiteBalance {
    fn default() -> Self {
        Self {
            temperature: 5500, // Daylight default
            tint: 0,
        }
    }
}

/// Sharpening settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SharpeningSettings {
    pub amount: u32,   // 0-150
    pub radius: f32,   // 0.5-3.0
    pub detail: u32,   // 0-100
    pub masking: u32,  // 0-100
}

impl Default for SharpeningSettings {
    fn default() -> Self {
        Self {
            amount: 0, // Disabled by default for identity check
            radius: 1.0,
            detail: 25,
            masking: 0,
        }
    }
}

/// Noise reduction settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NoiseReductionSettings {
    pub luminance: u32,        // 0-100
    pub luminance_detail: u32, // 0-100
    pub color: u32,            // 0-100
    pub color_detail: u32,     // 0-100
}

impl Default for NoiseReductionSettings {
    fn default() -> Self {
        Self {
            luminance: 0,
            luminance_detail: 50,
            color: 0, // Disabled by default for identity check
            color_detail: 50,
        }
    }
}

/// Crop settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CropSettings {
    pub left: f32,   // 0.0-1.0 (normalized)
    pub top: f32,    // 0.0-1.0
    pub right: f32,  // 0.0-1.0
    pub bottom: f32, // 0.0-1.0
    pub angle: f32,  // rotation in degrees
}

impl Default for CropSettings {
    fn default() -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            right: 1.0,
            bottom: 1.0,
            angle: 0.0,
        }
    }
}

impl CropSettings {
    pub fn is_identity(&self) -> bool {
        self.left == 0.0
            && self.top == 0.0
            && self.right == 1.0
            && self.bottom == 1.0
            && self.angle == 0.0
    }
}

/// Point on a tone curve
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CurvePoint {
    pub x: f32, // 0.0-1.0
    pub y: f32, // 0.0-1.0
}

/// Tone curve (maps input luminance to output luminance)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToneCurve {
    pub points: Vec<CurvePoint>,
}

impl Default for ToneCurve {
    fn default() -> Self {
        // Linear curve by default
        Self {
            points: vec![
                CurvePoint { x: 0.0, y: 0.0 },
                CurvePoint { x: 1.0, y: 1.0 },
            ],
        }
    }
}

/// HSL adjustments per color channel (8 channels)
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HslAdjustments {
    pub hue: [i32; 8],        // -180 to +180 per channel
    pub saturation: [i32; 8], // -100 to +100 per channel
    pub luminance: [i32; 8],  // -100 to +100 per channel
}

/// Color grading settings (3-way color wheels)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ColorGrading {
    pub shadows_hue: u32,    // 0-360
    pub shadows_sat: u32,    // 0-100
    pub midtones_hue: u32,   // 0-360
    pub midtones_sat: u32,   // 0-100
    pub highlights_hue: u32, // 0-360
    pub highlights_sat: u32, // 0-100
    pub blending: u32,       // 0-100
    pub balance: i32,        // -100 to +100
}

impl Default for ColorGrading {
    fn default() -> Self {
        Self {
            shadows_hue: 0,
            shadows_sat: 0,
            midtones_hue: 0,
            midtones_sat: 0,
            highlights_hue: 0,
            highlights_sat: 0,
            blending: 50,
            balance: 0,
        }
    }
}

/// Lens correction settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LensCorrections {
    pub distortion: f32,         // -100 to +100
    pub vignetting: f32,         // -100 to +100
    pub chromatic_aberration: bool,
    pub profile_name: Option<String>,
}

impl Default for LensCorrections {
    fn default() -> Self {
        Self {
            distortion: 0.0,
            vignetting: 0.0,
            chromatic_aberration: false,
            profile_name: None,
        }
    }
}

impl LensCorrections {
    pub fn is_identity(&self) -> bool {
        self.distortion == 0.0
            && self.vignetting == 0.0
            && !self.chromatic_aberration
            && self.profile_name.is_none()
    }
}

/// Old color grading settings (kept for compatibility)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ColorGradingSettings {
    pub shadows_hue: f32,    // 0.0-360.0
    pub shadows_sat: f32,    // -100.0 to 100.0
    pub midtones_hue: f32,   // 0.0-360.0
    pub midtones_sat: f32,   // -100.0 to 100.0
    pub highlights_hue: f32, // 0.0-360.0
    pub highlights_sat: f32, // -100.0 to 100.0
    pub global_hue: f32,     // 0.0-360.0
    pub global_sat: f32,     // -100.0 to 100.0
    pub blending: f32,       // 0.0-100.0
}

impl Default for ColorGradingSettings {
    fn default() -> Self {
        Self {
            shadows_hue: 0.0,
            shadows_sat: 0.0,
            midtones_hue: 0.0,
            midtones_sat: 0.0,
            highlights_hue: 0.0,
            highlights_sat: 0.0,
            global_hue: 0.0,
            global_sat: 0.0,
            blending: 50.0,
        }
    }
}

/// Brush stroke data for painting masks
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrushStroke {
    pub points: Vec<(f32, f32)>, // normalized 0.0-1.0 coordinates
    pub size: f32,               // brush size in normalized units
    pub feather: f32,            // feather amount 0.0-1.0
    pub flow: f32,               // flow/opacity 0.0-1.0
    pub erase: bool,             // true = eraser mode
}

/// Mask type for local adjustments
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MaskType {
    /// Brush mask with painted strokes
    Brush { strokes: Vec<BrushStroke> },
    /// Linear gradient mask
    Gradient {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
    },
    /// Radial gradient mask
    Radial {
        center_x: f32,
        center_y: f32,
        radius_x: f32,
        radius_y: f32,
        feather: f32,
        invert: bool,
    },
}

/// Settings for a local adjustment
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct LocalSettings {
    pub exposure: f32,    // -5.0 to +5.0
    pub contrast: i32,    // -100 to +100
    pub highlights: i32,  // -100 to +100
    pub shadows: i32,     // -100 to +100
    pub clarity: i32,     // -100 to +100
    pub saturation: i32,  // -100 to +100
    pub sharpness: i32,   // -100 to +100
}

/// Local adjustment with mask and settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LocalAdjustment {
    pub id: String,              // UUID
    pub mask_type: MaskType,     // brush, gradient, or radial
    pub settings: LocalSettings, // adjustment parameters
    pub enabled: bool,           // can be toggled on/off
    pub order: u32,              // rendering order
}

/// Healing/Clone spot type
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpotType {
    /// Heal: blend source texture with target luminance
    Heal,
    /// Clone: copy source region exactly
    Clone,
}

/// Healing or clone spot for removing blemishes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealingSpot {
    pub id: String,        // UUID
    pub spot_type: SpotType, // Heal or Clone
    pub target_x: f32,     // normalized 0-1 (center of spot)
    pub target_y: f32,     // normalized 0-1
    pub source_x: f32,     // where to copy from (normalized 0-1)
    pub source_y: f32,     // normalized 0-1
    pub radius: f32,       // normalized radius (0-1, relative to image size)
    pub feather: f32,      // edge feathering 0-1
    pub opacity: f32,      // blend opacity 0-1
}

/// Complete edit recipe for an image
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct EditRecipe {
    pub white_balance: WhiteBalance,
    pub exposure: f32,        // -5.0 to +5.0
    pub contrast: i32,        // -100 to +100
    pub highlights: i32,      // -100 to +100
    pub shadows: i32,         // -100 to +100
    pub whites: i32,          // -100 to +100
    pub blacks: i32,          // -100 to +100
    pub clarity: i32,         // -100 to +100
    pub dehaze: i32,          // -100 to +100
    pub vibrance: i32,        // -100 to +100
    pub saturation: i32,      // -100 to +100
    pub sharpening: SharpeningSettings,
    pub noise_reduction: NoiseReductionSettings,
    pub crop: CropSettings,
    pub color_grading: ColorGradingSettings,
    pub tone_curve_rgb: ToneCurve,
    pub hsl: HslAdjustments,
    pub color_grading_new: ColorGrading,
    pub lens_corrections: LensCorrections,
    pub local_adjustments: Vec<LocalAdjustment>,
    pub healing_spots: Vec<HealingSpot>,
}

impl EditRecipe {
    /// Check if this recipe is identity (no meaningful edits)
    /// This checks if the recipe will produce the same result as no processing
    pub fn is_identity(&self) -> bool {
        // Check basic tone adjustments
        let basic_identity = self.exposure == 0.0
            && self.contrast == 0
            && self.highlights == 0
            && self.shadows == 0
            && self.whites == 0
            && self.blacks == 0
            && self.clarity == 0
            && self.dehaze == 0
            && self.vibrance == 0
            && self.saturation == 0;

        // Check white balance (neutral is 5500K, 0 tint)
        let wb_identity = self.white_balance.temperature == 5500
            && self.white_balance.tint == 0;

        // Check geometric transforms
        let geo_identity = self.crop.is_identity();

        // For sharpening and NR, we only check if they're effectively disabled
        // Default sharpening amount (40) is ignored if it's the default preset
        // But for identity check, we want to know if it will actually sharpen
        let detail_identity = self.sharpening.amount == 0
            && self.noise_reduction.luminance == 0
            && self.noise_reduction.color == 0;

        // Check lens corrections
        let lens_identity = self.lens_corrections.is_identity();

        basic_identity && wb_identity && geo_identity && detail_identity && lens_identity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_image16_new() {
        let img = RgbImage16::new(100, 100);
        assert_eq!(img.width, 100);
        assert_eq!(img.height, 100);
        assert_eq!(img.data.len(), 100 * 100 * 3);
    }

    #[test]
    fn test_rgb_image16_pixel_access() {
        let mut img = RgbImage16::new(10, 10);
        img.set_pixel(5, 5, [100, 200, 300]);
        let pixel = img.get_pixel(5, 5);
        assert_eq!(pixel, [100, 200, 300]);
    }

    #[test]
    fn test_default_white_balance() {
        let wb = WhiteBalance::default();
        assert_eq!(wb.temperature, 5500);
        assert_eq!(wb.tint, 0);
    }

    #[test]
    fn test_crop_is_identity() {
        let crop = CropSettings::default();
        assert!(crop.is_identity());

        let mut crop2 = CropSettings::default();
        crop2.left = 0.1;
        assert!(!crop2.is_identity());
    }

    #[test]
    fn test_edit_recipe_is_identity() {
        let recipe = EditRecipe::default();
        assert!(recipe.is_identity());

        let mut recipe2 = EditRecipe::default();
        recipe2.exposure = 1.0;
        assert!(!recipe2.is_identity());
    }

    #[test]
    fn test_local_adjustment_serialization() {
        use uuid::Uuid;

        // Create a local adjustment with brush
        let adjustment = LocalAdjustment {
            id: Uuid::new_v4().to_string(),
            mask_type: MaskType::Brush {
                strokes: vec![
                    BrushStroke {
                        points: vec![(0.5, 0.5), (0.6, 0.6)],
                        size: 0.1,
                        feather: 0.5,
                        flow: 0.8,
                        erase: false,
                    },
                ],
            },
            settings: LocalSettings {
                exposure: 0.5,
                contrast: 10,
                ..Default::default()
            },
            enabled: true,
            order: 0,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&adjustment).unwrap();
        assert!(json.contains("Brush"));
        assert!(json.contains("exposure"));

        // Deserialize back
        let deserialized: LocalAdjustment = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, adjustment.id);
        assert_eq!(deserialized.enabled, true);
        assert_eq!(deserialized.settings.exposure, 0.5);

        // Check mask type
        match deserialized.mask_type {
            MaskType::Brush { strokes } => {
                assert_eq!(strokes.len(), 1);
                assert_eq!(strokes[0].points.len(), 2);
            }
            _ => panic!("Expected Brush mask type"),
        }
    }

    #[test]
    fn test_gradient_mask_serialization() {
        use uuid::Uuid;

        let adjustment = LocalAdjustment {
            id: Uuid::new_v4().to_string(),
            mask_type: MaskType::Gradient {
                start_x: 0.0,
                start_y: 0.0,
                end_x: 1.0,
                end_y: 1.0,
            },
            settings: LocalSettings {
                exposure: -1.0,
                ..Default::default()
            },
            enabled: true,
            order: 1,
        };

        let json = serde_json::to_string(&adjustment).unwrap();
        let deserialized: LocalAdjustment = serde_json::from_str(&json).unwrap();

        match deserialized.mask_type {
            MaskType::Gradient { start_x, start_y, end_x, end_y } => {
                assert_eq!(start_x, 0.0);
                assert_eq!(start_y, 0.0);
                assert_eq!(end_x, 1.0);
                assert_eq!(end_y, 1.0);
            }
            _ => panic!("Expected Gradient mask type"),
        }
    }

    #[test]
    fn test_radial_mask_serialization() {
        use uuid::Uuid;

        let adjustment = LocalAdjustment {
            id: Uuid::new_v4().to_string(),
            mask_type: MaskType::Radial {
                center_x: 0.5,
                center_y: 0.5,
                radius_x: 0.3,
                radius_y: 0.3,
                feather: 0.5,
                invert: false,
            },
            settings: LocalSettings {
                clarity: 50,
                saturation: -20,
                ..Default::default()
            },
            enabled: true,
            order: 2,
        };

        let json = serde_json::to_string(&adjustment).unwrap();
        let deserialized: LocalAdjustment = serde_json::from_str(&json).unwrap();

        match deserialized.mask_type {
            MaskType::Radial { center_x, center_y, radius_x, radius_y, feather, invert } => {
                assert_eq!(center_x, 0.5);
                assert_eq!(center_y, 0.5);
                assert_eq!(radius_x, 0.3);
                assert_eq!(radius_y, 0.3);
                assert_eq!(feather, 0.5);
                assert_eq!(invert, false);
            }
            _ => panic!("Expected Radial mask type"),
        }
    }
}
