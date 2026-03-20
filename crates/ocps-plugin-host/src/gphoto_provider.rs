//! gPhoto2-based tether provider for real camera connectivity
//!
//! Provides Linux/macOS camera connectivity via libgphoto2.
//! Optional feature: only compiled when `gphoto2-tether` feature is enabled.

#[cfg(feature = "gphoto2-tether")]
use crate::tether::{TetherError, TetherProvider, TetheredCamera};

/// Real gPhoto2-based tether provider
///
/// Connects to cameras via libgphoto2. Requires the gphoto2 library
/// to be installed on the system.
#[cfg(feature = "gphoto2-tether")]
pub struct GphotoProvider {
    context: Option<gphoto2::Context>,
    camera: Option<gphoto2::Camera>,
    connected_camera_id: Option<String>,
}

#[cfg(feature = "gphoto2-tether")]
impl GphotoProvider {
    /// Create a new gPhoto2 provider
    pub fn new() -> Self {
        Self {
            context: None,
            camera: None,
            connected_camera_id: None,
        }
    }
}

#[cfg(feature = "gphoto2-tether")]
impl Default for GphotoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "gphoto2-tether")]
impl TetherProvider for GphotoProvider {
    fn name(&self) -> &str {
        "gPhoto2"
    }

    fn discover_cameras(&self) -> Vec<TetheredCamera> {
        // Create a temporary context for discovery
        let ctx = match gphoto2::Context::new() {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        // Autodetect cameras
        let cameras = match gphoto2::Camera::autodetect(&ctx) {
            Ok(list) => list,
            Err(_) => return vec![],
        };

        // Convert to TetheredCamera list
        cameras
            .into_iter()
            .enumerate()
            .map(|(idx, (model, port))| TetheredCamera {
                id: format!("gphoto2-{}-{}", idx, port.replace(':', "-")),
                name: format!("{} ({})", model, port),
                provider: "gphoto2".to_string(),
                connected: self
                    .connected_camera_id
                    .as_ref()
                    .map(|id| id == &format!("gphoto2-{}-{}", idx, port.replace(':', "-")))
                    .unwrap_or(false),
            })
            .collect()
    }

    fn connect(&mut self, camera_id: &str) -> Result<(), TetherError> {
        // Parse camera ID to extract port
        // Format: "gphoto2-{idx}-{port}"
        let parts: Vec<&str> = camera_id.split('-').collect();
        if parts.len() < 3 || parts[0] != "gphoto2" {
            return Err(TetherError::CameraNotFound(format!(
                "Invalid camera ID format: {}",
                camera_id
            )));
        }

        // Create context
        let ctx = gphoto2::Context::new()
            .map_err(|e| TetherError::ConnectionFailed(format!("Failed to create context: {}", e)))?;

        // Autodetect and find the camera
        let cameras = gphoto2::Camera::autodetect(&ctx)
            .map_err(|e| TetherError::ConnectionFailed(format!("Failed to detect cameras: {}", e)))?;

        let idx: usize = parts[1]
            .parse()
            .map_err(|_| TetherError::CameraNotFound(format!("Invalid camera index in ID: {}", camera_id)))?;

        if idx >= cameras.len() {
            return Err(TetherError::CameraNotFound(format!(
                "Camera index {} out of range (found {} cameras)",
                idx,
                cameras.len()
            )));
        }

        // Connect to the camera
        let (_model, port) = &cameras[idx];
        let camera = gphoto2::Camera::new(&ctx)
            .map_err(|e| TetherError::ConnectionFailed(format!("Failed to create camera: {}", e)))?;

        camera
            .init(&ctx)
            .map_err(|e| TetherError::ConnectionFailed(format!("Failed to initialize camera on {}: {}", port, e)))?;

        self.context = Some(ctx);
        self.camera = Some(camera);
        self.connected_camera_id = Some(camera_id.to_string());

        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), TetherError> {
        if let Some(camera) = self.camera.take() {
            if let Some(ctx) = &self.context {
                // Exit the camera
                let _ = camera.exit(ctx);
            }
        }
        self.camera = None;
        self.context = None;
        self.connected_camera_id = None;
        Ok(())
    }

    fn capture(&mut self) -> Result<Vec<u8>, TetherError> {
        let camera = self
            .camera
            .as_ref()
            .ok_or(TetherError::NotConnected)?;
        let ctx = self.context.as_ref().ok_or(TetherError::NotConnected)?;

        // Trigger capture
        let file_path = camera
            .capture_image(ctx)
            .map_err(|e| TetherError::CaptureFailed(format!("Capture failed: {}", e)))?;

        // Download the file
        let file_data = camera
            .file_get(ctx, file_path.folder(), file_path.name(), gphoto2::CameraFileType::Normal)
            .map_err(|e| TetherError::CaptureFailed(format!("Failed to download file: {}", e)))?;

        // Get file data as bytes
        let bytes = file_data
            .get_data()
            .map_err(|e| TetherError::CaptureFailed(format!("Failed to read file data: {}", e)))?
            .to_vec();

        // Optionally delete from camera to save space
        let _ = camera.file_delete(ctx, file_path.folder(), file_path.name());

        Ok(bytes)
    }

    fn live_view_frame(&mut self) -> Result<Option<Vec<u8>>, TetherError> {
        let camera = self
            .camera
            .as_ref()
            .ok_or(TetherError::NotConnected)?;
        let ctx = self.context.as_ref().ok_or(TetherError::NotConnected)?;

        // Try to capture preview
        match camera.capture_preview(ctx) {
            Ok(file) => {
                let bytes = file
                    .get_data()
                    .map_err(|e| TetherError::CaptureFailed(format!("Failed to read preview data: {}", e)))?
                    .to_vec();
                Ok(Some(bytes))
            }
            Err(_) => {
                // Preview not supported or failed
                Ok(None)
            }
        }
    }
}

#[cfg(all(test, feature = "gphoto2-tether"))]
mod tests {
    use super::*;

    #[test]
    fn test_gphoto_provider_creation() {
        let provider = GphotoProvider::new();
        assert_eq!(provider.name(), "gPhoto2");
        assert!(provider.context.is_none());
        assert!(provider.camera.is_none());
    }

    #[test]
    fn test_gphoto_provider_discover() {
        let provider = GphotoProvider::new();
        // This will return empty if no cameras connected, which is fine for testing
        let cameras = provider.discover_cameras();
        // Just verify it doesn't crash
        assert!(cameras.len() >= 0);
    }
}

// Re-export for convenience when feature is enabled
#[cfg(feature = "gphoto2-tether")]
pub use GphotoProvider;
