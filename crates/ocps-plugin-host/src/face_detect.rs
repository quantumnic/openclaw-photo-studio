//! Face detection for focus checking and person organization
//!
//! Privacy-first local face detection using simple computer vision techniques.
//! No ML models, no cloud services, no privacy concerns.

use serde::{Deserialize, Serialize};

/// A detected face region (normalized coordinates)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceRegion {
    /// X coordinate (0.0 - 1.0, left to right)
    pub x: f32,
    /// Y coordinate (0.0 - 1.0, top to bottom)
    pub y: f32,
    /// Width (0.0 - 1.0)
    pub width: f32,
    /// Height (0.0 - 1.0)
    pub height: f32,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

impl FaceRegion {
    /// Create a new face region
    pub fn new(x: f32, y: f32, width: f32, height: f32, confidence: f32) -> Self {
        Self {
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
            width: width.clamp(0.0, 1.0),
            height: height.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Get the center point of the face
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Get the area of the face region
    pub fn area(&self) -> f32 {
        self.width * self.height
    }
}

/// Simple face detection using skin tone and aspect ratio heuristics
///
/// This is a basic implementation that doesn't use ML. It works by:
/// 1. Converting to HSV color space
/// 2. Detecting skin-tone regions
/// 3. Finding connected components
/// 4. Filtering by aspect ratio (faces are roughly square)
/// 5. Ranking by size and position (faces are usually in upper portion)
///
/// This won't be as accurate as ML-based detection, but it's:
/// - 100% local (privacy-preserving)
/// - Fast
/// - No dependencies on external models
/// - Good enough for focus checking
pub fn detect_faces_simple(rgb_data: &[u8], width: u32, height: u32) -> Vec<FaceRegion> {
    if width == 0 || height == 0 || rgb_data.is_empty() {
        return vec![];
    }

    let expected_len = (width * height * 3) as usize;
    if rgb_data.len() != expected_len {
        return vec![];
    }

    // Step 1: Convert to HSV and detect skin tones
    let mut skin_map = vec![false; (width * height) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 3) as usize;
            let r = rgb_data[idx] as f32 / 255.0;
            let g = rgb_data[idx + 1] as f32 / 255.0;
            let b = rgb_data[idx + 2] as f32 / 255.0;

            // Simple skin tone detection in RGB space
            // Skin tones typically have:
            // - R > G > B
            // - R > 95/255, G > 40/255, B > 20/255
            // - max(R,G,B) - min(R,G,B) > 15/255
            // - abs(R-G) > 15/255
            let is_skin = r > 0.37
                && g > 0.16
                && b > 0.08
                && r > g
                && g > b
                && (r - b) > 0.06
                && (r - g) > 0.06;

            skin_map[(y * width + x) as usize] = is_skin;
        }
    }

    // Step 2: Find connected regions (simple flood fill)
    let mut regions = find_connected_regions(&skin_map, width, height);

    // Step 3: Filter by aspect ratio (faces are roughly square)
    // and minimum size (at least 3% of image area)
    let min_area = 0.03;
    regions.retain(|region| {
        let aspect_ratio = region.width / region.height.max(0.001);
        let area = region.area();

        // Face aspect ratio is roughly 0.7 - 1.4
        area >= min_area && (0.6..=1.6).contains(&aspect_ratio)
    });

    // Step 4: Sort by confidence (size + position heuristics)
    regions.sort_by(|a, b| {
        let score_a = face_score(a, width as f32, height as f32);
        let score_b = face_score(b, width as f32, height as f32);
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Update confidence scores
    for (i, region) in regions.iter_mut().enumerate() {
        region.confidence = face_score(region, width as f32, height as f32) * (1.0 - i as f32 * 0.1);
    }

    // Return top 10 candidates
    regions.truncate(10);

    regions
}

/// Find connected regions in a binary map
fn find_connected_regions(map: &[bool], width: u32, height: u32) -> Vec<FaceRegion> {
    let mut visited = vec![false; (width * height) as usize];
    let mut regions = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;

            if map[idx] && !visited[idx] {
                // Start flood fill
                let mut min_x = x;
                let mut max_x = x;
                let mut min_y = y;
                let mut max_y = y;
                let mut stack = vec![(x, y)];

                while let Some((cx, cy)) = stack.pop() {
                    let cidx = (cy * width + cx) as usize;

                    if visited[cidx] {
                        continue;
                    }

                    visited[cidx] = true;

                    min_x = min_x.min(cx);
                    max_x = max_x.max(cx);
                    min_y = min_y.min(cy);
                    max_y = max_y.max(cy);

                    // Check neighbors
                    let neighbors = [
                        (cx.wrapping_sub(1), cy),
                        (cx + 1, cy),
                        (cx, cy.wrapping_sub(1)),
                        (cx, cy + 1),
                    ];

                    for (nx, ny) in neighbors {
                        if nx < width && ny < height {
                            let nidx = (ny * width + nx) as usize;
                            if map[nidx] && !visited[nidx] {
                                stack.push((nx, ny));
                            }
                        }
                    }
                }

                // Create region (normalized coordinates)
                let region_width = (max_x - min_x + 1) as f32 / width as f32;
                let region_height = (max_y - min_y + 1) as f32 / height as f32;
                let region_x = min_x as f32 / width as f32;
                let region_y = min_y as f32 / height as f32;

                regions.push(FaceRegion::new(region_x, region_y, region_width, region_height, 0.5));
            }
        }
    }

