// ============================================================
// Sift - Tauri Library Entry Point
// Registers all commands and plugins
// ============================================================

mod commands;
mod models;
mod utils;

use commands::{archive, cache, delete, export, file_actions, scan, thumbnail};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Clean up any leftover cache from previous sessions
    cache::cleanup_cache_sync();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                cache::cleanup_cache_sync();
            }
        })
        .invoke_handler(tauri::generate_handler![
            scan::scan_folder,
            delete::delete_pair,
            archive::archive_photos,
            export::export_picks,
            thumbnail::generate_thumbnails,
            file_actions::show_in_folder,
            file_actions::copy_image_to_clipboard,
            cache::cleanup_cache,
            read_exif,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// EXIF reading command (uses utils::exif)
/// For RAW-only photos, reads EXIF from the RAW file instead of the preview JPG
#[tauri::command]
fn read_exif(
    jpg_path: String,
    raw_path: Option<String>,
    source: Option<String>,
) -> Result<models::photo::ExifData, String> {
    // For RAW-only photos, prefer reading EXIF from the RAW original
    if source.as_deref() == Some("rawPreview") {
        if let Some(rp) = &raw_path {
            if let Ok(exif) = utils::exif::read_exif_data(rp) {
                return Ok(exif);
            }
        }
    }
    utils::exif::read_exif_data(&jpg_path)
}
