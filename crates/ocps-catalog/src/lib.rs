//! ocps-catalog — OpenClaw Photo Studio Catalog Engine
//!
//! SQLite-based catalog with FTS5 search, collections, keywords.

pub mod db;
pub mod geo;
pub mod lightroom_import;
pub mod metadata_template;
pub mod models;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub mod schema {
    //! SQLite schema definitions

    pub const SCHEMA_VERSION: u32 = 1;

    pub const CREATE_PHOTOS: &str = r#"
        CREATE TABLE IF NOT EXISTS photos (
            id              TEXT PRIMARY KEY,
            file_path       TEXT NOT NULL,
            file_name       TEXT NOT NULL,
            file_size       INTEGER,
            file_hash       TEXT,
            mime_type       TEXT,
            width           INTEGER,
            height          INTEGER,
            orientation     INTEGER DEFAULT 1,
            date_taken      TEXT,
            date_imported   TEXT NOT NULL,
            camera_make     TEXT,
            camera_model    TEXT,
            lens            TEXT,
            focal_length    REAL,
            aperture        REAL,
            shutter_speed   TEXT,
            iso             INTEGER,
            gps_lat         REAL,
            gps_lon         REAL,
            rating          INTEGER DEFAULT 0,
            color_label     TEXT DEFAULT 'none',
            flag            TEXT DEFAULT 'none',
            has_edits       INTEGER DEFAULT 0,
            edit_version    INTEGER DEFAULT 0,
            folder_id       TEXT,
            stack_id        TEXT,
            stack_position  INTEGER,
            virtual_copy_of TEXT REFERENCES photos(id),
            copyright       TEXT,
            creator         TEXT,
            city            TEXT,
            country         TEXT,
            face_regions    TEXT,
            schema_version  INTEGER DEFAULT 1
        )
    "#;

    pub const CREATE_COLLECTIONS: &str = r#"
        CREATE TABLE IF NOT EXISTS collections (
            id              TEXT PRIMARY KEY,
            name            TEXT NOT NULL,
            type            TEXT DEFAULT 'manual',
            parent_id       TEXT REFERENCES collections(id),
            smart_rules     TEXT,
            sort_order      INTEGER DEFAULT 0,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        )
    "#;

    pub const CREATE_PHOTO_COLLECTIONS: &str = r#"
        CREATE TABLE IF NOT EXISTS photo_collections (
            photo_id        TEXT REFERENCES photos(id) ON DELETE CASCADE,
            collection_id   TEXT REFERENCES collections(id) ON DELETE CASCADE,
            added_at        TEXT DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (photo_id, collection_id)
        )
    "#;

    pub const CREATE_KEYWORDS: &str = r#"
        CREATE TABLE IF NOT EXISTS keywords (
            id              TEXT PRIMARY KEY,
            name            TEXT NOT NULL,
            parent_id       TEXT REFERENCES keywords(id),
            synonyms        TEXT
        )
    "#;

    pub const CREATE_PHOTO_KEYWORDS: &str = r#"
        CREATE TABLE IF NOT EXISTS photo_keywords (
            photo_id        TEXT REFERENCES photos(id) ON DELETE CASCADE,
            keyword_id      TEXT REFERENCES keywords(id) ON DELETE CASCADE,
            PRIMARY KEY (photo_id, keyword_id)
        )
    "#;

    pub const CREATE_EDITS: &str = r#"
        CREATE TABLE IF NOT EXISTS edits (
            id              TEXT PRIMARY KEY,
            photo_id        TEXT REFERENCES photos(id) ON DELETE CASCADE,
            version         INTEGER NOT NULL,
            edit_data       TEXT NOT NULL,
            created_at      TEXT NOT NULL,
            snapshot_name   TEXT,
            is_current      INTEGER DEFAULT 1
        )
    "#;

    pub const CREATE_FTS: &str = r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS photos_fts USING fts5(
            id UNINDEXED,
            file_name,
            camera_make,
            camera_model,
            content='photos',
            content_rowid='rowid'
        )
    "#;

    pub const CREATE_FTS_TRIGGERS: &str = r#"
        CREATE TRIGGER IF NOT EXISTS photos_fts_insert AFTER INSERT ON photos BEGIN
            INSERT INTO photos_fts(rowid, id, file_name, camera_make, camera_model)
            VALUES (new.rowid, new.id, new.file_name, new.camera_make, new.camera_model);
        END;

        CREATE TRIGGER IF NOT EXISTS photos_fts_delete AFTER DELETE ON photos BEGIN
            DELETE FROM photos_fts WHERE rowid = old.rowid;
        END;

        CREATE TRIGGER IF NOT EXISTS photos_fts_update AFTER UPDATE ON photos BEGIN
            DELETE FROM photos_fts WHERE rowid = old.rowid;
            INSERT INTO photos_fts(rowid, id, file_name, camera_make, camera_model)
            VALUES (new.rowid, new.id, new.file_name, new.camera_make, new.camera_model);
        END;
    "#;

    pub const CREATE_MIGRATIONS: &str = r#"
        CREATE TABLE IF NOT EXISTS _migrations (
            version         INTEGER PRIMARY KEY,
            applied_at      TEXT NOT NULL,
            description     TEXT
        )
    "#;

    pub const INDEXES: &[&str] = &[
        "CREATE INDEX IF NOT EXISTS idx_photos_date ON photos(date_taken)",
        "CREATE INDEX IF NOT EXISTS idx_photos_rating ON photos(rating)",
        "CREATE INDEX IF NOT EXISTS idx_photos_flag ON photos(flag)",
        "CREATE INDEX IF NOT EXISTS idx_photos_camera ON photos(camera_make, camera_model)",
        "CREATE INDEX IF NOT EXISTS idx_photos_hash ON photos(file_hash)",
    ];
}

// Re-export main types for convenience
pub use db::Catalog;
pub use geo::GeoPhoto;
pub use lightroom_import::{import_lightroom_catalog, LightroomImportResult};
pub use models::{ImportResult, PhotoFilter, PhotoRecord, SortOrder};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_catalog() {
        let catalog = db::Catalog::in_memory().unwrap();
        assert_eq!(catalog.photo_count().unwrap(), 0);
    }

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
