// ============================================================
// Sift - Archive Command
// Moves surviving photos into RAW/ and JPG/ subfolders
// ============================================================

use crate::models::photo::{ArchivePairInput, ArchiveProgress, ArchiveResult};
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub fn archive_photos(
    app: AppHandle,
    folder_path: String,
    pairs: Vec<ArchivePairInput>,
) -> Result<ArchiveResult, String> {
    let base = Path::new(&folder_path);
    let jpg_dir = base.join("JPG");
    let raw_dir = base.join("RAW");

    // Create subdirectories
    fs::create_dir_all(&jpg_dir).map_err(|e| format!("Failed to create JPG dir: {}", e))?;
    fs::create_dir_all(&raw_dir).map_err(|e| format!("Failed to create RAW dir: {}", e))?;

    let total = pairs.len();
    let mut moved_count: usize = 0;

    for (i, pair) in pairs.iter().enumerate() {
        // Only archive non-deleted pairs
        if pair.status == "deleted" {
            continue;
        }

        // Move JPG
        let jpg_path = Path::new(&pair.jpg_path);
        if jpg_path.exists() {
            let filename = jpg_path.file_name().unwrap_or_default();

            // Emit progress
            let _ = app.emit(
                "archive-progress",
                ArchiveProgress {
                    current: i + 1,
                    total,
                    current_file: filename.to_string_lossy().to_string(),
                },
            );

            // Skip if file is already in the target directory
            if !is_already_in_dir(jpg_path, &jpg_dir) {
                let dest = unique_path(&jpg_dir.join(filename));
                fs::rename(&jpg_path, &dest)
                    .map_err(|e| format!("Failed to move JPG {}: {}", pair.jpg_path, e))?;
                moved_count += 1;
            }
        }

        // Move RAW
        if let Some(raw_path_str) = &pair.raw_path {
            let raw_path = Path::new(raw_path_str);
            if raw_path.exists() && !is_already_in_dir(raw_path, &raw_dir) {
                let filename = raw_path.file_name().unwrap_or_default();
                let dest = unique_path(&raw_dir.join(filename));
                fs::rename(&raw_path, &dest)
                    .map_err(|e| format!("Failed to move RAW {}: {}", raw_path_str, e))?;
                moved_count += 1;
            }
        }
    }

    Ok(ArchiveResult {
        moved_count,
        jpg_folder: jpg_dir.to_string_lossy().to_string(),
        raw_folder: raw_dir.to_string_lossy().to_string(),
    })
}

/// Check if a file is already located inside the target directory
fn is_already_in_dir(file: &Path, dir: &Path) -> bool {
    let file_parent = match file.parent().and_then(|p| p.canonicalize().ok()) {
        Some(p) => p,
        None => return false,
    };
    let target = match dir.canonicalize().ok() {
        Some(p) => p,
        None => return false,
    };
    file_parent == target
}

/// Generate a unique file path by appending _1, _2, etc. if file already exists
fn unique_path(path: &Path) -> std::path::PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let parent = path.parent().unwrap_or(path);

    let mut counter = 1;
    loop {
        let new_name = if ext.is_empty() {
            format!("{}_{}", stem, counter)
        } else {
            format!("{}_{}.{}", stem, counter, ext)
        };
        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}
