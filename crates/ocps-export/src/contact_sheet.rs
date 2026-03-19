//! Contact sheet generation — create photo grid layouts

/// Contact sheet settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContactSheetSettings {
    pub columns: u32,               // 3-8 columns
    pub rows: u32,                  // 3-10 rows
    pub cell_width: u32,            // pixels per cell
    pub cell_height: u32,           // pixels per cell
    pub background_color: [u8; 3],  // RGB background color
    pub show_filename: bool,        // show filename below each photo
    pub show_rating: bool,          // show rating stars
    pub padding: u32,               // pixels between cells
}

impl Default for ContactSheetSettings {
    fn default() -> Self {
        Self {
            columns: 4,
            rows: 4,
            cell_width: 300,
            cell_height: 200,
            background_color: [40, 40, 40], // Dark grey
            show_filename: false,
            show_rating: false,
            padding: 10,
        }
    }
}

/// Generate a contact sheet from a list of photos
///
/// # Arguments
/// * `photos` - Vec of (rgb_data, width, height) tuples for each photo
/// * `settings` - Contact sheet layout settings
///
/// # Returns
/// Tuple of (rgb_data, width, height) for the complete contact sheet
pub fn generate_contact_sheet(
    photos: &[(Vec<u8>, u32, u32)],
    settings: &ContactSheetSettings,
) -> (Vec<u8>, u32, u32) {
    // Handle empty photo list
    if photos.is_empty() {
        // Return minimal valid image (1x1 with background color)
        return (
            vec![
                settings.background_color[0],
                settings.background_color[1],
                settings.background_color[2],
            ],
            1,
            1,
        );
    }

    // Calculate sheet dimensions
    let sheet_width = settings.columns * settings.cell_width
        + (settings.columns + 1) * settings.padding;
    let sheet_height = settings.rows * settings.cell_height
        + (settings.rows + 1) * settings.padding;

    // Create background image filled with background color
    let mut sheet_data = vec![0u8; (sheet_width * sheet_height * 3) as usize];
    for pixel in sheet_data.chunks_exact_mut(3) {
        pixel.copy_from_slice(&settings.background_color);
    }

    // Place each photo in a grid cell
    let total_cells = (settings.columns * settings.rows) as usize;

    for (i, (photo_data, photo_width, photo_height)) in photos.iter().enumerate().take(total_cells) {
        let row = i as u32 / settings.columns;
        let col = i as u32 % settings.columns;

        // Calculate cell position on sheet
        let cell_x = settings.padding + col * (settings.cell_width + settings.padding);
        let cell_y = settings.padding + row * (settings.cell_height + settings.padding);

        // Resize photo to fit cell (maintain aspect ratio, center in cell)
        let resized = resize_to_fit(
            photo_data,
            *photo_width,
            *photo_height,
            settings.cell_width,
            settings.cell_height,
        );

        // Calculate centering offset within cell
        let offset_x = (settings.cell_width - resized.1) / 2;
        let offset_y = (settings.cell_height - resized.2) / 2;

        // Paste resized photo into cell
        paste_image(
            &mut sheet_data,
            sheet_width,
            &resized.0,
            resized.1,
            resized.2,
            cell_x + offset_x,
            cell_y + offset_y,
        );
    }

    (sheet_data, sheet_width, sheet_height)
}

/// Resize an image to fit within target dimensions (maintains aspect ratio)
///
/// Returns (resized_data, new_width, new_height)
fn resize_to_fit(
    data: &[u8],
    width: u32,
    height: u32,
    target_width: u32,
    target_height: u32,
) -> (Vec<u8>, u32, u32) {
    // Calculate aspect ratios
    let aspect = width as f32 / height as f32;
    let target_aspect = target_width as f32 / target_height as f32;

    // Determine new dimensions
    let (new_width, new_height) = if aspect > target_aspect {
        // Width-limited
        let w = target_width;
        let h = (w as f32 / aspect) as u32;
        (w, h)
    } else {
        // Height-limited
        let h = target_height;
        let w = (h as f32 * aspect) as u32;
        (w, h)
    };

    // Simple nearest-neighbor resize
    let mut resized = Vec::with_capacity((new_width * new_height * 3) as usize);

    for y in 0..new_height {
        for x in 0..new_width {
            // Map to source pixel
            let src_x = ((x as f32 / new_width as f32) * width as f32) as u32;
            let src_y = ((y as f32 / new_height as f32) * height as f32) as u32;
            let src_x = src_x.min(width - 1);
            let src_y = src_y.min(height - 1);

            let src_idx = ((src_y * width + src_x) * 3) as usize;
            if src_idx + 2 < data.len() {
                resized.push(data[src_idx]);
                resized.push(data[src_idx + 1]);
                resized.push(data[src_idx + 2]);
            } else {
                resized.push(0);
                resized.push(0);
                resized.push(0);
            }
        }
    }

    (resized, new_width, new_height)
}

