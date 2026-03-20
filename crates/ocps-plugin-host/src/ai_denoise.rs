//! AI-powered noise reduction using ONNX Runtime
//!
//! Optional feature: only compiled when `ai` feature is enabled.
//! Falls back to CPU-based noise reduction if ONNX model not available.

use thiserror::Error;

#[cfg(feature = "ai")]
use std::path::Path;

#[derive(Debug, Error)]
pub enum DenoiseError {
    #[error("ONNX Runtime error: {0}")]
    #[cfg(feature = "ai")]
    OnnxError(String),

    #[error("Invalid input dimensions: {0}")]
    InvalidDimensions(String),

    #[error("Model not loaded")]
    ModelNotLoaded,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// AI-powered denoising plugin using ONNX Runtime
#[cfg(feature = "ai")]
pub struct AiDenoisePlugin {
    session: Option<ort::Session>,
}

#[cfg(feature = "ai")]
impl AiDenoisePlugin {
    /// Create a new AI denoise plugin with a model file
    pub fn new(model_path: &Path) -> Result<Self, DenoiseError> {
        // Initialize ONNX Runtime
        let session = ort::Session::builder()
            .map_err(|e| DenoiseError::OnnxError(e.to_string()))?
            .commit_from_file(model_path)
            .map_err(|e| DenoiseError::OnnxError(e.to_string()))?;

        Ok(Self {
            session: Some(session),
        })
    }

    /// Create without loading a model (will use CPU fallback)
    pub fn new_fallback() -> Self {
        Self { session: None }
    }

    /// Check if AI model is available
    pub fn has_model(&self) -> bool {
        self.session.is_some()
    }

    /// Denoise an image using AI or CPU fallback
    pub fn denoise(
        &self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        strength: f32,
    ) -> Result<Vec<u8>, DenoiseError> {
        if let Some(_session) = &self.session {
            // AI path: preprocess, run inference, postprocess
            // For now, this is a stub - real implementation would:
            // 1. Convert u8 RGB to f32 normalized [-1, 1]
            // 2. Create ONNX tensor
            // 3. Run inference
            // 4. Convert back to u8 RGB
            self.denoise_ai(rgb_data, width, height, strength)
        } else {
            // CPU fallback
            denoise_cpu_fallback(rgb_data, width, height, strength)
        }
    }

    #[cfg(feature = "ai")]
    fn denoise_ai(
        &self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        _strength: f32,
    ) -> Result<Vec<u8>, DenoiseError> {
        // Stub implementation - real AI inference would go here
        // For now, just return identity (no change)

        // Validate dimensions
        let expected_len = (width * height * 3) as usize;
        if rgb_data.len() != expected_len {
            return Err(DenoiseError::InvalidDimensions(format!(
                "Expected {} bytes for {}x{} RGB, got {}",
                expected_len,
                width,
                height,
                rgb_data.len()
            )));
        }

        // TODO: Real AI inference
        // 1. Preprocess: normalize to [-1, 1] float tensor
        // 2. Run inference with session
        // 3. Postprocess: back to u8

        Ok(rgb_data.to_vec())
    }
}

#[cfg(not(feature = "ai"))]
pub struct AiDenoisePlugin;

#[cfg(not(feature = "ai"))]
impl AiDenoisePlugin {
    pub fn new_fallback() -> Self {
        Self
    }

    pub fn has_model(&self) -> bool {
        false
    }

    pub fn denoise(
        &self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        strength: f32,
    ) -> Result<Vec<u8>, DenoiseError> {
        denoise_cpu_fallback(rgb_data, width, height, strength)
    }
}

/// CPU-based noise reduction fallback
///
/// Simple bilateral filter approximation for noise reduction.
/// Not as powerful as AI methods, but works without any models.
pub fn denoise_cpu_fallback(
    rgb_data: &[u8],
    width: u32,
    height: u32,
    strength: f32,
) -> Result<Vec<u8>, DenoiseError> {
    // Validate dimensions
    let expected_len = (width * height * 3) as usize;
    if rgb_data.len() != expected_len {
        return Err(DenoiseError::InvalidDimensions(format!(
            "Expected {} bytes for {}x{} RGB, got {}",
            expected_len,
            width,
            height,
            rgb_data.len()
        )));
    }

    // Clamp strength to [0, 1]
    let strength = strength.clamp(0.0, 1.0);

    // For zero strength, return identity
    if strength < 0.001 {
        return Ok(rgb_data.to_vec());
    }

    // Simple box blur for noise reduction
    // In production, this would use a proper bilateral filter or NLM
    let mut output = rgb_data.to_vec();
    let radius = (strength * 3.0).ceil() as i32; // 0-3 pixel radius

    if radius == 0 {
        return Ok(output);
    }

    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let mut r_sum = 0u32;
            let mut g_sum = 0u32;
            let mut b_sum = 0u32;
            let mut count = 0u32;

            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = x + dx;
                    let ny = y + dy;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let idx = ((ny * width as i32 + nx) * 3) as usize;
                        r_sum += rgb_data[idx] as u32;
                        g_sum += rgb_data[idx + 1] as u32;
                        b_sum += rgb_data[idx + 2] as u32;
                        count += 1;
                    }
                }
            }

            let out_idx = ((y * width as i32 + x) * 3) as usize;
            output[out_idx] = (r_sum / count) as u8;
            output[out_idx + 1] = (g_sum / count) as u8;
            output[out_idx + 2] = (b_sum / count) as u8;
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_fallback_denoise() {
        let width = 4;
        let height = 4;
        let data = vec![128u8; (width * height * 3) as usize];

        let result = denoise_cpu_fallback(&data, width, height, 0.5);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.len(), data.len());
    }

    #[test]
    fn test_denoise_returns_same_size() {
        let width = 10;
        let height = 10;
        let data = vec![100u8; (width * height * 3) as usize];

        let result = denoise_cpu_fallback(&data, width, height, 0.7);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.len(), data.len());
        assert_eq!(output.len(), (width * height * 3) as usize);
    }

    #[test]
    fn test_denoise_strength_zero_minimal_change() {
        let width = 4;
        let height = 4;
        let data = vec![50u8, 100u8, 150u8, 200u8, 50u8, 100u8, 150u8, 200u8, 50u8, 100u8, 150u8, 200u8];
        let data_full = data.repeat(4); // 4x4 RGB

        let result = denoise_cpu_fallback(&data_full, width, height, 0.0);
        assert!(result.is_ok());

        let output = result.unwrap();
        // At strength 0, output should be identical
        assert_eq!(output, data_full);
    }

    #[test]
    fn test_denoise_invalid_dimensions() {
        let data = vec![128u8; 100]; // Wrong size for 10x10 RGB (should be 300)

        let result = denoise_cpu_fallback(&data, 10, 10, 0.5);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DenoiseError::InvalidDimensions(_)));
    }

    #[test]
    fn test_ai_plugin_fallback_creation() {
        let plugin = AiDenoisePlugin::new_fallback();
        assert!(!plugin.has_model());
    }

    #[test]
    fn test_ai_plugin_fallback_denoise() {
        let plugin = AiDenoisePlugin::new_fallback();
        let width = 4;
        let height = 4;
        let data = vec![128u8; (width * height * 3) as usize];

        let result = plugin.denoise(&data, width, height, 0.5);
        assert!(result.is_ok());
    }
}
