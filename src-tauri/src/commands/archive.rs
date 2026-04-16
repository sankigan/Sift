// ============================================================
// Sift - Archive Command
// Moves surviving photos into RAW/ and JPG/ subfolders
// Supports RAW-only pairs: RAW → RAW/, preview JPG → JPG/
// ============================================================

use crate::models::photo::{ArchivePairInput, ArchiveProgress, ArchiveResult, PhotoSource};
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Emitter};

/// Move a file to the destination, falling back to copy+remove if rename fails
/// (e.g. cross-volume moves on Windows).
fn move_file(src: &Path, dest: &Path) -> Result<(), String> {
    let src_str = src.to_string_lossy();
    let dest_str = dest.to_string_lossy();

    match fs::rename(src, dest) {
        Ok(()) => Ok(()),
        Err(_) => {
            // Fallback: copy then remove (handles cross-volume moves on Windows)
            fs::copy(src, dest)
                .map_err(|e| format!("Failed to copy {} -> {}: {}", src_str, dest_str, e))?;
            fs::remove_file(src)
                .map_err(|e| format!("Failed to remove source after copy {}: {}", src_str, e))?;
            Ok(())
        }
    }
}

#[tauri::command]
pub async fn archive_photos(
    app: AppHandle,
    folder_path: String,
    pairs: Vec<ArchivePairInput>,
) -> Result<ArchiveResult, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || archive_photos_sync(app_clone, &folder_path, &pairs))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

fn archive_photos_sync(
    app: AppHandle,
    folder_path: &str,
    pairs: &[ArchivePairInput],
) -> Result<ArchiveResult, String> {
    let base = Path::new(folder_path);
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

        let is_raw_preview = pair.source == PhotoSource::RawPreview;

        if is_raw_preview {
            // RAW Only: move RAW to RAW/, move preview JPG to JPG/
            // (so re-opening folder later creates a proper JPG+RAW pair)

            // Move RAW first
            if let Some(raw_path_str) = &pair.raw_path {
                let raw_path = Path::new(raw_path_str);
                if raw_path.exists() {
                    let filename = raw_path.file_name().unwrap_or_default();

                    // Emit progress with RAW filename
                    let _ = app.emit(
                        "archive-progress",
                        ArchiveProgress {
                            current: i + 1,
                            total,
                            current_file: filename.to_string_lossy().to_string(),
                        },
                    );

                    if !is_already_in_dir(raw_path, &raw_dir) {
                        let dest = unique_path(&raw_dir.join(filename));
                        move_file(raw_path, &dest)?;
                        moved_count += 1;
                    }
                }
            }

            // Move preview JPG to JPG/ (so user sees it paired next time)
            let jpg_path = Path::new(&pair.jpg_path);
            if jpg_path.exists() {
                // Use the RAW stem with .jpg extension for the destination
                let dest_name = if let Some(raw_path_str) = &pair.raw_path {
                    let raw_stem = Path::new(raw_path_str)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("preview");
                    format!("{}.jpg", raw_stem)
                } else {
                    jpg_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                };

                if !is_already_in_dir(jpg_path, &jpg_dir) {
                    let dest = unique_path(&jpg_dir.join(&dest_name));
                    // Copy rather than rename since it's a temp file
                    fs::copy(&jpg_path, &dest)
                        .map_err(|e| format!("Failed to copy preview JPG: {}", e))?;
                    // Clean up the temp preview
                    let _ = fs::remove_file(&jpg_path);
                    moved_count += 1;
                }
            }

            // Move XMP sidecars to RAW/ (they belong with the RAW file)
            for xmp_path_str in &pair.xmp_paths {
                let xmp_path = Path::new(xmp_path_str);
                if xmp_path.exists() && !is_already_in_dir(xmp_path, &raw_dir) {
                    let xmp_filename = xmp_path.file_name().unwrap_or_default();
                    let dest = unique_path(&raw_dir.join(xmp_filename));
                    move_file(xmp_path, &dest)?;
                }
            }
        } else {
            // Normal: move JPG to JPG/, RAW to RAW/
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
                    move_file(jpg_path, &dest)?;
                    moved_count += 1;
                }
            }

            // Move RAW
            if let Some(raw_path_str) = &pair.raw_path {
                let raw_path = Path::new(raw_path_str);
                if raw_path.exists() && !is_already_in_dir(raw_path, &raw_dir) {
                    let filename = raw_path.file_name().unwrap_or_default();
                    let dest = unique_path(&raw_dir.join(filename));
                    move_file(raw_path, &dest)?;
                    moved_count += 1;
                }
            }

            // Move XMP sidecars: if RAW exists, follow RAW to RAW/; otherwise follow JPG to JPG/
            let xmp_target_dir = if pair.raw_path.is_some() { &raw_dir } else { &jpg_dir };
            for xmp_path_str in &pair.xmp_paths {
                let xmp_path = Path::new(xmp_path_str);
                if xmp_path.exists() && !is_already_in_dir(xmp_path, xmp_target_dir) {
                    let xmp_filename = xmp_path.file_name().unwrap_or_default();
                    let dest = unique_path(&xmp_target_dir.join(xmp_filename));
                    move_file(xmp_path, &dest)?;
                }
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
