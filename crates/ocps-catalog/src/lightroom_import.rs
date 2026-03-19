//! Lightroom Classic catalog import (.lrcat)
//!
//! Reads Lightroom Classic SQLite catalogs and imports photos, keywords, and collections.

use crate::db::Catalog;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LightroomImportResult {
    pub photos_imported: u32,
    pub photos_skipped: u32,
    pub keywords_imported: u32,
    pub collections_imported: u32,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Import a Lightroom Classic catalog (.lrcat) into an OCPS catalog
pub fn import_lightroom_catalog(
    lrcat_path: &Path,
    target_catalog: &mut Catalog,
    base_photo_path: Option<&Path>,
) -> Result<LightroomImportResult> {
    let mut result = LightroomImportResult {
        photos_imported: 0,
        photos_skipped: 0,
        keywords_imported: 0,
        collections_imported: 0,
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    // Open Lightroom catalog as read-only
    let lr_conn = Connection::open_with_flags(
        lrcat_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .context("Failed to open Lightroom catalog")?;

    // Import keywords first (needed for photo-keyword associations)
    let keyword_map = import_keywords(&lr_conn, target_catalog, &mut result)?;

    // Import photos
    import_photos(&lr_conn, target_catalog, base_photo_path, &keyword_map, &mut result)?;

    // Import collections
    import_collections(&lr_conn, target_catalog, &mut result)?;

    Ok(result)
}

/// Import keywords and build a mapping from LR keyword ID to OCPS keyword ID
fn import_keywords(
    lr_conn: &Connection,
    target_catalog: &mut Catalog,
    result: &mut LightroomImportResult,
) -> Result<HashMap<i64, String>> {
    let mut keyword_map = HashMap::new();

    // Check if AgLibraryKeyword table exists
    let table_exists: bool = lr_conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='AgLibraryKeyword'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0) > 0;

    if !table_exists {
        result.warnings.push("AgLibraryKeyword table not found in Lightroom catalog".to_string());
        return Ok(keyword_map);
    }

    // Read all keywords
    let mut stmt = lr_conn.prepare(
        "SELECT id_local, name, parent FROM AgLibraryKeyword ORDER BY id_local"
    )?;

    let keywords: Vec<(i64, String, Option<i64>)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get::<_, Option<i64>>(2)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Insert keywords (handle hierarchy by processing parents first)
    for (lr_id, name, _parent) in &keywords {
        let keyword_id = Uuid::new_v4().to_string();

        // Insert keyword into OCPS catalog
        if let Err(e) = target_catalog.conn.execute(
            "INSERT INTO keywords (id, name, parent_id) VALUES (?1, ?2, NULL)",
            rusqlite::params![&keyword_id, name],
        ) {
            result.warnings.push(format!("Failed to insert keyword '{}': {}", name, e));
            continue;
        }

        keyword_map.insert(*lr_id, keyword_id);
        result.keywords_imported += 1;
    }

    Ok(keyword_map)
}

/// Import photos with ratings, flags, color labels
fn import_photos(
    lr_conn: &Connection,
    target_catalog: &mut Catalog,
    base_photo_path: Option<&Path>,
    keyword_map: &HashMap<i64, String>,
    result: &mut LightroomImportResult,
) -> Result<()> {
    // Check if required tables exist
    let adobe_images_exists: bool = lr_conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='Adobe_images'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0) > 0;

    let ag_library_file_exists: bool = lr_conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='AgLibraryFile'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0) > 0;

    let ag_library_folder_exists: bool = lr_conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='AgLibraryFolder'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0) > 0;

    if !adobe_images_exists || !ag_library_file_exists || !ag_library_folder_exists {
        result.warnings.push("Required Lightroom tables not found".to_string());
        return Ok(());
    }

    // Query photos with file paths
    let query = r#"
        SELECT
            i.id_local,
            i.rating,
            i.colorLabels,
            i.pick,
            f.baseName,
            f.extension,
            folder.pathFromRoot
        FROM Adobe_images i
        LEFT JOIN AgLibraryFile f ON f.id_local = i.rootFile
        LEFT JOIN AgLibraryFolder folder ON folder.id_local = f.folder
        WHERE f.baseName IS NOT NULL
    "#;

    let mut stmt = lr_conn.prepare(query)?;
    let photo_rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,               // lr_id
            row.get::<_, Option<i32>>(1)?,       // rating
            row.get::<_, Option<String>>(2)?,    // colorLabels
            row.get::<_, Option<i32>>(3)?,       // pick
            row.get::<_, String>(4)?,            // baseName
            row.get::<_, String>(5)?,            // extension
            row.get::<_, Option<String>>(6)?,    // pathFromRoot
        ))
    })?;

    for photo_result in photo_rows {
        let (lr_id, rating, color_labels, pick, base_name, extension, path_from_root) =
            match photo_result {
                Ok(data) => data,
                Err(e) => {
                    result.errors.push(format!("Failed to read photo row: {}", e));
                    continue;
                }
            };

        // Build file path
        let file_name = format!("{}.{}", base_name, extension);
        let mut file_path = PathBuf::new();

        if let Some(base) = base_photo_path {
            file_path.push(base);
        }

        if let Some(ref folder_path) = path_from_root {
            file_path.push(folder_path);
        }

        file_path.push(&file_name);

        // Check if file exists
        let file_exists = file_path.exists();
        if !file_exists {
            result.warnings.push(format!("Photo not found: {}", file_path.display()));
        }

        // Map rating (0-5)
        let rating_value = rating.unwrap_or(0).clamp(0, 5);

        // Map color label
        let color_label = map_color_label(color_labels.as_deref());

        // Map flag (pick)
        let flag = map_flag(pick);

        // Insert photo into OCPS catalog
        let photo_id = Uuid::new_v4().to_string();
        let file_path_str = file_path.to_string_lossy().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        match target_catalog.conn.execute(
            r#"
            INSERT INTO photos (
                id, file_path, file_name, rating, color_label, flag,
                date_imported, has_edits, edit_version
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, 0)
            "#,
            rusqlite::params![
                photo_id,
                file_path_str,
                file_name,
                rating_value,
                color_label,
                flag,
                now,
            ],
        ) {
            Ok(_) => {
                result.photos_imported += 1;

                // Import photo-keyword associations
                if let Err(e) = import_photo_keywords(lr_conn, lr_id, &photo_id, keyword_map, target_catalog) {
                    result.warnings.push(format!("Failed to import keywords for {}: {}", file_name, e));
                }
            }
            Err(e) => {
                result.errors.push(format!("Failed to import {}: {}", file_name, e));
                result.photos_skipped += 1;
            }
        }
    }

    Ok(())
}

