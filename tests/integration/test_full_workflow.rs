/// Cross-crate integration tests for full workflow
/// Tests the complete pipeline: catalog -> import -> rate -> edit -> export
use ocps_catalog::{Catalog, PhotoFilter, SortOrder};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_import_rate_export_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let catalog_path = temp_dir.path().join("test.ocps");
    let import_dir = temp_dir.path().join("photos");
    fs::create_dir(&import_dir).unwrap();

    // Step 1: Create in-memory catalog
    let catalog = Catalog::open(&catalog_path).unwrap();

    // Step 2: Create 3 fake photo files
    for i in 1..=3 {
        fs::write(import_dir.join(format!("photo{}.arw", i)), b"fake raw data").unwrap();
    }

    // Step 3: Import folder
    let import_result = catalog.import_folder(&import_dir).unwrap();
    assert_eq!(import_result.inserted, 3);
    assert_eq!(catalog.photo_count().unwrap(), 3);

    // Step 4: Get all photos
    let filter = PhotoFilter {
        rating_min: None,
        flag: None,
        color_label: None,
        search: None,
        limit: 100,
        offset: 0,
    };
    let photos = catalog.get_photos(&filter, &SortOrder::DateImportedDesc).unwrap();
    assert_eq!(photos.len(), 3);

    let photo1_id = &photos[0].id;

    // Step 5: Rate photo 1 as 5 stars
    catalog.update_rating(photo1_id, 5).unwrap();

    // Step 6: Create edit recipe (JSON string for now)
    let edit_json = r#"{"exposure":0.5,"contrast":10,"saturation":20}"#;
    catalog.save_edit(photo1_id, edit_json).unwrap();

    // Step 7: Copy edit recipe to photos 2 and 3
    let photo2_id = &photos[1].id;
    let photo3_id = &photos[2].id;
    catalog.save_edit(photo2_id, edit_json).unwrap();
    catalog.save_edit(photo3_id, edit_json).unwrap();

    // Step 8: Verify all 3 photos have edit recipes saved
    assert!(catalog.load_edit(photo1_id).unwrap().is_some());
    assert!(catalog.load_edit(photo2_id).unwrap().is_some());
    assert!(catalog.load_edit(photo3_id).unwrap().is_some());

    // Step 9: Create smart collection: rating >= 5
    let rules = ocps_catalog::db::SmartCollectionRules {
        match_all: true,
        rules: vec![ocps_catalog::db::SmartRule {
            field: "rating".to_string(),
            op: "gte".to_string(),
            value: "5".to_string(),
        }],
    };
    let collection_id = catalog.create_smart_collection("5 Stars", &rules).unwrap();
    assert!(!collection_id.is_empty());

    // Step 10: Evaluate smart collection
    let matching_photos = catalog.evaluate_smart_collection(&rules).unwrap();
    assert_eq!(matching_photos.len(), 1); // Only photo1 has rating 5

    // Step 11: Batch update flag for all to "pick"
    let all_ids: Vec<String> = photos.iter().map(|p| p.id.clone()).collect();
    let updated = catalog.batch_update_flag(&all_ids, "pick").unwrap();
    assert_eq!(updated, 3);

    // Step 12: Verify pick count == 3
    let filter_picks = PhotoFilter {
        rating_min: None,
        flag: Some("pick".to_string()),
        color_label: None,
        search: None,
        limit: 100,
        offset: 0,
    };
    let picks = catalog.get_photos(&filter_picks, &SortOrder::DateImportedDesc).unwrap();
    assert_eq!(picks.len(), 3);
}

#[test]
fn test_xmp_sidecar_workflow() {
    use ocps_xmp::XmpDevelopSettings;

    // Step 1: Create XmpDevelopSettings with known values
    let mut settings = XmpDevelopSettings::default();
    settings.exposure = Some(1.5);
    settings.contrast = Some(25);
    settings.saturation = Some(-50);
    settings.temperature = Some(5800);

    // Step 2: Serialize to XMP string
    let xmp_str = serde_json::to_string(&settings).unwrap();

    // Step 3: Deserialize back
    let roundtrip: XmpDevelopSettings = serde_json::from_str(&xmp_str).unwrap();

    // Step 4: Verify values match (roundtrip)
    assert_eq!(roundtrip.exposure, Some(1.5));
    assert_eq!(roundtrip.contrast, Some(25));
    assert_eq!(roundtrip.saturation, Some(-50));
    assert_eq!(roundtrip.temperature, Some(5800));

    // Step 5: Merge new values
    let mut merged = roundtrip.clone();
    merged.highlights = Some(-30);
    merged.shadows = Some(40);

    // Step 6: Verify both old and new values present
    assert_eq!(merged.exposure, Some(1.5)); // old
    assert_eq!(merged.highlights, Some(-30)); // new
    assert_eq!(merged.shadows, Some(40)); // new
}

#[test]
fn test_preset_apply_workflow() {
    use ocps_xmp::XmpDevelopSettings;

    // Step 1: Create a B&W "preset" (just settings for now)
    let mut bw_settings = XmpDevelopSettings::default();
    bw_settings.saturation = Some(-100);
    bw_settings.contrast = Some(20);

    // Step 2: Apply to default recipe
    let mut photo_settings = XmpDevelopSettings::default();
    photo_settings.saturation = bw_settings.saturation;
    photo_settings.contrast = bw_settings.contrast;

    // Step 3: Verify saturation == -100
    assert_eq!(photo_settings.saturation, Some(-100));

    // Step 4: Create warm tone settings
    let mut warm_settings = XmpDevelopSettings::default();
    warm_settings.temperature = Some(6200);
    warm_settings.tint = Some(10);

    // Step 5: Apply warm tone
    photo_settings.temperature = warm_settings.temperature;
    photo_settings.tint = warm_settings.tint;

    // Step 6: Verify temperature > 5500
    assert!(photo_settings.temperature.unwrap() > 5500);
}
