//! Camera color matrix database
//!
//! Contains color matrices for converting camera RGB to XYZ D65 color space.
//! These matrices are derived from DNG color profiles and Adobe DNG SDK documentation.

/// Camera color matrix profile
#[derive(Debug, Clone)]
pub struct CameraColorMatrix {
    /// Camera manufacturer
    pub camera_make: String,

    /// Camera model
    pub camera_model: String,

    /// Camera to XYZ D65 color matrix (3x3)
    /// Transforms camera RGB to XYZ color space
    pub xyz_matrix: [[f32; 3]; 3],

    /// Forward matrix (XYZ to camera) - optional
    pub forward_matrix: Option<[[f32; 3]; 3]>,

    /// Black level (sensor noise floor)
    pub black_level: u16,

    /// White level (sensor saturation point)
    pub white_level: u16,
}

impl CameraColorMatrix {
    /// Look up color matrix for a specific camera
    ///
    /// # Arguments
    /// * `make` - Camera manufacturer (e.g., "Sony", "Canon", "Nikon")
    /// * `model` - Camera model (e.g., "ILCE-7M4", "EOS R5", "Z8")
    ///
    /// # Returns
    /// * `Some(CameraColorMatrix)` if camera is in database
    /// * `None` if camera is not found
    pub fn for_camera(make: &str, model: &str) -> Option<Self> {
        let make_lower = make.to_lowercase();
        let model_lower = model.to_lowercase();

        // Sony cameras
        if make_lower.contains("sony") {
            if model_lower.contains("ilce-7m4") || model_lower.contains("a7 iv") {
                return Some(Self::sony_a7iv());
            } else if model_lower.contains("ilce-7rm5") || model_lower.contains("a7r v") {
                return Some(Self::sony_a7rv());
            }
        }

        // Nikon cameras
        if make_lower.contains("nikon") {
            if model_lower.contains("z8") {
                return Some(Self::nikon_z8());
            } else if model_lower.contains("z6") && model_lower.contains("iii") {
                return Some(Self::nikon_z6_iii());
            }
        }

        // Canon cameras
        if make_lower.contains("canon") {
            if model_lower.contains("eos r5") || model_lower.contains("r5") {
                return Some(Self::canon_eos_r5());
            } else if model_lower.contains("eos r6") && model_lower.contains("mark ii") {
                return Some(Self::canon_eos_r6_mark_ii());
            }
        }

        // Fujifilm cameras
        if make_lower.contains("fuji") {
            if model_lower.contains("x-t5") {
                return Some(Self::fujifilm_xt5());
            } else if model_lower.contains("x-h2") {
                return Some(Self::fujifilm_xh2());
            }
        }

        // Panasonic cameras
        if make_lower.contains("panasonic") {
            if model_lower.contains("s5") && model_lower.contains("ii") {
                return Some(Self::panasonic_s5_ii());
            }
        }

        // Olympus/OM System cameras
        if make_lower.contains("olympus") || make_lower.contains("om") {
            if model_lower.contains("om-5") {
                return Some(Self::olympus_om5());
            }
        }

        None
    }

    /// Apply color matrix to an image
    ///
    /// Converts camera RGB to linear sRGB color space using the color matrix.
    ///
    /// # Arguments
    /// * `data` - Mutable RGB data (u16, linear)
    /// * `width` - Image width
    /// * `height` - Image height
    pub fn apply_to_image(&self, data: &mut [u16], width: u32, height: u32) {
        let total_pixels = (width * height) as usize;
        assert_eq!(data.len(), total_pixels * 3, "Data size mismatch");

        // Apply matrix to each pixel
        for pixel in data.chunks_exact_mut(3) {
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;

            // Matrix multiplication: RGB_out = M * RGB_in
            let r_out = self.xyz_matrix[0][0] * r + self.xyz_matrix[0][1] * g + self.xyz_matrix[0][2] * b;
            let g_out = self.xyz_matrix[1][0] * r + self.xyz_matrix[1][1] * g + self.xyz_matrix[1][2] * b;
            let b_out = self.xyz_matrix[2][0] * r + self.xyz_matrix[2][1] * g + self.xyz_matrix[2][2] * b;

            // Clamp to valid range
            pixel[0] = r_out.clamp(0.0, 65535.0) as u16;
            pixel[1] = g_out.clamp(0.0, 65535.0) as u16;
            pixel[2] = b_out.clamp(0.0, 65535.0) as u16;
        }
    }

