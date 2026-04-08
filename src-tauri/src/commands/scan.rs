// ============================================================
// Sift - Scan Command
// Smart scan: detects folder structure and pairs JPG with RAW
// Supports: loose files, archived JPG/RAW subdirectories, or both
// ============================================================

use crate::models::photo::{PhotoPair, PhotoStatus, ScanResult};
use crate::utils::file_types::{is_jpg, is_raw, raw_format_name};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

#[tauri::command]
pub async fn scan_folder(folder_path: String) -> Result<ScanResult, String> {
    tokio::task::spawn_blocking(move || scan_folder_sync(&folder_path))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Collect JPG and RAW files from a single directory (non-recursive)
fn collect_files(
    dir: &Path,
    jpg_map: &mut HashMap<String, String>,
    raw_map: &mut HashMap<String, (String, String)>,
) -> usize {
    let mut count: usize = 0;
    for entry in WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }

        if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            let stem = entry_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let full_path = entry_path.to_string_lossy().to_string();

            if is_jpg(&ext_lower) {
                count += 1;
                jpg_map.insert(stem, full_path);
            } else if is_raw(&ext_lower) {
                count += 1;
                raw_map.insert(stem, (full_path, ext_lower));
            }
        }
    }
    count
}

fn scan_folder_sync(folder_path: &str) -> Result<ScanResult, String> {
    let path = Path::new(folder_path);
    if !path.exists() || !path.is_dir() {
        return Err(format!("Folder does not exist: {}", folder_path));
    }

    let mut jpg_map: HashMap<String, String> = HashMap::new();
    let mut raw_map: HashMap<String, (String, String)> = HashMap::new();
    let mut total_files: usize = 0;

    // Phase 1: Scan root directory for loose JPG/RAW files
    total_files += collect_files(path, &mut jpg_map, &mut raw_map);

    // Phase 2: If root has no JPG files, check for archived JPG/ + RAW/ subdirectories
    if jpg_map.is_empty() {
        let jpg_dir = path.join("JPG");
        let raw_dir = path.join("RAW");

        if jpg_dir.is_dir() {
            total_files += collect_files(&jpg_dir, &mut jpg_map, &mut raw_map);
        }
        if raw_dir.is_dir() {
            total_files += collect_files(&raw_dir, &mut jpg_map, &mut raw_map);
        }
    }

    // Build pairs from JPG map, matching RAW by stem
    let mut pairs: Vec<PhotoPair> = Vec::new();
    let mut paired_count: usize = 0;
    let mut jpg_only_count: usize = 0;

    for (stem, jpg_path) in &jpg_map {
        let (raw_path, raw_format) = if let Some((rp, ext)) = raw_map.get(stem) {
            paired_count += 1;
            (Some(rp.clone()), Some(raw_format_name(ext)))
        } else {
            jpg_only_count += 1;
            (None, None)
        };

        pairs.push(PhotoPair {
            id: uuid::Uuid::new_v4().to_string(),
            jpg_path: jpg_path.clone(),
            raw_path,
            raw_format,
            status: PhotoStatus::Unprocessed,
            thumbnail_path: None,
            dominant_color: None,
        });
    }

    // Natural sort by filename
    pairs.sort_by(|a, b| {
        let name_a = Path::new(&a.jpg_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let name_b = Path::new(&b.jpg_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        natord::compare(name_a, name_b)
    });

    Ok(ScanResult {
        pairs,
        total_files,
        paired_count,
        jpg_only_count,
    })
}
