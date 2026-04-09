// ============================================================
// Sift - Rust Models
// Data structures shared between Rust backend and Vue frontend
// ============================================================

use serde::{Deserialize, Serialize};

/// Status of a photo pair in the culling workflow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PhotoStatus {
    Unprocessed,
    Starred,
    Deleted,
    Skipped,
}

/// A paired photo: one JPG + optional RAW
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoPair {
    pub id: String,
    pub jpg_path: String,
    pub raw_path: Option<String>,
    pub raw_format: Option<String>,
    pub status: PhotoStatus,
    pub thumbnail_path: Option<String>,
    pub dominant_color: Option<String>,
}

/// Result of scanning a folder
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub pairs: Vec<PhotoPair>,
    pub total_files: usize,
    pub paired_count: usize,
    pub jpg_only_count: usize,
}

/// Thumbnail generation result for a single photo
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailResult {
    pub id: String,
    pub path: String,
    pub dominant_color: String,
}

/// EXIF metadata extracted from a JPG
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExifData {
    pub camera: String,
    pub lens: String,
    pub iso: u32,
    pub aperture: String,
    pub shutter_speed: String,
    pub focal_length: String,
    pub date_taken: String,
    pub dimensions: Dimensions,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

/// Archive operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveResult {
    pub moved_count: usize,
    pub jpg_folder: String,
    pub raw_folder: String,
}

/// Archive/Export progress event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
}

/// Export result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub exported_count: usize,
    pub export_folder: String,
}

/// Input for archive command
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePairInput {
    pub jpg_path: String,
    pub raw_path: Option<String>,
    pub status: String,
}

/// Input for export command
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPairInput {
    pub jpg_path: String,
    pub raw_path: Option<String>,
}

/// Input for thumbnail generation
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailInput {
    pub id: String,
    pub jpg_path: String,
}
