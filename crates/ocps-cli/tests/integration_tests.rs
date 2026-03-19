use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn get_cli_bin() -> PathBuf {
    // Path to the compiled binary
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../target/debug/ocps");
    path
}

#[test]
fn test_cli_stats_empty_catalog() {
    let temp_dir = TempDir::new().unwrap();
    let catalog_path = temp_dir.path().join("test.ocps");

    // Create empty catalog
    let catalog = ocps_catalog::Catalog::open(&catalog_path).unwrap();
    drop(catalog);

    // Run CLI stats command
    let output = Command::new(get_cli_bin())
        .arg("stats")
        .arg("--catalog")
        .arg(&catalog_path)
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0"), "Expected '0 photos' in output, got: {}", stdout);
    assert!(stdout.contains("Total photos"), "Expected 'Total photos' in output");
}

#[test]
fn test_cli_import_folder() {
    let temp_dir = TempDir::new().unwrap();
    let catalog_path = temp_dir.path().join("test.ocps");
    let import_dir = temp_dir.path().join("import");
    fs::create_dir(&import_dir).unwrap();

    // Create 2 fake ARW files (just empty files for testing catalog insertion)
    fs::write(import_dir.join("test1.arw"), b"").unwrap();
    fs::write(import_dir.join("test2.arw"), b"").unwrap();

    // Run CLI import command
    let output = Command::new(get_cli_bin())
        .arg("import")
        .arg(&import_dir)
        .arg("--catalog")
        .arg(&catalog_path)
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Inserted: 2") || stdout.contains("Total:    2"),
            "Expected 2 photos imported, got: {}", stdout);

    // Verify catalog has 2 photos
    let catalog = ocps_catalog::Catalog::open(&catalog_path).unwrap();
    assert_eq!(catalog.photo_count().unwrap(), 2);
}

#[test]
fn test_cli_list() {
    let temp_dir = TempDir::new().unwrap();
    let catalog_path = temp_dir.path().join("test.ocps");
    let import_dir = temp_dir.path().join("import");
    fs::create_dir(&import_dir).unwrap();

    // Create test files
    fs::write(import_dir.join("photo1.jpg"), b"").unwrap();
    fs::write(import_dir.join("photo2.jpg"), b"").unwrap();

    // Import via catalog directly for control
    let catalog = ocps_catalog::Catalog::open(&catalog_path).unwrap();
    catalog.import_folder(&import_dir).unwrap();
    drop(catalog);

    // Run CLI list command
    let output = Command::new(get_cli_bin())
        .arg("list")
        .arg("--catalog")
        .arg(&catalog_path)
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("photo1.jpg") || stdout.contains("photo2.jpg"),
            "Expected filenames in output, got: {}", stdout);
    assert!(stdout.contains("2 photos found") || stdout.contains("Filename"),
            "Expected photo listing header");
}
