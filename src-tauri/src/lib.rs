// ============================================================
// Sift - Tauri Library Entry Point
// Registers all commands and plugins
// ============================================================

mod commands;
mod models;
mod utils;

use commands::{archive, delete, export, scan, thumbnail};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            scan::scan_folder,
            delete::delete_pair,
            archive::archive_photos,
            export::export_picks,
            thumbnail::generate_thumbnails,
            read_exif,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// EXIF reading command (uses utils::exif)
#[tauri::command]
fn read_exif(jpg_path: String) -> Result<models::photo::ExifData, String> {
    utils::exif::read_exif_data(&jpg_path)
}
