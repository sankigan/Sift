// ============================================================
// Sift - Cache Cleanup Command
// Removes temporary thumbnail and RAW preview cache directories
// ============================================================

use std::env;
use std::fs;

#[tauri::command]
pub async fn cleanup_cache() -> Result<(), String> {
    tokio::task::spawn_blocking(|| {
        let temp = env::temp_dir();
        let dirs = ["sift-thumbnails", "sift-raw-previews"];

        for dir_name in &dirs {
            let dir = temp.join(dir_name);
            if dir.exists() {
                if let Err(e) = fs::remove_dir_all(&dir) {
                    eprintln!("[Sift] Failed to clean cache {}: {}", dir_name, e);
                }
            }
        }

        Ok(())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
