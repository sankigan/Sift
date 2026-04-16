// ============================================================
// Sift - Export Command
// Copies starred photos (JPG + RAW) to target directory
// Supports RAW-only pairs: exports RAW original only
// ============================================================

use crate::models::photo::{ArchiveProgress, ExportPairInput, ExportResult, PhotoSource};
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn export_picks(
    app: AppHandle,
    pairs: Vec<ExportPairInput>,
    target_folder: String,
) -> Result<ExportResult, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || export_picks_sync(app_clone, &pairs, &target_folder))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

fn export_picks_sync(
    app: AppHandle,
    pairs: &[ExportPairInput],
    target_folder: &str,
) -> Result<ExportResult, String> {
    let target = Path::new(target_folder);
    fs::create_dir_all(target).map_err(|e| format!("Failed to create export dir: {}", e))?;

    let total = pairs.len();
    let mut exported_count: usize = 0;

    for (i, pair) in pairs.iter().enumerate() {
        let is_raw_preview = pair.source == PhotoSource::RawPreview;

        if is_raw_preview {
            // RAW Only: export the RAW original (not the temporary preview)
            if let Some(raw_path_str) = &pair.raw_path {
                let raw_path = Path::new(raw_path_str);
                if raw_path.exists() {
                    let filename = raw_path.file_name().unwrap_or_default();
                    let dest = target.join(filename);

                    let _ = app.emit(
                        "export-progress",
                        ArchiveProgress {
                            current: i + 1,
                            total,
                            current_file: filename.to_string_lossy().to_string(),
                        },
                    );

                    fs::copy(&raw_path, &dest)
                        .map_err(|e| format!("Failed to copy RAW: {}", e))?;
                    exported_count += 1;
                }
            }

            // Export associated XMP sidecars
            for xmp_path_str in &pair.xmp_paths {
                let xmp_path = Path::new(xmp_path_str);
                if xmp_path.exists() {
                    let xmp_filename = xmp_path.file_name().unwrap_or_default();
                    let dest = target.join(xmp_filename);
                    fs::copy(xmp_path, &dest)
                        .map_err(|e| format!("Failed to copy XMP: {}", e))?;
                }
            }
        } else {
            // Normal: copy JPG + optional RAW
            let jpg_path = Path::new(&pair.jpg_path);
            if jpg_path.exists() {
                let filename = jpg_path.file_name().unwrap_or_default();
                let dest = target.join(filename);

                let _ = app.emit(
                    "export-progress",
                    ArchiveProgress {
                        current: i + 1,
                        total,
                        current_file: filename.to_string_lossy().to_string(),
                    },
                );

                fs::copy(&jpg_path, &dest)
                    .map_err(|e| format!("Failed to copy JPG: {}", e))?;
                exported_count += 1;
            }

            // Copy RAW
            if let Some(raw_path_str) = &pair.raw_path {
                let raw_path = Path::new(raw_path_str);
                if raw_path.exists() {
                    let filename = raw_path.file_name().unwrap_or_default();
                    let dest = target.join(filename);
                    fs::copy(&raw_path, &dest)
                        .map_err(|e| format!("Failed to copy RAW: {}", e))?;
                    exported_count += 1;
                }
            }

            // Export associated XMP sidecars
            for xmp_path_str in &pair.xmp_paths {
                let xmp_path = Path::new(xmp_path_str);
                if xmp_path.exists() {
                    let xmp_filename = xmp_path.file_name().unwrap_or_default();
                    let dest = target.join(xmp_filename);
                    fs::copy(xmp_path, &dest)
                        .map_err(|e| format!("Failed to copy XMP: {}", e))?;
                }
            }
        }
    }

    Ok(ExportResult {
        exported_count,
        export_folder: target_folder.to_string(),
    })
}