/// Paste a source image into a destination image at specified position
fn paste_image(
    dest: &mut [u8],
    dest_width: u32,
    src: &[u8],
    src_width: u32,
    src_height: u32,
    x: u32,
    y: u32,
) {
    for row in 0..src_height {
        for col in 0..src_width {
            let dest_x = x + col;
            let dest_y = y + row;

            let src_idx = ((row * src_width + col) * 3) as usize;
            let dest_idx = ((dest_y * dest_width + dest_x) * 3) as usize;

            if src_idx + 2 < src.len() && dest_idx + 2 < dest.len() {
                dest[dest_idx] = src[src_idx];
                dest[dest_idx + 1] = src[src_idx + 1];
                dest[dest_idx + 2] = src[src_idx + 2];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_sheet_2x2() {
        // Create 4 test photos (2x2)
        let mut photos = Vec::new();
        for i in 0..4 {
            let color = (i * 60) as u8;
            let photo_data = vec![color; 100 * 100 * 3]; // 100x100 solid color
            photos.push((photo_data, 100, 100));
        }

        let settings = ContactSheetSettings {
            columns: 2,
            rows: 2,
            cell_width: 50,
            cell_height: 50,
            background_color: [0, 0, 0],
            show_filename: false,
            show_rating: false,
            padding: 5,
        };

        let (sheet, width, height) = generate_contact_sheet(&photos, &settings);

        // Expected dimensions: 2*50 + 3*5 = 115
        let expected_width = 2 * 50 + 3 * 5;
        let expected_height = 2 * 50 + 3 * 5;

        assert_eq!(width, expected_width);
        assert_eq!(height, expected_height);
        assert_eq!(sheet.len(), (width * height * 3) as usize);
    }

    #[test]
    fn test_contact_sheet_empty() {
        let photos: Vec<(Vec<u8>, u32, u32)> = Vec::new();
        let settings = ContactSheetSettings::default();

        let (sheet, width, height) = generate_contact_sheet(&photos, &settings);

        // Should return minimal valid image
        assert_eq!(width, 1);
        assert_eq!(height, 1);
        assert_eq!(sheet.len(), 3);
        assert_eq!(sheet, vec![40, 40, 40]); // Default background color
    }

    #[test]
    fn test_contact_sheet_more_photos_than_cells() {
        // Create 10 photos but only 2x2 grid (4 cells)
        let mut photos = Vec::new();
        for _i in 0..10 {
            let photo_data = vec![100u8; 50 * 50 * 3];
            photos.push((photo_data, 50, 50));
        }

        let settings = ContactSheetSettings {
            columns: 2,
            rows: 2,
            cell_width: 40,
            cell_height: 40,
            background_color: [50, 50, 50],
            show_filename: false,
            show_rating: false,
            padding: 5,
        };

        let (sheet, width, height) = generate_contact_sheet(&photos, &settings);

        // Should only place 4 photos (2x2)
        assert!(width > 0);
        assert!(height > 0);
        assert_eq!(sheet.len(), (width * height * 3) as usize);
    }

    #[test]
    fn test_resize_to_fit_width_limited() {
        // Create 200x100 image (wide)
        let data = vec![128u8; 200 * 100 * 3];

        let (resized, w, h) = resize_to_fit(&data, 200, 100, 100, 100);

        // Should be width-limited to 100px wide, height scaled proportionally
        assert_eq!(w, 100);
        assert_eq!(h, 50);
        assert_eq!(resized.len(), (w * h * 3) as usize);
    }

    #[test]
    fn test_resize_to_fit_height_limited() {
        // Create 100x200 image (tall)
        let data = vec![128u8; 100 * 200 * 3];

        let (resized, w, h) = resize_to_fit(&data, 100, 200, 100, 100);

        // Should be height-limited to 100px tall, width scaled proportionally
        assert_eq!(w, 50);
        assert_eq!(h, 100);
        assert_eq!(resized.len(), (w * h * 3) as usize);
    }

    #[test]
    fn test_paste_image() {
        // Create 10x10 destination (black)
        let mut dest = vec![0u8; 10 * 10 * 3];

        // Create 3x3 source (white)
        let src = vec![255u8; 3 * 3 * 3];

        // Paste at (2, 2)
        paste_image(&mut dest, 10, &src, 3, 3, 2, 2);

        // Check that pixel at (2,2) is now white
        let idx = ((2 * 10 + 2) * 3) as usize;
        assert_eq!(dest[idx], 255);
        assert_eq!(dest[idx + 1], 255);
        assert_eq!(dest[idx + 2], 255);

        // Check that pixel at (0,0) is still black
        assert_eq!(dest[0], 0);
        assert_eq!(dest[1], 0);
        assert_eq!(dest[2], 0);
    }
}