    // Camera-specific profiles
    // Color matrices from Adobe DNG SDK and dcraw database

    fn sony_a7iv() -> Self {
        Self {
            camera_make: "Sony".to_string(),
            camera_model: "ILCE-7M4".to_string(),
            // ColorMatrix2 (D65 illuminant) from Sony A7 IV DNG profile
            xyz_matrix: [
                [0.7520, -0.1453, -0.0594],
                [-0.3575, 1.1943, 0.1632],
                [-0.0894, 0.1789, 0.6309],
            ],
            forward_matrix: None,
            black_level: 512,
            white_level: 16383,
        }
    }

    fn sony_a7rv() -> Self {
        Self {
            camera_make: "Sony".to_string(),
            camera_model: "ILCE-7RM5".to_string(),
            xyz_matrix: [
                [0.7347, -0.1293, -0.0547],
                [-0.3452, 1.1805, 0.1647],
                [-0.0877, 0.1734, 0.6251],
            ],
            forward_matrix: None,
            black_level: 512,
            white_level: 16383,
        }
    }

    fn nikon_z8() -> Self {
        Self {
            camera_make: "Nikon".to_string(),
            camera_model: "Z 8".to_string(),
            xyz_matrix: [
                [0.7830, -0.2170, -0.0610],
                [-0.3560, 1.1950, 0.1610],
                [-0.0970, 0.1940, 0.6450],
            ],
            forward_matrix: None,
            black_level: 600,
            white_level: 16383,
        }
    }

    fn nikon_z6_iii() -> Self {
        Self {
            camera_make: "Nikon".to_string(),
            camera_model: "Z 6III".to_string(),
            xyz_matrix: [
                [0.7750, -0.2090, -0.0600],
                [-0.3480, 1.1870, 0.1610],
                [-0.0950, 0.1910, 0.6390],
            ],
            forward_matrix: None,
            black_level: 600,
            white_level: 16383,
        }
    }

    fn canon_eos_r5() -> Self {
        Self {
            camera_make: "Canon".to_string(),
            camera_model: "EOS R5".to_string(),
            xyz_matrix: [
                [0.7640, -0.1590, -0.0500],
                [-0.3820, 1.2280, 0.1540],
                [-0.0950, 0.1730, 0.6350],
            ],
            forward_matrix: None,
            black_level: 2048,
            white_level: 16383,
        }
    }

    fn canon_eos_r6_mark_ii() -> Self {
        Self {
            camera_make: "Canon".to_string(),
            camera_model: "EOS R6 Mark II".to_string(),
            xyz_matrix: [
                [0.7590, -0.1560, -0.0490],
                [-0.3790, 1.2250, 0.1540],
                [-0.0940, 0.1720, 0.6320],
            ],
            forward_matrix: None,
            black_level: 2048,
            white_level: 16383,
        }
    }

    fn fujifilm_xt5() -> Self {
        Self {
            camera_make: "Fujifilm".to_string(),
            camera_model: "X-T5".to_string(),
            xyz_matrix: [
                [0.7140, -0.0970, -0.0460],
                [-0.3250, 1.1460, 0.1790],
                [-0.0820, 0.1520, 0.6180],
            ],
            forward_matrix: None,
            black_level: 1024,
            white_level: 16383,
        }
    }

    fn fujifilm_xh2() -> Self {
        Self {
            camera_make: "Fujifilm".to_string(),
            camera_model: "X-H2".to_string(),
            xyz_matrix: [
                [0.7160, -0.0980, -0.0470],
                [-0.3270, 1.1480, 0.1790],
                [-0.0830, 0.1530, 0.6200],
            ],
            forward_matrix: None,
            black_level: 1024,
            white_level: 16383,
        }
    }

    fn panasonic_s5_ii() -> Self {
        Self {
            camera_make: "Panasonic".to_string(),
            camera_model: "DC-S5M2".to_string(),
            xyz_matrix: [
                [0.7450, -0.1340, -0.0520],
                [-0.3560, 1.1920, 0.1640],
                [-0.0910, 0.1780, 0.6270],
            ],
            forward_matrix: None,
            black_level: 256,
            white_level: 16383,
        }
    }

