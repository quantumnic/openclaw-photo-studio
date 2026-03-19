//! Data models for catalog operations

use serde::{Deserialize, Serialize};

/// Represents a single photo record in the catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoRecord {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub date_taken: Option<String>,
    pub date_imported: String,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub rating: u8,
    pub color_label: String,
    pub flag: String,
    pub has_edits: bool,
}

/// Result of an import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub total: u32,
    pub inserted: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}

/// Filter options for querying photos
#[derive(Debug, Clone, Default)]
pub struct PhotoFilter {
    pub rating_min: Option<u8>,
    pub flag: Option<String>,
    pub color_label: Option<String>,
    pub search: Option<String>,
    pub limit: u32,
    pub offset: u32,
}

/// Sort order for photo queries
#[derive(Debug, Clone, Default)]
pub enum SortOrder {
    #[default]
    DateTaken,
    FileName,
    Rating,
    FileSize,
}

impl SortOrder {
    pub fn to_sql(&self) -> &'static str {
        match self {
            SortOrder::DateTaken => "date_taken DESC",
            SortOrder::FileName => "file_name ASC",
            SortOrder::Rating => "rating DESC",
            SortOrder::FileSize => "file_size DESC",
        }
    }
}
