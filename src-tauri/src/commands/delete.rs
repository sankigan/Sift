// ============================================================
// Sift - Delete Command
// Moves JPG + RAW to system trash (recycle bin)
// Supports RAW-only pairs: deletes RAW to trash + cleans up
// temporary preview file
// Async: uses spawn_blocking to avoid blocking IPC on Windows
// ============================================================

use std::path::Path;

#[tauri::command]
pub async fn delete_pair(
    jpg_path: String,
    raw_path: Option<String>,
    source: Option<String>,
    xmp_paths: Option<Vec<String>>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        delete_pair_sync(&jpg_path, raw_path.as_deref(), source.as_deref(), xmp_paths.as_deref())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

fn delete_pair_sync(
    jpg_path: &str,
    raw_path: Option<&str>,
    source: Option<&str>,
    xmp_paths: Option<&[String]>,
) -> Result<(), String> {
    let is_raw_preview = source == Some("rawPreview");

    if is_raw_preview {
        if let Some(raw) = raw_path {
            let raw_p = Path::new(raw);
            if raw_p.exists() {
                trash::delete(raw).map_err(|e| format!("Failed to trash RAW: {}", e))?;
            }
        }
        let jpg = Path::new(jpg_path);
        if jpg.exists() {
            std::fs::remove_file(jpg_path)
                .map_err(|e| format!("Failed to clean preview file: {}", e))?;
        }
    } else {
        let jpg = Path::new(jpg_path);
        if !jpg.exists() {
            return Err(format!("JPG file not found: {}", jpg_path));
        }

        trash::delete(jpg_path).map_err(|e| format!("Failed to trash JPG: {}", e))?;

        if let Some(raw) = raw_path {
            let raw_p = Path::new(raw);
            if raw_p.exists() {
                trash::delete(raw).map_err(|e| format!("Failed to trash RAW: {}", e))?;
            }
        }
    }

    if let Some(xmps) = xmp_paths {
        for xmp_path in xmps {
            let xmp = Path::new(xmp_path);
            if xmp.exists() {
                trash::delete(xmp_path)
                    .map_err(|e| format!("Failed to trash XMP {}: {}", xmp_path, e))?;
            }
        }
    }

    Ok(())
}
