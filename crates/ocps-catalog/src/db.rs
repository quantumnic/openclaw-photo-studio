//! SQLite catalog database implementation

use crate::models::{ImportResult, PhotoFilter, PhotoRecord, SortOrder};
use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection, Row};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use uuid::Uuid;
use walkdir::WalkDir;

pub struct Catalog {
    pub(crate) conn: Connection,
    pub(crate) path: Option<std::path::PathBuf>,
}

impl Catalog {
    /// Open or create a catalog at the given path
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let catalog = Self {
            conn,
            path: Some(path.to_path_buf()),
        };
        catalog.initialize()?;
        Ok(catalog)
    }

    /// Create an in-memory catalog (for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let catalog = Self {
            conn,
            path: None,
        };
        catalog.initialize()?;
        Ok(catalog)
    }

    /// Get the database path (if not in-memory)
    pub fn database_path(&self) -> &Path {
        self.path.as_deref().unwrap_or(Path::new(":memory:"))
    }

    fn initialize(&self) -> Result<()> {
        self.conn
            .execute_batch(crate::schema::CREATE_MIGRATIONS)?;
        self.conn.execute_batch(crate::schema::CREATE_PHOTOS)?;
        self.conn
            .execute_batch(crate::schema::CREATE_COLLECTIONS)?;
        self.conn
            .execute_batch(crate::schema::CREATE_PHOTO_COLLECTIONS)?;
        self.conn.execute_batch(crate::schema::CREATE_KEYWORDS)?;
        self.conn
            .execute_batch(crate::schema::CREATE_PHOTO_KEYWORDS)?;
        self.conn.execute_batch(crate::schema::CREATE_EDITS)?;
        self.conn.execute_batch(crate::schema::CREATE_FTS)?;
        self.conn.execute_batch(crate::schema::CREATE_FTS_TRIGGERS)?;
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
            .query_map(&param_refs[..], Self::row_to_photo)?
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
            stack_id: row.get(26).ok(),
            stack_position: row.get(27).ok(),
            virtual_copy_of: row.get(28).ok(),
        })
    }

    /// Get a single photo by ID
    pub fn get_photo(&self, id: &str) -> Result<Option<PhotoRecord>> {
        let mut stmt = self.conn.prepare("SELECT * FROM photos WHERE id = ?")?;
        let result = stmt.query_row(params![id], Self::row_to_photo);

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

    /// Search photos by query using FTS5 full-text search
    pub fn search(&self, query: &str, limit: u32) -> Result<Vec<PhotoRecord>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Add * for prefix search (e.g., "can" matches "canon")
        let fts_query = format!("{}*", query.trim());

        let mut stmt = self.conn.prepare(
            "SELECT p.* FROM photos p
             JOIN photos_fts f ON p.id = f.id
             WHERE photos_fts MATCH ?
             ORDER BY rank
             LIMIT ?",
        )?;

        let photos = stmt
            .query_map(params![&fts_query, limit], Self::row_to_photo)?
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

    /// Create a smart collection with rules
    pub fn create_smart_collection(&self, name: &str, rules: &SmartCollectionRules) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let rules_json = serde_json::to_string(rules)?;

        self.conn.execute(
            "INSERT INTO collections (id, name, type, smart_rules, created_at, updated_at) VALUES (?, ?, 'smart', ?, datetime('now'), datetime('now'))",
            params![&id, name, &rules_json],
        )?;

        Ok(id)
    }

    /// Evaluate a smart collection and return matching photo IDs
    pub fn evaluate_smart_collection(&self, rules: &SmartCollectionRules) -> Result<Vec<String>> {
        let sql = self.build_smart_collection_query(rules);
        let mut stmt = self.conn.prepare(&sql)?;
        let photo_ids = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?;
        Ok(photo_ids)
    }

    fn build_smart_collection_query(&self, rules: &SmartCollectionRules) -> String {
        let mut conditions = Vec::new();

        for rule in &rules.rules {
            let condition = match rule.op.as_str() {
                "gte" => {
                    format!("{} >= {}", rule.field, rule.value)
                }
                "lte" => {
                    format!("{} <= {}", rule.field, rule.value)
                }
                "eq" => {
                    // String equality needs quotes
                    if rule.field == "flag" || rule.field.ends_with("_make") || rule.field.ends_with("_model") {
                        format!("{} = '{}'", rule.field, rule.value)
                    } else {
                        format!("{} = {}", rule.field, rule.value)
                    }
                }
                "contains" => {
                    format!("{} LIKE '%{}%'", rule.field, rule.value)
                }
                "in_last_days" => {
                    let days: i32 = rule.value.parse().unwrap_or(7);
                    format!("{} >= datetime('now', '-{} days')", rule.field, days)
                }
                _ => "1=1".to_string(), // Default to true for unknown operators
            };
            conditions.push(condition);
        }

        let connector = if rules.match_all { " AND " } else { " OR " };
        let where_clause = if conditions.is_empty() {
            "1=1".to_string()
        } else {
            conditions.join(connector)
        };

        format!("SELECT id FROM photos WHERE {}", where_clause)
    }

    // ===== STACKING OPERATIONS =====

    /// Create a stack from the given photo IDs
    pub fn create_stack(&self, photo_ids: &[String]) -> Result<String> {
        if photo_ids.is_empty() {
            anyhow::bail!("Cannot create stack with no photos");
        }

        let stack_id = Uuid::new_v4().to_string();

        for (position, photo_id) in photo_ids.iter().enumerate() {
            self.conn.execute(
                "UPDATE photos SET stack_id = ?, stack_position = ? WHERE id = ?",
                params![&stack_id, position as i32, photo_id],
            )?;
        }

        Ok(stack_id)
    }

    /// Unstack all photos in the given stack
    pub fn unstack(&self, stack_id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE photos SET stack_id = NULL, stack_position = NULL WHERE stack_id = ?",
            params![stack_id],
        )?;
        Ok(())
    }

    /// Get all photos in a stack, ordered by stack_position
    pub fn get_stack(&self, stack_id: &str) -> Result<Vec<PhotoRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM photos WHERE stack_id = ? ORDER BY stack_position ASC"
        )?;
        let photos = stmt
            .query_map(params![stack_id], Self::row_to_photo)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(photos)
    }

    /// Move a photo to the top of its stack
    pub fn move_to_top_of_stack(&self, photo_id: &str) -> Result<()> {
        // Get the stack_id for this photo
        let stack_id: Option<String> = self.conn.query_row(
            "SELECT stack_id FROM photos WHERE id = ?",
            params![photo_id],
            |row| row.get(0),
        ).ok();

        if let Some(sid) = stack_id {
            // Set this photo's position to -1
            self.conn.execute(
                "UPDATE photos SET stack_position = -1 WHERE id = ?",
                params![photo_id],
            )?;

            // Get all other photos in the stack
            let mut stmt = self.conn.prepare(
                "SELECT id FROM photos WHERE stack_id = ? AND id != ? ORDER BY stack_position ASC"
            )?;
            let other_ids: Vec<String> = stmt
                .query_map(params![&sid, photo_id], |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?;

            // Renumber them starting from 1
            for (i, id) in other_ids.iter().enumerate() {
                self.conn.execute(
                    "UPDATE photos SET stack_position = ? WHERE id = ?",
                    params![(i + 1) as i32, id],
                )?;
            }

            // Now set the target photo to position 0
            self.conn.execute(
                "UPDATE photos SET stack_position = 0 WHERE id = ?",
                params![photo_id],
            )?;
        }

        Ok(())
    }

    // ===== VIRTUAL COPIES =====

    /// Create a virtual copy of a photo
    pub fn create_virtual_copy(&self, original_photo_id: &str) -> Result<String> {
        // Get the original photo (verify it exists)
        let _original = self.get_photo(original_photo_id)?
            .ok_or_else(|| anyhow::anyhow!("Original photo not found"))?;

        // Generate new ID
        let copy_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Create a copy with same metadata but new ID
        self.conn.execute(
            r#"
            INSERT INTO photos (
                id, file_path, file_name, file_size, file_hash, mime_type,
                width, height, orientation,
                date_taken, date_imported,
                camera_make, camera_model, lens,
                focal_length, aperture, shutter_speed, iso,
                gps_lat, gps_lon,
                rating, color_label, flag,
                virtual_copy_of
            )
            SELECT
                ?, file_path, file_name, file_size, file_hash, mime_type,
                width, height, orientation,
                date_taken, ?,
                camera_make, camera_model, lens,
                focal_length, aperture, shutter_speed, iso,
                gps_lat, gps_lon,
                rating, color_label, flag,
                ?
            FROM photos WHERE id = ?
            "#,
            params![&copy_id, &now, original_photo_id, original_photo_id],
        )?;

        // Copy current edit if it exists
        if let Some(edit_data) = self.load_edit(original_photo_id)? {
            self.save_edit(&copy_id, &edit_data)?;
        }

        Ok(copy_id)
    }

    /// Get all virtual copies of a photo
    pub fn get_virtual_copies(&self, original_photo_id: &str) -> Result<Vec<PhotoRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM photos WHERE virtual_copy_of = ? ORDER BY date_imported ASC"
        )?;
        let copies = stmt
            .query_map(params![original_photo_id], Self::row_to_photo)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(copies)
    }

    // ===== BATCH OPERATIONS =====

    /// Batch update rating for multiple photos
    pub fn batch_update_rating(&self, photo_ids: &[String], rating: u8) -> Result<u32> {
        if rating > 5 {
            anyhow::bail!("Rating must be 0-5");
        }
        if photo_ids.is_empty() {
            return Ok(0);
        }

        let placeholders = photo_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("UPDATE photos SET rating = ? WHERE id IN ({})", placeholders);

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(rating)];
        for id in photo_ids {
            params.push(Box::new(id.clone()));
        }
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|b| &**b as &dyn rusqlite::ToSql).collect();

        let count = self.conn.execute(&sql, &param_refs[..])?;
        Ok(count as u32)
    }

    /// Batch update flag for multiple photos
    pub fn batch_update_flag(&self, photo_ids: &[String], flag: &str) -> Result<u32> {
        let valid_flags = ["none", "pick", "reject"];
        if !valid_flags.contains(&flag) {
            anyhow::bail!("Invalid flag value");
        }
        if photo_ids.is_empty() {
            return Ok(0);
        }

        let placeholders = photo_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("UPDATE photos SET flag = ? WHERE id IN ({})", placeholders);

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(flag.to_string())];
        for id in photo_ids {
            params.push(Box::new(id.clone()));
        }
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|b| &**b as &dyn rusqlite::ToSql).collect();

        let count = self.conn.execute(&sql, &param_refs[..])?;
        Ok(count as u32)
    }

    /// Batch update color label for multiple photos
    pub fn batch_update_color_label(&self, photo_ids: &[String], label: &str) -> Result<u32> {
        let valid_labels = ["none", "red", "yellow", "green", "blue", "purple"];
        if !valid_labels.contains(&label) {
            anyhow::bail!("Invalid color label");
        }
        if photo_ids.is_empty() {
            return Ok(0);
        }

        let placeholders = photo_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("UPDATE photos SET color_label = ? WHERE id IN ({})", placeholders);

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(label.to_string())];
        for id in photo_ids {
            params.push(Box::new(id.clone()));
        }
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|b| &**b as &dyn rusqlite::ToSql).collect();

        let count = self.conn.execute(&sql, &param_refs[..])?;
        Ok(count as u32)
    }

    /// Batch delete photos (from catalog only, not from disk)
    pub fn batch_delete(&self, photo_ids: &[String]) -> Result<u32> {
        if photo_ids.is_empty() {
            return Ok(0);
        }

        let placeholders = photo_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("DELETE FROM photos WHERE id IN ({})", placeholders);

        let params: Vec<Box<dyn rusqlite::ToSql>> = photo_ids
            .iter()
            .map(|id| Box::new(id.clone()) as Box<dyn rusqlite::ToSql>)
            .collect();
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|b| &**b as &dyn rusqlite::ToSql).collect();

        let count = self.conn.execute(&sql, &param_refs[..])?;
        Ok(count as u32)
    }

    /// Batch add keywords to multiple photos
    pub fn batch_add_keywords(&self, photo_ids: &[String], keyword_ids: &[String]) -> Result<u32> {
        if photo_ids.is_empty() || keyword_ids.is_empty() {
            return Ok(0);
        }

        let mut count = 0;
        for photo_id in photo_ids {
            for keyword_id in keyword_ids {
                // Use INSERT OR IGNORE to avoid duplicates
                self.conn.execute(
                    "INSERT OR IGNORE INTO photo_keywords (photo_id, keyword_id) VALUES (?, ?)",
                    params![photo_id, keyword_id],
                )?;
                count += 1;
            }
        }

        Ok(count)
    }

    // ===== IPTC METADATA OPERATIONS =====

    /// Update IPTC field for a photo
    pub fn update_photo_iptc(&self, photo_id: &str, field: &str, value: &str) -> Result<()> {
        let valid_fields = ["copyright", "creator", "city", "country"];
        if !valid_fields.contains(&field) {
            anyhow::bail!("Invalid IPTC field: {}", field);
        }

        let sql = format!("UPDATE photos SET {} = ? WHERE id = ?", field);
        self.conn.execute(&sql, params![value, photo_id])?;
        Ok(())
    }

    /// Get or create a keyword by name
    pub fn get_or_create_keyword(&self, name: &str) -> Result<String> {
        // Try to find existing keyword
        let existing: Option<String> = self.conn.query_row(
            "SELECT id FROM keywords WHERE name = ?",
            params![name],
            |row| row.get(0),
        ).ok();

        if let Some(id) = existing {
            return Ok(id);
        }

        // Create new keyword
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO keywords (id, name) VALUES (?, ?)",
            params![&id, name],
        )?;
        Ok(id)
    }

    // ===== KEYWORD OPERATIONS =====

    /// Get all keywords with usage count
    pub fn get_all_keywords_with_count(&self) -> Result<Vec<(String, String, u32)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT k.id, k.name, COUNT(pk.photo_id) as count
            FROM keywords k
            LEFT JOIN photo_keywords pk ON k.id = pk.keyword_id
            GROUP BY k.id, k.name
            ORDER BY k.name ASC
            "#
        )?;

        let keywords = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)? as u32,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(keywords)
    }

    /// Get reference to the underlying connection (for tests and migrations)
    #[cfg(test)]
    pub fn conn_ref(&self) -> &Connection {
        &self.conn
    }

    // ===== CRASH RECOVERY & MAINTENANCE =====

    /// Verify database integrity using SQLite's PRAGMA integrity_check
    ///
    /// # Returns
    /// * `Ok(true)` - Database is intact
    /// * `Ok(false)` - Database has integrity issues
    /// * `Err(_)` - Failed to run integrity check
    pub fn verify_integrity(&self) -> Result<bool> {
        let result: String = self.conn.query_row(
            "PRAGMA integrity_check",
            [],
            |row| row.get(0),
        )?;

        Ok(result == "ok")
    }

    /// Vacuum the database to reclaim space after deletes
    pub fn vacuum(&self) -> Result<()> {
        self.conn.execute_batch("VACUUM")?;
        Ok(())
    }

    /// Create a backup of the database to the specified path
    ///
    /// Uses SQLite's backup API to create a consistent backup
    pub fn create_backup(&self, backup_path: &Path) -> Result<()> {
        use rusqlite::backup::Backup;
        use std::time::Duration;

        // Open backup database
        let mut backup_conn = Connection::open(backup_path)?;

        // Run backup
        let backup = Backup::new(&self.conn, &mut backup_conn)?;
        backup.run_to_completion(5, Duration::from_millis(250), None)?;

        Ok(())
    }

    /// Repair catalog if integrity check fails
    ///
    /// # Arguments
    /// * `backup_dir` - Directory to store backup of corrupted database
    ///
    /// # Returns
    /// * `Ok(true)` - Repair was performed
    /// * `Ok(false)` - No repair needed (database was healthy)
    /// * `Err(_)` - Repair failed
    pub fn repair_if_needed(&mut self, backup_dir: &Path) -> Result<bool> {
        // Check integrity first
        if self.verify_integrity()? {
            return Ok(false); // No repair needed
        }

        // Database is corrupted - create backup
        std::fs::create_dir_all(backup_dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = backup_dir.join(format!("corrupted_{}.db", timestamp));

        // Try to backup what we can
        let _ = self.create_backup(&backup_path);

        // Try to recover: re-create schema
        // Note: This is a basic recovery - in production you'd want more sophisticated recovery
        self.initialize()?;

        Ok(true)
    }

    /// Get count of rejected photos
    pub fn get_rejected_count(&self) -> Result<u32> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM photos WHERE flag = 'reject'",
            [],
            |row| row.get(0),
        )?;
        Ok(count as u32)
    }

    /// Delete all photos with a specific flag
    pub fn delete_by_flag(&self, flag: &str) -> Result<u32> {
        let deleted = self.conn.execute(
            "DELETE FROM photos WHERE flag = ?",
            params![flag],
        )?;
        Ok(deleted as u32)
    }

    /// Toggle photo in/out of quick collection (special collection named "_quick_collection")
    /// Returns true if now in collection, false if removed
    pub fn toggle_quick_collection(&self, photo_id: &str) -> Result<bool> {
        // Ensure quick collection exists
        let collection_id = self.get_or_create_collection("_quick_collection")?;

        // Check if photo is already in the collection
        let in_collection: bool = self.conn.query_row(
            "SELECT 1 FROM photo_collections WHERE photo_id = ? AND collection_id = ? LIMIT 1",
            params![photo_id, &collection_id],
            |_| Ok(true),
        ).unwrap_or(false);

        if in_collection {
            // Remove from collection
            self.conn.execute(
                "DELETE FROM photo_collections WHERE photo_id = ? AND collection_id = ?",
                params![photo_id, &collection_id],
            )?;
            Ok(false)
        } else {
            // Add to collection
            self.conn.execute(
                "INSERT INTO photo_collections (photo_id, collection_id) VALUES (?, ?)",
                params![photo_id, &collection_id],
            )?;
            Ok(true)
        }
    }

    /// Get all photo IDs in the quick collection
    pub fn get_quick_collection(&self) -> Result<Vec<String>> {
        let collection_id = self.get_or_create_collection("_quick_collection")?;

        let mut stmt = self.conn.prepare(
            "SELECT photo_id FROM photo_collections WHERE collection_id = ?"
        )?;

        let photo_ids = stmt.query_map(params![&collection_id], |row| {
            row.get::<_, String>(0)
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(photo_ids)
    }

    /// Get or create a collection by name, returning its ID
    fn get_or_create_collection(&self, name: &str) -> Result<String> {
        // Check if collection exists
        let existing: Option<String> = self.conn.query_row(
            "SELECT id FROM collections WHERE name = ? LIMIT 1",
            params![name],
            |row| row.get(0),
        ).ok();

        if let Some(id) = existing {
            return Ok(id);
        }

        // Create new collection
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO collections (id, name, type, created_at, updated_at) VALUES (?, ?, 'manual', ?, ?)",
            params![&id, name, &now, &now],
        )?;

        Ok(id)
    }
}

