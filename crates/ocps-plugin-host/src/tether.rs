//! Tethering infrastructure for camera connectivity
//!
//! Provides abstraction layer for connecting to cameras, capturing images,
//! and retrieving live view frames. Supports multiple providers (mock, gphoto2, SDK).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Tethering errors
#[derive(Debug, Error)]
pub enum TetherError {
    #[error("Camera not found: {0}")]
    CameraNotFound(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Not connected to any camera")]
    NotConnected,

    #[error("Capture failed: {0}")]
    CaptureFailed(String),

    #[error("Live view not supported")]
    LiveViewNotSupported,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Information about a tethered camera
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TetheredCamera {
    /// Unique camera ID
    pub id: String,

    /// Camera display name (e.g., "Canon EOS R5")
    pub name: String,

    /// Provider name (e.g., "gphoto2", "ptp", "sdk", "mock")
    pub provider: String,

    /// Whether camera is currently connected
    pub connected: bool,
}

/// Tether session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TetherSession {
    /// Connected camera
    pub camera: TetheredCamera,

    /// Folder to import captured images to
    pub import_folder: PathBuf,

    /// Whether to automatically import after capture
    pub auto_import: bool,

    /// Number of shots taken in this session
    pub shot_count: u32,
}

impl TetherSession {
    /// Create a new tether session
    pub fn new(camera: TetheredCamera, import_folder: PathBuf) -> Self {
        Self {
            camera,
            import_folder,
            auto_import: true,
            shot_count: 0,
        }
    }
}

/// Tether provider trait
///
/// Implement this trait to add support for new camera connection methods
/// (e.g., gphoto2, PTP, manufacturer SDKs).
pub trait TetherProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;

    /// Discover available cameras
    fn discover_cameras(&self) -> Vec<TetheredCamera>;

    /// Connect to a camera
    fn connect(&mut self, camera_id: &str) -> Result<(), TetherError>;

    /// Disconnect from current camera
    fn disconnect(&mut self) -> Result<(), TetherError>;

    /// Capture an image and return RAW file bytes
    ///
    /// Returns the raw file data (DNG, CR3, ARW, etc.)
    fn capture(&mut self) -> Result<Vec<u8>, TetherError>;

    /// Get a live view frame (JPEG)
    ///
    /// Returns `Ok(Some(jpeg_bytes))` if live view is available,
    /// `Ok(None)` if no frame is available,
    /// `Err(_)` if live view is not supported.
    fn live_view_frame(&mut self) -> Result<Option<Vec<u8>>, TetherError>;
}

/// Mock tether provider for testing
///
/// Simulates camera connectivity without requiring actual hardware.
pub struct MockTetherProvider {
    connected: bool,
    shot_count: u32,
}

impl MockTetherProvider {
    /// Create a new mock provider
    pub fn new() -> Self {
        Self {
            connected: false,
            shot_count: 0,
        }
    }
}

impl Default for MockTetherProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TetherProvider for MockTetherProvider {
    fn name(&self) -> &str {
        "Mock Camera"
    }

    fn discover_cameras(&self) -> Vec<TetheredCamera> {
        vec![TetheredCamera {
            id: "mock-001".to_string(),
            name: "Mock Camera (Test)".to_string(),
            provider: "mock".to_string(),
            connected: self.connected,
        }]
    }

    fn connect(&mut self, camera_id: &str) -> Result<(), TetherError> {
        if camera_id == "mock-001" {
            self.connected = true;
            Ok(())
        } else {
            Err(TetherError::CameraNotFound(camera_id.to_string()))
        }
    }

    fn disconnect(&mut self) -> Result<(), TetherError> {
        self.connected = false;
        Ok(())
    }

    fn capture(&mut self) -> Result<Vec<u8>, TetherError> {
        if !self.connected {
            return Err(TetherError::NotConnected);
        }

        self.shot_count += 1;

        // Return a minimal valid JPEG (magic bytes + EOI marker)
        // This is the smallest valid JPEG file
        Ok(vec![
            0xFF, 0xD8, // SOI (Start of Image)
            0xFF, 0xE0, // APP0 marker
            0x00, 0x10, // Length
            b'J', b'F', b'I', b'F', 0x00, // JFIF identifier
            0x01, 0x01, // Version 1.1
            0x00, // No units
            0x00, 0x01, 0x00, 0x01, // 1x1 aspect ratio
            0x00, 0x00, // No thumbnail
            0xFF, 0xD9, // EOI (End of Image)
        ])
    }

    fn live_view_frame(&mut self) -> Result<Option<Vec<u8>>, TetherError> {
        // Mock doesn't support live view
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_tether_discover_cameras() {
        let provider = MockTetherProvider::new();
        let cameras = provider.discover_cameras();

        assert_eq!(cameras.len(), 1);
        assert_eq!(cameras[0].id, "mock-001");
        assert_eq!(cameras[0].name, "Mock Camera (Test)");
        assert_eq!(cameras[0].provider, "mock");
        assert!(!cameras[0].connected);
    }

    #[test]
    fn test_mock_tether_connect() {
        let mut provider = MockTetherProvider::new();

        assert!(!provider.connected);

        // Connect to mock camera
        let result = provider.connect("mock-001");
        assert!(result.is_ok());
        assert!(provider.connected);

        // Try to connect to unknown camera
        let result = provider.connect("unknown-camera");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_tether_disconnect() {
        let mut provider = MockTetherProvider::new();

        provider.connect("mock-001").unwrap();
        assert!(provider.connected);

        let result = provider.disconnect();
        assert!(result.is_ok());
        assert!(!provider.connected);
    }

    #[test]
    fn test_mock_tether_capture() {
        let mut provider = MockTetherProvider::new();

        // Capture should fail when not connected
        let result = provider.capture();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TetherError::NotConnected));

        // Connect and capture
        provider.connect("mock-001").unwrap();
        let result = provider.capture();
        assert!(result.is_ok());

        let data = result.unwrap();
        assert!(!data.is_empty());

        // Check JPEG magic bytes
        assert_eq!(data[0], 0xFF);
        assert_eq!(data[1], 0xD8);

        // Check shot count incremented
        assert_eq!(provider.shot_count, 1);

        // Capture again
        provider.capture().unwrap();
        assert_eq!(provider.shot_count, 2);
    }

    #[test]
    fn test_mock_capture_returns_jpeg_bytes() {
        let mut provider = MockTetherProvider::new();
        provider.connect("mock-001").unwrap();

        let data = provider.capture().unwrap();

        // Verify JPEG structure
        assert!(data.len() > 4);
        assert_eq!(&data[0..2], &[0xFF, 0xD8]); // SOI
        assert_eq!(&data[data.len() - 2..], &[0xFF, 0xD9]); // EOI
    }

    #[test]
    fn test_mock_live_view_not_supported() {
        let mut provider = MockTetherProvider::new();
        provider.connect("mock-001").unwrap();

        let result = provider.live_view_frame();
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_tether_session_creation() {
        let camera = TetheredCamera {
            id: "test-001".to_string(),
            name: "Test Camera".to_string(),
            provider: "test".to_string(),
            connected: true,
        };

        let import_folder = PathBuf::from("/tmp/tether");
        let session = TetherSession::new(camera.clone(), import_folder.clone());

        assert_eq!(session.camera.id, camera.id);
        assert_eq!(session.import_folder, import_folder);
        assert!(session.auto_import);
        assert_eq!(session.shot_count, 0);
    }
}
