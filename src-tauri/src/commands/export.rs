// ============================================================
// Sift - Export Command
// Copies starred photos (JPG + RAW) to target directory
// ============================================================

use crate::models::photo::{ArchiveProgress, ExportPairInput, ExportResult};
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub fn export_picks(
    app: AppHandle,
    pairs: Vec<ExportPairInput>,
    target_folder: String,
) -> Result<ExportResult, String> {
    let target = Path::new(&target_folder);
    fs::create_dir_all(target).map_err(|e| format!("Failed to create export dir: {}", e))?;

    let total = pairs.len();
    let mut exported_count: usize = 0;

    for (i, pair) in pairs.iter().enumerate() {
        // Copy JPG
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
    }

    Ok(ExportResult {
        exported_count,
        export_folder: target_folder,
    })
}