    regions
}

/// Calculate face score based on size and position
fn face_score(region: &FaceRegion, _width: f32, _height: f32) -> f32 {
    // Prefer larger regions
    let size_score = region.area();

    // Prefer regions in the upper 2/3 of the image (where faces usually are)
    let (_, center_y) = region.center();
    let position_score = if center_y < 0.66 { 1.0 } else { 0.5 };

    // Prefer regions not at the very edges
    let (center_x, _) = region.center();
    let edge_score = if center_x > 0.1 && center_x < 0.9 { 1.0 } else { 0.7 };

    size_score * position_score * edge_score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_region_creation() {
        let face = FaceRegion::new(0.25, 0.25, 0.2, 0.3, 0.85);

        assert_eq!(face.x, 0.25);
        assert_eq!(face.y, 0.25);
        assert_eq!(face.width, 0.2);
        assert_eq!(face.height, 0.3);
        assert_eq!(face.confidence, 0.85);
    }

    #[test]
    fn test_face_region_clamping() {
        let face = FaceRegion::new(-0.5, 1.5, 2.0, -1.0, 1.2);

        assert_eq!(face.x, 0.0);
        assert_eq!(face.y, 1.0);
        assert_eq!(face.width, 1.0);
        assert_eq!(face.height, 0.0);
        assert_eq!(face.confidence, 1.0);
    }

    #[test]
    fn test_face_region_center() {
        let face = FaceRegion::new(0.2, 0.3, 0.4, 0.2, 0.9);
        let (cx, cy) = face.center();

        assert!((cx - 0.4).abs() < 0.001);
        assert!((cy - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_face_region_area() {
        let face = FaceRegion::new(0.0, 0.0, 0.5, 0.4, 1.0);
        assert!((face.area() - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_detect_faces_no_faces() {
        // Uniform grey image - no skin tones
        let width = 10;
        let height = 10;
        let data = vec![128u8; (width * height * 3) as usize];

        let faces = detect_faces_simple(&data, width, height);
        assert_eq!(faces.len(), 0);
    }

    #[test]
    fn test_detect_faces_returns_normalized_coords() {
        // Create an image with a skin-tone region
        let width = 20;
        let height = 20;
        let mut data = vec![50u8; (width * height * 3) as usize];

        // Add a skin-tone square in the middle (10x10 pixels)
        for y in 5..15 {
            for x in 5..15 {
                let idx = ((y * width + x) * 3) as usize;
                data[idx] = 220; // R
                data[idx + 1] = 180; // G
                data[idx + 2] = 150; // B
            }
        }

        let faces = detect_faces_simple(&data, width, height);

        // Should detect at least one region
        if !faces.is_empty() {
            // All coordinates should be normalized [0, 1]
            for face in &faces {
                assert!(face.x >= 0.0 && face.x <= 1.0);
                assert!(face.y >= 0.0 && face.y <= 1.0);
                assert!(face.width >= 0.0 && face.width <= 1.0);
                assert!(face.height >= 0.0 && face.height <= 1.0);
                assert!(face.confidence >= 0.0 && face.confidence <= 1.0);
            }
        }
    }

    #[test]
    fn test_detect_faces_empty_input() {
        let faces = detect_faces_simple(&[], 0, 0);
        assert_eq!(faces.len(), 0);
    }

    #[test]
    fn test_detect_faces_invalid_dimensions() {
        let data = vec![128u8; 100]; // Wrong size
        let faces = detect_faces_simple(&data, 10, 10); // Should be 300 bytes
        assert_eq!(faces.len(), 0);
    }
}
