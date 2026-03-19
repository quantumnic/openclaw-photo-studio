//! SQLite catalog database implementation

use crate::models::{ImportResult, PhotoFilter, PhotoRecord, SortOrder};
use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection, Row};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;

pub struct Catalog {
    conn: Connection,
}

impl Catalog {
    /// Open or create a catalog at the given path
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let catalog = Self { conn };
        catalog.initialize()?;
        Ok(catalog)
    }

    /// Create an in-memory catalog (for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let catalog = Self { conn };
        catalog.initialize()?;
        Ok(catalog)
    }

    fn initialize(&self) -> Result<()> {
        self.conn
            .execute_batch(crate::schema::CREATE_MIGRATIONS)?;
        self.conn.execute_batch(crate::schema::CREATE_PHOTOS)?;
        self.conn
            .execute_batch(crate::schema::CREATE_COLLECTIONS)?;
        self.conn.execute_batch(crate::schema::CREATE_KEYWORDS)?;
        self.conn
            .execute_batch(crate::schema::CREATE_PHOTO_KEYWORDS)?;
        self.conn.execute_batch(crate::schema::CREATE_EDITS)?;
        for index in crate::schema::INDEXES {
            self.conn.execute_batch(index)?;
        }
        Ok(())
    }

    /// Import photos from a folder
    pub fn import_folder(&self, folder_path: &Path) -> Result<ImportResult> {
        let mut result = ImportResult {
            total: 0,
            inserted: 0,
            skipped: 0,
            errors: Vec::new(),
        };

        let supported_extensions = [
            "arw", "nef", "raf", "dng", "cr2", "cr3", "orf", "rw2", "jpg", "jpeg", "tiff", "tif",
            "png",
        ];

        for entry in WalkDir::new(folder_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if supported_extensions.contains(&ext_str.as_str()) {
                    result.total += 1;

                    match self.import_single_photo(path) {
                        Ok(true) => result.inserted += 1,
                        Ok(false) => result.skipped += 1,
                        Err(e) => {
                            result.errors.push(format!("{}: {}", path.display(), e));
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    fn import_single_photo(&self, path: &Path) -> Result<bool> {
        // Check if already imported
        let path_str = path.to_string_lossy().to_string();
        let exists: bool = self
            .conn
            .query_row(
                "SELECT 1 FROM photos WHERE file_path = ? LIMIT 1",
                params![&path_str],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if exists {
            return Ok(false); // skipped
        }

        // Get file metadata
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();

        // Calculate file hash
        let file_hash = self.calculate_file_hash(path)?;

        // Generate UUID
        let id = Uuid::new_v4().to_string();

        // Get file name
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Determine MIME type
        let mime_type = self.guess_mime_type(path);

        // Current timestamp
        let now = Utc::now().to_rfc3339();

        // Insert into database
        self.conn.execute(
            r#"
            INSERT INTO photos (
                id, file_path, file_name, file_size, file_hash, mime_type,
                date_imported, rating, color_label, flag
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, 'none', 'none')
            "#,
            params![&id, &path_str, &file_name, &file_size, &file_hash, &mime_type, &now],
        )?;

        Ok(true) // inserted
    }

    fn calculate_file_hash(&self, path: &Path) -> Result<String> {
        let data = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    fn guess_mime_type(&self, path: &Path) -> String {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            match ext_str.as_str() {
                "arw" => "image/x-sony-arw",
                "nef" => "image/x-nikon-nef",
                "raf" => "image/x-fuji-raf",
                "dng" => "image/x-adobe-dng",
                "cr2" => "image/x-canon-cr2",
                "cr3" => "image/x-canon-cr3",
                "orf" => "image/x-olympus-orf",
                "rw2" => "image/x-panasonic-rw2",
                "jpg" | "jpeg" => "image/jpeg",
                "tiff" | "tif" => "image/tiff",
                "png" => "image/png",
                _ => "application/octet-stream",
            }
            .to_string()
        } else {
            "application/octet-stream".to_string()
        }
    }

    /// Get photos with filtering and pagination
    pub fn get_photos(&self, filter: &PhotoFilter, sort: &SortOrder) -> Result<Vec<PhotoRecord>> {
        let mut sql = String::from("SELECT * FROM photos WHERE 1=1");
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(rating_min) = filter.rating_min {
            sql.push_str(" AND rating >= ?");
            params.push(Box::new(rating_min));
        }

        if let Some(ref flag) = filter.flag {
            sql.push_str(" AND flag = ?");
            params.push(Box::new(flag.clone()));
        }

        if let Some(ref color_label) = filter.color_label {
            sql.push_str(" AND color_label = ?");
            params.push(Box::new(color_label.clone()));
        }

        if let Some(ref search) = filter.search {
            sql.push_str(" AND (file_name LIKE ? OR camera_make LIKE ? OR camera_model LIKE ?)");
            let search_pattern = format!("%{}%", search);
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern));
        }

        sql.push_str(&format!(" ORDER BY {}", sort.to_sql()));
        sql.push_str(&format!(" LIMIT {} OFFSET {}", filter.limit, filter.offset));

        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|b| &**b as &dyn rusqlite::ToSql).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let photos = stmt
            .query_map(&param_refs[..], |row| Self::row_to_photo(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(photos)
    }

    fn row_to_photo(row: &Row) -> rusqlite::Result<PhotoRecord> {
        Ok(PhotoRecord {
            id: row.get(0)?,
            file_path: row.get(1)?,
            file_name: row.get(2)?,
            file_size: row.get::<_, i64>(3)? as u64,
            width: row.get(6).ok(),
            height: row.get(7).ok(),
            date_taken: row.get(9).ok(),
            date_imported: row.get(10)?,
            camera_make: row.get(11).ok(),
            camera_model: row.get(12).ok(),
            rating: row.get::<_, i64>(20)? as u8,
            color_label: row.get(21)?,
            flag: row.get(22)?,
            has_edits: row.get::<_, i64>(23)? != 0,
        })
    }

    /// Get a single photo by ID
    pub fn get_photo(&self, id: &str) -> Result<Option<PhotoRecord>> {
        let mut stmt = self.conn.prepare("SELECT * FROM photos WHERE id = ?")?;
        let result = stmt.query_row(params![id], |row| Self::row_to_photo(row));

        match result {
            Ok(photo) => Ok(Some(photo)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Update photo rating
    pub fn update_rating(&self, id: &str, rating: u8) -> Result<()> {
        if rating > 5 {
            anyhow::bail!("Rating must be 0-5");
        }

        self.conn.execute(
            "UPDATE photos SET rating = ? WHERE id = ?",
            params![rating, id],
        )?;

        Ok(())
    }

    /// Update photo flag
    pub fn update_flag(&self, id: &str, flag: &str) -> Result<()> {
        let valid_flags = ["none", "pick", "reject"];
        if !valid_flags.contains(&flag) {
            anyhow::bail!("Invalid flag value");
        }

        self.conn
            .execute("UPDATE photos SET flag = ? WHERE id = ?", params![flag, id])?;

        Ok(())
    }

    /// Update color label
    pub fn update_color_label(&self, id: &str, label: &str) -> Result<()> {
        let valid_labels = ["none", "red", "yellow", "green", "blue", "purple"];
        if !valid_labels.contains(&label) {
            anyhow::bail!("Invalid color label");
        }

        self.conn.execute(
            "UPDATE photos SET color_label = ? WHERE id = ?",
            params![label, id],
        )?;

        Ok(())
    }

    /// Get total photo count
    pub fn photo_count(&self) -> Result<u64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM photos", [], |row| row.get(0))?;
        Ok(count as u64)
    }

    /// Search photos by query (simple text search)
    pub fn search(&self, query: &str) -> Result<Vec<PhotoRecord>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT * FROM photos
             WHERE file_name LIKE ?
                OR camera_make LIKE ?
                OR camera_model LIKE ?
             ORDER BY date_taken DESC
             LIMIT 500",
        )?;

        let photos = stmt
            .query_map(
                params![&search_pattern, &search_pattern, &search_pattern],
                |row| Self::row_to_photo(row),
            )?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(photos)
    }

    /// Save edit settings for a photo
    pub fn save_edit(&self, photo_id: &str, edit_json: &str) -> Result<()> {
        let edit_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Set all previous edits as not current
        self.conn.execute(
            "UPDATE edits SET is_current = 0 WHERE photo_id = ?",
            params![photo_id],
        )?;

        // Get next version number
        let version: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) + 1 FROM edits WHERE photo_id = ?",
                params![photo_id],
                |row| row.get(0),
            )
            .unwrap_or(1);

        // Insert new edit
        self.conn.execute(
            "INSERT INTO edits (id, photo_id, version, edit_data, created_at, is_current)
             VALUES (?, ?, ?, ?, ?, 1)",
            params![&edit_id, photo_id, version, edit_json, &now],
        )?;

        // Update photo has_edits flag
        self.conn.execute(
            "UPDATE photos SET has_edits = 1, edit_version = ? WHERE id = ?",
            params![version, photo_id],
        )?;

        Ok(())
    }

    /// Load current edit settings for a photo
    pub fn load_edit(&self, photo_id: &str) -> Result<Option<String>> {
        let result: rusqlite::Result<String> = self.conn.query_row(
            "SELECT edit_data FROM edits WHERE photo_id = ? AND is_current = 1",
            params![photo_id],
            |row| row.get(0),
        );

        match result {
            Ok(data) => Ok(Some(data)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_catalog() {
        let catalog = Catalog::in_memory().unwrap();
        assert_eq!(catalog.photo_count().unwrap(), 0);
    }

    #[test]
    fn test_update_rating() {
        let catalog = Catalog::in_memory().unwrap();

        // Insert a test photo
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        catalog
            .conn
            .execute(
                "INSERT INTO photos (id, file_path, file_name, file_size, date_imported)
                 VALUES (?, ?, ?, ?, ?)",
                params![&id, "/test.jpg", "test.jpg", 1000, &now],
            )
            .unwrap();

        // Update rating
        catalog.update_rating(&id, 5).unwrap();

        // Verify
        let photo = catalog.get_photo(&id).unwrap().unwrap();
        assert_eq!(photo.rating, 5);
    }

    #[test]
    fn test_update_flag() {
        let catalog = Catalog::in_memory().unwrap();

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        catalog
            .conn
            .execute(
                "INSERT INTO photos (id, file_path, file_name, file_size, date_imported)
                 VALUES (?, ?, ?, ?, ?)",
                params![&id, "/test.jpg", "test.jpg", 1000, &now],
            )
            .unwrap();

        catalog.update_flag(&id, "pick").unwrap();

        let photo = catalog.get_photo(&id).unwrap().unwrap();
        assert_eq!(photo.flag, "pick");
    }

    #[test]
    fn test_save_load_edit() {
        let catalog = Catalog::in_memory().unwrap();

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        catalog
            .conn
            .execute(
                "INSERT INTO photos (id, file_path, file_name, file_size, date_imported)
                 VALUES (?, ?, ?, ?, ?)",
                params![&id, "/test.jpg", "test.jpg", 1000, &now],
            )
            .unwrap();

        let edit_data = r#"{"exposure": 1.5, "contrast": 20}"#;
        catalog.save_edit(&id, edit_data).unwrap();

        let loaded = catalog.load_edit(&id).unwrap();
        assert_eq!(loaded, Some(edit_data.to_string()));
    }
}