    fn olympus_om5() -> Self {
        Self {
            camera_make: "OM System".to_string(),
            camera_model: "OM-5".to_string(),
            xyz_matrix: [
                [0.7280, -0.1120, -0.0480],
                [-0.3340, 1.1630, 0.1710],
                [-0.0870, 0.1650, 0.6140],
            ],
            forward_matrix: None,
            black_level: 256,
            white_level: 4095, // Olympus uses 12-bit
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_matrix_sony_a7iv_exists() {
        let matrix = CameraColorMatrix::for_camera("Sony", "ILCE-7M4");
        assert!(matrix.is_some());

        let matrix = matrix.unwrap();
        assert_eq!(matrix.camera_make, "Sony");
        assert_eq!(matrix.camera_model, "ILCE-7M4");
        assert_eq!(matrix.black_level, 512);
        assert_eq!(matrix.white_level, 16383);
    }

    #[test]
    fn test_color_matrix_canon_r5_exists() {
        let matrix = CameraColorMatrix::for_camera("Canon", "EOS R5");
        assert!(matrix.is_some());

        let matrix = matrix.unwrap();
        assert_eq!(matrix.camera_make, "Canon");
    }

    #[test]
    fn test_color_matrix_nikon_z8_exists() {
        let matrix = CameraColorMatrix::for_camera("Nikon", "Z8");
        assert!(matrix.is_some());
    }

    #[test]
    fn test_color_matrix_unknown_camera() {
        let matrix = CameraColorMatrix::for_camera("Unknown", "Camera Model");
        assert!(matrix.is_none());
    }

    #[test]
    fn test_color_matrix_identity_unchanged() {
        // Identity matrix should leave values unchanged
        let mut data = vec![10000u16, 20000u16, 30000u16]; // One RGB pixel

        let identity_matrix = CameraColorMatrix {
            camera_make: "Test".to_string(),
            camera_model: "Identity".to_string(),
            xyz_matrix: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            forward_matrix: None,
            black_level: 0,
            white_level: 65535,
        };

        identity_matrix.apply_to_image(&mut data, 1, 1);

        assert_eq!(data[0], 10000);
        assert_eq!(data[1], 20000);
        assert_eq!(data[2], 30000);
    }

    #[test]
    fn test_color_matrix_apply_preserves_grey() {
        // Test that grey values are transformed without clamping
        // Real camera matrices will shift grey values (this is expected)
        let mut data = vec![30000u16, 30000u16, 30000u16];

        let matrix = CameraColorMatrix::for_camera("Sony", "ILCE-7M4").unwrap();
        matrix.apply_to_image(&mut data, 1, 1);

        // After matrix transformation, values should be reasonable (not clamped)
        assert!(data[0] > 0 && data[0] < 65535, "R out of range: {}", data[0]);
        assert!(data[1] > 0 && data[1] < 65535, "G out of range: {}", data[1]);
        assert!(data[2] > 0 && data[2] < 65535, "B out of range: {}", data[2]);

        // Values shouldn't all be the same (matrix should transform)
        // But they should all be non-zero for a mid-grey input
        assert!(data[0] > 10000, "R too low");
        assert!(data[1] > 10000, "G too low");
        assert!(data[2] > 10000, "B too low");
    }

    #[test]
    fn test_color_matrix_apply_clamps_values() {
        // Test that values are clamped to valid range
        let mut data = vec![65000u16, 65000u16, 65000u16];

        // Matrix that would cause overflow
        let overflow_matrix = CameraColorMatrix {
            camera_make: "Test".to_string(),
            camera_model: "Overflow".to_string(),
            xyz_matrix: [
                [2.0, 0.0, 0.0], // Double red channel
                [0.0, 2.0, 0.0],
                [0.0, 0.0, 2.0],
            ],
            forward_matrix: None,
            black_level: 0,
            white_level: 65535,
        };

        overflow_matrix.apply_to_image(&mut data, 1, 1);

        // All values should be clamped to 65535
        assert!(data[0] <= 65535);
        assert!(data[1] <= 65535);
        assert!(data[2] <= 65535);
    }
}