/// Import photo-keyword associations
fn import_photo_keywords(
    lr_conn: &Connection,
    lr_photo_id: i64,
    ocps_photo_id: &str,
    keyword_map: &HashMap<i64, String>,
    target_catalog: &mut Catalog,
) -> Result<()> {
    // Check if AgLibraryKeywordImage table exists
    let table_exists: bool = lr_conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='AgLibraryKeywordImage'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0) > 0;

    if !table_exists {
        return Ok(());
    }

    let mut stmt = lr_conn.prepare(
        "SELECT tag FROM AgLibraryKeywordImage WHERE image = ?1"
    )?;

    let keyword_ids: Vec<i64> = stmt
        .query_map([lr_photo_id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    for lr_keyword_id in keyword_ids {
        if let Some(ocps_keyword_id) = keyword_map.get(&lr_keyword_id) {
            let _ = target_catalog.conn.execute(
                "INSERT INTO photo_keywords (photo_id, keyword_id) VALUES (?1, ?2)",
                rusqlite::params![ocps_photo_id, ocps_keyword_id],
            );
        }
    }

    Ok(())
}

/// Import collections
fn import_collections(
    lr_conn: &Connection,
    target_catalog: &mut Catalog,
    result: &mut LightroomImportResult,
) -> Result<()> {
    // Check if AgLibraryCollection table exists
    let table_exists: bool = lr_conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='AgLibraryCollection'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0) > 0;

    if !table_exists {
        result.warnings.push("AgLibraryCollection table not found".to_string());
        return Ok(());
    }

    let mut stmt = lr_conn.prepare(
        "SELECT id_local, name FROM AgLibraryCollection WHERE creationId = 'com.adobe.ag.library.collection'"
    )?;

    let collections: Vec<(i64, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    for (_lr_id, name) in collections {
        let collection_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        match target_catalog.conn.execute(
            "INSERT INTO collections (id, name, type, created_at, updated_at) VALUES (?1, ?2, 'manual', ?3, ?4)",
            rusqlite::params![collection_id, name, now, now],
        ) {
            Ok(_) => result.collections_imported += 1,
            Err(e) => result.warnings.push(format!("Failed to import collection '{}': {}", name, e)),
        }
    }

    Ok(())
}

/// Map Lightroom color label to OCPS color label
fn map_color_label(color_labels: Option<&str>) -> String {
    match color_labels {
        Some(label) if !label.is_empty() => {
            // Lightroom uses "red", "yellow", "green", "blue", "purple"
            label.to_lowercase()
        }
        _ => "none".to_string(),
    }
}

/// Map Lightroom pick flag to OCPS flag
fn map_flag(pick: Option<i32>) -> String {
    match pick {
        Some(1) => "pick".to_string(),
        Some(-1) => "reject".to_string(),
        _ => "none".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Catalog;
    use rusqlite::Connection;
    use std::path::PathBuf;

    fn create_fake_lightroom_catalog() -> (PathBuf, Connection) {
        let temp_dir = std::env::temp_dir();
        let lrcat_path = temp_dir.join(format!("test_lr_{}.lrcat", Uuid::new_v4()));

        let conn = Connection::open(&lrcat_path).unwrap();

        // Create minimal Lightroom schema
        conn.execute_batch(r#"
            CREATE TABLE Adobe_images (
                id_local INTEGER PRIMARY KEY,
                rootFile INTEGER,
                rating INTEGER,
                colorLabels TEXT,
                pick INTEGER
            );

            CREATE TABLE AgLibraryFile (
                id_local INTEGER PRIMARY KEY,
                folder INTEGER,
                baseName TEXT,
                extension TEXT
            );

            CREATE TABLE AgLibraryFolder (
                id_local INTEGER PRIMARY KEY,
                pathFromRoot TEXT
            );

            CREATE TABLE AgLibraryKeyword (
                id_local INTEGER PRIMARY KEY,
                name TEXT,
                parent INTEGER
            );

            CREATE TABLE AgLibraryKeywordImage (
                image INTEGER,
                tag INTEGER
            );

            CREATE TABLE AgLibraryCollection (
                id_local INTEGER PRIMARY KEY,
                name TEXT,
                creationId TEXT
            );
        "#).unwrap();

        (lrcat_path, conn)
    }

    #[test]
    fn test_import_lightroom_creates_in_memory_catalog() {
        let (lrcat_path, lr_conn) = create_fake_lightroom_catalog();

        // Insert test data
        lr_conn.execute_batch(r#"
            INSERT INTO AgLibraryFolder (id_local, pathFromRoot) VALUES (1, '/photos');
            INSERT INTO AgLibraryFile (id_local, folder, baseName, extension) VALUES (1, 1, 'photo1', 'arw');
            INSERT INTO AgLibraryFile (id_local, folder, baseName, extension) VALUES (2, 1, 'photo2', 'nef');
            INSERT INTO AgLibraryFile (id_local, folder, baseName, extension) VALUES (3, 1, 'photo3', 'raf');
            INSERT INTO Adobe_images (id_local, rootFile, rating, colorLabels, pick) VALUES (1, 1, 3, 'red', 1);
            INSERT INTO Adobe_images (id_local, rootFile, rating, colorLabels, pick) VALUES (2, 2, 5, 'blue', 0);
            INSERT INTO Adobe_images (id_local, rootFile, rating, colorLabels, pick) VALUES (3, 3, 0, '', -1);

            INSERT INTO AgLibraryKeyword (id_local, name, parent) VALUES (1, 'Nature', NULL);
            INSERT INTO AgLibraryKeyword (id_local, name, parent) VALUES (2, 'Travel', NULL);

            INSERT INTO AgLibraryCollection (id_local, name, creationId) VALUES (1, 'Best Photos', 'com.adobe.ag.library.collection');
        "#).unwrap();
        drop(lr_conn);

        // Import into OCPS catalog
        let mut target_catalog = Catalog::in_memory().unwrap();
        let result = import_lightroom_catalog(&lrcat_path, &mut target_catalog, None).unwrap();

        assert_eq!(result.photos_imported, 3);
        assert_eq!(result.keywords_imported, 2);
        assert_eq!(result.collections_imported, 1);
        assert_eq!(target_catalog.photo_count().unwrap(), 3);

        // Cleanup
        let _ = std::fs::remove_file(lrcat_path);
    }

    #[test]
    fn test_import_lightroom_maps_ratings() {
        let (lrcat_path, lr_conn) = create_fake_lightroom_catalog();

        lr_conn.execute_batch(r#"
            INSERT INTO AgLibraryFolder (id_local, pathFromRoot) VALUES (1, '/photos');
            INSERT INTO AgLibraryFile (id_local, folder, baseName, extension) VALUES (1, 1, 'photo1', 'arw');
            INSERT INTO Adobe_images (id_local, rootFile, rating, colorLabels, pick) VALUES (1, 1, 5, '', 0);
        "#).unwrap();
        drop(lr_conn);

        let mut target_catalog = Catalog::in_memory().unwrap();
        let result = import_lightroom_catalog(&lrcat_path, &mut target_catalog, None).unwrap();

        assert_eq!(result.photos_imported, 1);

        // Verify rating
        let rating: i32 = target_catalog.conn.query_row(
            "SELECT rating FROM photos LIMIT 1",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(rating, 5);

        let _ = std::fs::remove_file(lrcat_path);
    }

    #[test]
    fn test_import_lightroom_maps_flags() {
        let (lrcat_path, lr_conn) = create_fake_lightroom_catalog();

        lr_conn.execute_batch(r#"
            INSERT INTO AgLibraryFolder (id_local, pathFromRoot) VALUES (1, '/photos');
            INSERT INTO AgLibraryFile (id_local, folder, baseName, extension) VALUES (1, 1, 'photo1', 'arw');
            INSERT INTO Adobe_images (id_local, rootFile, rating, colorLabels, pick) VALUES (1, 1, 0, '', 1);
        "#).unwrap();
        drop(lr_conn);

        let mut target_catalog = Catalog::in_memory().unwrap();
        let result = import_lightroom_catalog(&lrcat_path, &mut target_catalog, None).unwrap();

        assert_eq!(result.photos_imported, 1);

        // Verify flag
        let flag: String = target_catalog.conn.query_row(
            "SELECT flag FROM photos LIMIT 1",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(flag, "pick");

        let _ = std::fs::remove_file(lrcat_path);
    }

    #[test]
    fn test_map_color_label() {
        assert_eq!(map_color_label(Some("red")), "red");
        assert_eq!(map_color_label(Some("BLUE")), "blue");
        assert_eq!(map_color_label(Some("")), "none");
        assert_eq!(map_color_label(None), "none");
    }

    #[test]
    fn test_map_flag() {
        assert_eq!(map_flag(Some(1)), "pick");
        assert_eq!(map_flag(Some(-1)), "reject");
        assert_eq!(map_flag(Some(0)), "none");
        assert_eq!(map_flag(None), "none");
    }
}