/// Smart collection rule
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmartRule {
    pub field: String,  // "rating", "flag", "camera_make", "date_taken"
    pub op: String,     // "gte", "lte", "eq", "contains", "in_last_days"
    pub value: String,
}

/// Smart collection rules container
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmartCollectionRules {
    pub match_all: bool,
    pub rules: Vec<SmartRule>,
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

    #[test]
    fn test_smart_collection_rating_filter() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        // Insert photos with different ratings
        for rating in 1..=5 {
            let id = Uuid::new_v4().to_string();
            catalog
                .conn
                .execute(
                    "INSERT INTO photos (id, file_path, file_name, file_size, date_imported, rating)
                     VALUES (?, ?, ?, ?, ?, ?)",
                    params![&id, format!("/test{}.jpg", rating), format!("test{}.jpg", rating), 1000, &now, rating],
                )
                .unwrap();
        }

        // Filter rating >= 4
        let rules = SmartCollectionRules {
            match_all: true,
            rules: vec![SmartRule {
                field: "rating".to_string(),
                op: "gte".to_string(),
                value: "4".to_string(),
            }],
        };

        let results = catalog.evaluate_smart_collection(&rules).unwrap();
        assert_eq!(results.len(), 2); // Should get rating 4 and 5
    }

    #[test]
    fn test_smart_collection_flag_filter() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        // Insert photos with different flags
        for (i, flag) in ["pick", "reject", ""].iter().enumerate() {
            let id = Uuid::new_v4().to_string();
            catalog
                .conn
                .execute(
                    "INSERT INTO photos (id, file_path, file_name, file_size, date_imported, flag)
                     VALUES (?, ?, ?, ?, ?, ?)",
                    params![&id, format!("/test{}.jpg", i), format!("test{}.jpg", i), 1000, &now, flag],
                )
                .unwrap();
        }

        // Filter flag = pick
        let rules = SmartCollectionRules {
            match_all: true,
            rules: vec![SmartRule {
                field: "flag".to_string(),
                op: "eq".to_string(),
                value: "pick".to_string(),
            }],
        };

        let results = catalog.evaluate_smart_collection(&rules).unwrap();
        assert_eq!(results.len(), 1); // Should get only "pick"
    }

    #[test]
    fn test_smart_collection_and_rules() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        // Insert test photos
        let test_cases = [
            (5, "pick"),   // Should match
            (4, "pick"),   // Should match
            (5, "reject"), // Should not match (wrong flag)
            (3, "pick"),   // Should not match (rating too low)
        ];

        for (i, (rating, flag)) in test_cases.iter().enumerate() {
            let id = Uuid::new_v4().to_string();
            catalog
                .conn
                .execute(
                    "INSERT INTO photos (id, file_path, file_name, file_size, date_imported, rating, flag)
                     VALUES (?, ?, ?, ?, ?, ?, ?)",
                    params![&id, format!("/test{}.jpg", i), format!("test{}.jpg", i), 1000, &now, rating, flag],
                )
                .unwrap();
        }

        // Filter rating >= 4 AND flag = pick
        let rules = SmartCollectionRules {
            match_all: true,
            rules: vec![
                SmartRule {
                    field: "rating".to_string(),
                    op: "gte".to_string(),
                    value: "4".to_string(),
                },
                SmartRule {
                    field: "flag".to_string(),
                    op: "eq".to_string(),
                    value: "pick".to_string(),
                },
            ],
        };

        let results = catalog.evaluate_smart_collection(&rules).unwrap();
        assert_eq!(results.len(), 2); // Should get first two photos only
    }

    #[test]
    fn test_create_stack() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        // Insert 3 photos
        let ids: Vec<String> = (0..3).map(|i| {
            let id = Uuid::new_v4().to_string();
            catalog.conn.execute(
                "INSERT INTO photos (id, file_path, file_name, file_size, date_imported)
                 VALUES (?, ?, ?, ?, ?)",
                params![&id, format!("/test{}.jpg", i), format!("test{}.jpg", i), 1000, &now],
            ).unwrap();
            id
        }).collect();

        // Create stack
        let stack_id = catalog.create_stack(&ids).unwrap();

        // Verify all photos have same stack_id
        for id in &ids {
            let photo = catalog.get_photo(id).unwrap().unwrap();
            assert_eq!(photo.stack_id, Some(stack_id.clone()));
            assert!(photo.stack_position.is_some());
        }

        // Verify stack ordering
        let stack = catalog.get_stack(&stack_id).unwrap();
        assert_eq!(stack.len(), 3);
        assert_eq!(stack[0].id, ids[0]);
        assert_eq!(stack[1].id, ids[1]);
        assert_eq!(stack[2].id, ids[2]);
    }

    #[test]
    fn test_unstack() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        let ids: Vec<String> = (0..3).map(|i| {
            let id = Uuid::new_v4().to_string();
            catalog.conn.execute(
                "INSERT INTO photos (id, file_path, file_name, file_size, date_imported)
                 VALUES (?, ?, ?, ?, ?)",
                params![&id, format!("/test{}.jpg", i), format!("test{}.jpg", i), 1000, &now],
            ).unwrap();
            id
        }).collect();

        let stack_id = catalog.create_stack(&ids).unwrap();

        // Unstack
        catalog.unstack(&stack_id).unwrap();

        // Verify stack_id is NULL
        for id in &ids {
            let photo = catalog.get_photo(id).unwrap().unwrap();
            assert_eq!(photo.stack_id, None);
            assert_eq!(photo.stack_position, None);
        }
    }

    #[test]
    fn test_stack_ordering() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        let ids: Vec<String> = (0..5).map(|i| {
            let id = Uuid::new_v4().to_string();
            catalog.conn.execute(
                "INSERT INTO photos (id, file_path, file_name, file_size, date_imported)
                 VALUES (?, ?, ?, ?, ?)",
                params![&id, format!("/test{}.jpg", i), format!("test{}.jpg", i), 1000, &now],
            ).unwrap();
            id
        }).collect();

        let stack_id = catalog.create_stack(&ids).unwrap();

        // Get stack and verify order
        let stack = catalog.get_stack(&stack_id).unwrap();
        for (i, photo) in stack.iter().enumerate() {
            assert_eq!(photo.stack_position, Some(i as i32));
        }
    }

    #[test]
    fn test_create_virtual_copy() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();

        catalog.conn.execute(
            "INSERT INTO photos (id, file_path, file_name, file_size, date_imported, camera_make, rating)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![&id, "/original.jpg", "original.jpg", 1000, &now, "Sony", 4],
        ).unwrap();

        // Create virtual copy
        let copy_id = catalog.create_virtual_copy(&id).unwrap();

        // Verify copy exists
        let copy = catalog.get_photo(&copy_id).unwrap().unwrap();
        assert_eq!(copy.virtual_copy_of, Some(id.clone()));
        assert_eq!(copy.file_path, "/original.jpg"); // Same file
        assert_eq!(copy.camera_make, Some("Sony".to_string())); // Same metadata
    }

    #[test]
    fn test_virtual_copy_inherits_metadata() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();

        catalog.conn.execute(
            "INSERT INTO photos (id, file_path, file_name, file_size, date_imported, camera_make, camera_model, rating)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![&id, "/original.jpg", "original.jpg", 1000, &now, "Canon", "EOS R5", 5],
        ).unwrap();

        let copy_id = catalog.create_virtual_copy(&id).unwrap();
        let copy = catalog.get_photo(&copy_id).unwrap().unwrap();

        assert_eq!(copy.camera_make, Some("Canon".to_string()));
        assert_eq!(copy.camera_model, Some("EOS R5".to_string()));
        assert_eq!(copy.rating, 5);
    }

    #[test]
    fn test_batch_rating() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        let ids: Vec<String> = (0..5).map(|i| {
            let id = Uuid::new_v4().to_string();
            catalog.conn.execute(
                "INSERT INTO photos (id, file_path, file_name, file_size, date_imported)
                 VALUES (?, ?, ?, ?, ?)",
                params![&id, format!("/test{}.jpg", i), format!("test{}.jpg", i), 1000, &now],
            ).unwrap();
            id
        }).collect();

        // Batch update rating
        let count = catalog.batch_update_rating(&ids, 3).unwrap();
        assert_eq!(count, 5);

        // Verify all have rating 3
        for id in &ids {
            let photo = catalog.get_photo(id).unwrap().unwrap();
            assert_eq!(photo.rating, 3);
        }
    }

    #[test]
    fn test_batch_flag() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        let ids: Vec<String> = (0..5).map(|i| {
            let id = Uuid::new_v4().to_string();
            catalog.conn.execute(
                "INSERT INTO photos (id, file_path, file_name, file_size, date_imported)
                 VALUES (?, ?, ?, ?, ?)",
                params![&id, format!("/test{}.jpg", i), format!("test{}.jpg", i), 1000, &now],
            ).unwrap();
            id
        }).collect();

        let count = catalog.batch_update_flag(&ids, "pick").unwrap();
        assert_eq!(count, 5);

        for id in &ids {
            let photo = catalog.get_photo(id).unwrap().unwrap();
            assert_eq!(photo.flag, "pick");
        }
    }

    #[test]
    fn test_batch_delete() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        let ids: Vec<String> = (0..5).map(|i| {
            let id = Uuid::new_v4().to_string();
            catalog.conn.execute(
                "INSERT INTO photos (id, file_path, file_name, file_size, date_imported)
                 VALUES (?, ?, ?, ?, ?)",
                params![&id, format!("/test{}.jpg", i), format!("test{}.jpg", i), 1000, &now],
            ).unwrap();
            id
        }).collect();

        // Delete first 3
        let delete_ids = ids[0..3].to_vec();
        let count = catalog.batch_delete(&delete_ids).unwrap();
        assert_eq!(count, 3);

        // Verify 2 remain
        assert_eq!(catalog.photo_count().unwrap(), 2);

        // Verify specific photos remain
        assert!(catalog.get_photo(&ids[3]).unwrap().is_some());
        assert!(catalog.get_photo(&ids[4]).unwrap().is_some());
        assert!(catalog.get_photo(&ids[0]).unwrap().is_none());
    }

    #[test]
    fn test_delete_by_flag_rejected() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        // Insert 5 photos: 2 rejected, 3 normal
        for i in 0..5 {
            let id = Uuid::new_v4().to_string();
            let flag = if i < 2 { "reject" } else { "none" };
            catalog.conn.execute(
                "INSERT INTO photos (id, file_path, file_name, file_size, date_imported, flag)
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![&id, format!("/test{}.jpg", i), format!("test{}.jpg", i), 1000, &now, flag],
            ).unwrap();
        }

        assert_eq!(catalog.photo_count().unwrap(), 5);
        assert_eq!(catalog.get_rejected_count().unwrap(), 2);

        // Delete all rejected
        let deleted = catalog.delete_by_flag("reject").unwrap();
        assert_eq!(deleted, 2);

        // Verify 3 remain
        assert_eq!(catalog.photo_count().unwrap(), 3);
        assert_eq!(catalog.get_rejected_count().unwrap(), 0);
    }

    #[test]
    fn test_quick_collection_toggle() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();

        catalog.conn.execute(
            "INSERT INTO photos (id, file_path, file_name, file_size, date_imported)
             VALUES (?, ?, ?, ?, ?)",
            params![&id, "/test.jpg", "test.jpg", 1000, &now],
        ).unwrap();

        // Toggle on -> should be in collection
        let in_collection = catalog.toggle_quick_collection(&id).unwrap();
        assert!(in_collection);

        let photos_in_qc = catalog.get_quick_collection().unwrap();
        assert_eq!(photos_in_qc.len(), 1);
        assert_eq!(photos_in_qc[0], id);

        // Toggle off -> should be removed
        let in_collection = catalog.toggle_quick_collection(&id).unwrap();
        assert!(!in_collection);

        let photos_in_qc = catalog.get_quick_collection().unwrap();
        assert_eq!(photos_in_qc.len(), 0);
    }

    #[test]
    fn test_get_rejected_count() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        // Insert 3 rejected photos
        for i in 0..3 {
            let id = Uuid::new_v4().to_string();
            catalog.conn.execute(
                "INSERT INTO photos (id, file_path, file_name, file_size, date_imported, flag)
                 VALUES (?, ?, ?, ?, ?, 'reject')",
                params![&id, format!("/test{}.jpg", i), format!("test{}.jpg", i), 1000, &now],
            ).unwrap();
        }

        assert_eq!(catalog.get_rejected_count().unwrap(), 3);
    }

    #[test]
    fn test_search_by_filename() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        // Insert photo with specific name
        let id = Uuid::new_v4().to_string();
        catalog.conn.execute(
            "INSERT INTO photos (id, file_path, file_name, file_size, date_imported)
             VALUES (?, ?, ?, ?, ?)",
            params![&id, "/path/DSC_4523.jpg", "DSC_4523.jpg", 1000, &now],
        ).unwrap();

        // Search by prefix
        let results = catalog.search("DSC", 100).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_name, "DSC_4523.jpg");
    }

    #[test]
    fn test_search_by_camera() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        // Insert photo with Sony camera
        let id = Uuid::new_v4().to_string();
        catalog.conn.execute(
            "INSERT INTO photos (id, file_path, file_name, file_size, date_imported, camera_make, camera_model)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![&id, "/test.jpg", "test.jpg", 1000, &now, "Sony", "A7III"],
        ).unwrap();

        // Search by camera make
        let results = catalog.search("Sony", 100).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].camera_make, Some("Sony".to_string()));
    }

    #[test]
    fn test_search_no_results() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        let id = Uuid::new_v4().to_string();
        catalog.conn.execute(
            "INSERT INTO photos (id, file_path, file_name, file_size, date_imported)
             VALUES (?, ?, ?, ?, ?)",
            params![&id, "/test.jpg", "test.jpg", 1000, &now],
        ).unwrap();

        let results = catalog.search("ZZZNOMATCH", 100).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_prefix() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        // Insert photo with Canon camera
        let id = Uuid::new_v4().to_string();
        catalog.conn.execute(
            "INSERT INTO photos (id, file_path, file_name, file_size, date_imported, camera_make, camera_model)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![&id, "/test.jpg", "test.jpg", 1000, &now, "Canon", "EOS R5"],
        ).unwrap();

        // Search with prefix
        let results = catalog.search("Canon", 100).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].camera_make, Some("Canon".to_string()));
    }

    #[test]
    fn test_integrity_on_fresh_catalog() {
        let catalog = Catalog::in_memory().unwrap();
        assert!(catalog.verify_integrity().unwrap());
    }

    #[test]
    fn test_backup_creates_file() {
        use std::env;

        let catalog = Catalog::in_memory().unwrap();

        // Insert test data
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        catalog.conn.execute(
            "INSERT INTO photos (id, file_path, file_name, file_size, date_imported)
             VALUES (?, ?, ?, ?, ?)",
            params![&id, "/test.jpg", "test.jpg", 1000, &now],
        ).unwrap();

        // Create backup
        let temp_dir = env::temp_dir();
        let backup_path = temp_dir.join(format!("test_backup_{}.db", Uuid::new_v4()));

        catalog.create_backup(&backup_path).unwrap();

        // Verify backup file exists
        assert!(backup_path.exists());

        // Verify backup can be opened and has data
        let backup_catalog = Catalog::open(&backup_path).unwrap();
        assert_eq!(backup_catalog.photo_count().unwrap(), 1);

        // Clean up
        let _ = std::fs::remove_file(backup_path);
    }

    #[test]
    fn test_vacuum_succeeds() {
        let catalog = Catalog::in_memory().unwrap();
        let now = Utc::now().to_rfc3339();

        // Insert photos
        for i in 0..10 {
            let id = Uuid::new_v4().to_string();
            catalog.conn.execute(
                "INSERT INTO photos (id, file_path, file_name, file_size, date_imported)
                 VALUES (?, ?, ?, ?, ?)",
                params![&id, format!("/test{}.jpg", i), format!("test{}.jpg", i), 1000, &now],
            ).unwrap();
        }

        // Delete some
        catalog.conn.execute("DELETE FROM photos WHERE file_name LIKE 'test0%'", []).unwrap();

        // Vacuum should succeed
        catalog.vacuum().unwrap();

        // Verify data is still intact
        assert!(catalog.photo_count().unwrap() > 0);
    }
}
