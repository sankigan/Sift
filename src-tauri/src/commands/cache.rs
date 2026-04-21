// ============================================================
// Sift - Cache Cleanup Command
// Removes temporary thumbnail and RAW preview cache directories
// ============================================================

use std::env;
use std::fs;

const CACHE_DIRS: [&str; 2] = ["sift-thumbnails", "sift-raw-previews"];

/// Synchronous cache cleanup — safe to call from startup and window close events
pub fn cleanup_cache_sync() {
    let temp = env::temp_dir();
    for dir_name in &CACHE_DIRS {
        let dir = temp.join(dir_name);
        if dir.exists() {
            if let Err(e) = fs::remove_dir_all(&dir) {
                eprintln!("[Sift] Failed to clean cache {}: {}", dir_name, e);
            }
        }
    }
}

#[tauri::command]
pub async fn cleanup_cache() -> Result<(), String> {
    tokio::task::spawn_blocking(cleanup_cache_sync)
        .await
        .map_err(|e| format!("Task join error: {}", e))
}
