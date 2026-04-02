// ============================================================
// Sift - Delete Command
// Moves JPG + RAW to system trash (recycle bin)
// ============================================================

use std::path::Path;

#[tauri::command]
pub fn delete_pair(jpg_path: String, raw_path: Option<String>) -> Result<(), String> {
    // Verify JPG exists
    let jpg = Path::new(&jpg_path);
    if !jpg.exists() {
        return Err(format!("JPG file not found: {}", jpg_path));
    }

    // Move JPG to trash
    trash::delete(&jpg_path).map_err(|e| format!("Failed to trash JPG: {}", e))?;

    // Move RAW to trash if it exists
    if let Some(raw) = &raw_path {
        let raw_p = Path::new(raw);
        if raw_p.exists() {
            trash::delete(raw).map_err(|e| format!("Failed to trash RAW: {}", e))?;
        }
    }

    Ok(())
}
