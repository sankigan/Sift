// ============================================================
// Sift - File Action Commands
// Provides "Show in Finder" and "Copy Image to Clipboard"
// ============================================================

use std::path::Path;

/// Open the system file manager and highlight the given file.
/// macOS: `open -R <path>`
#[tauri::command]
pub async fn show_in_folder(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("File not found: {}", path));
    }

    tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg("-R")
                .arg(&path)
                .status()
                .map_err(|e| format!("Failed to open Finder: {}", e))?;
        }

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(format!("/select,\"{}\"", &path))
                .status()
                .map_err(|e| format!("Failed to open Explorer: {}", e))?;
        }

        #[cfg(target_os = "linux")]
        {
            // Try xdg-open on the parent directory
            if let Some(parent) = Path::new(&path).parent() {
                std::process::Command::new("xdg-open")
                    .arg(parent)
                    .status()
                    .map_err(|e| format!("Failed to open file manager: {}", e))?;
            }
        }

        Ok(())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Copy an image file to the system clipboard.
/// macOS: uses osascript with NSPasteboard via AppleScript.
/// Windows: uses PowerShell with System.Windows.Forms.Clipboard.
#[tauri::command]
pub async fn copy_image_to_clipboard(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("File not found: {}", path));
    }

    tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "macos")]
        {
            let ext = Path::new(&path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let clipboard_type = match ext.as_str() {
                "png" => "«class PNGf»",
                "jpg" | "jpeg" | "jpe" => "«class JPEG»",
                "tiff" | "tif" => "«class TIFF»",
                _ => "«class JPEG»",
            };

            let script = format!(
                "set the clipboard to (read (POSIX file \"{}\") as {})",
                path, clipboard_type
            );

            let output = std::process::Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .output()
                .map_err(|e| format!("Failed to run osascript: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("osascript failed: {}", stderr));
            }
        }

        #[cfg(target_os = "windows")]
        {
            use base64::{Engine as _, engine::general_purpose::STANDARD};

            let ps_script = format!(
                "Add-Type -AssemblyName System.Windows.Forms; \
                 [System.Windows.Forms.Clipboard]::SetImage(\
                 [System.Drawing.Image]::FromFile('{}'))",
                path.replace("'", "''")
            );

            // Encode as UTF-16LE Base64 to avoid shell escaping issues
            // with special chars like backticks, $, etc. in file paths
            let utf16le: Vec<u8> = ps_script.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
            let encoded = STANDARD.encode(&utf16le);

            let output = std::process::Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-EncodedCommand", &encoded])
                .output()
                .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("PowerShell clipboard failed: {}", stderr));
            }
        }

        #[cfg(target_os = "linux")]
        {
            let output = std::process::Command::new("xclip")
                .args(["-selection", "clipboard", "-t", "image/png", "-i", &path])
                .output()
                .map_err(|e| format!("Failed to run xclip: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("xclip failed: {}", stderr));
            }
        }

        Ok(())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
