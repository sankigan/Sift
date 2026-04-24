// ============================================================
// Sift - Thumbnail Command
// Generates thumbnails + extracts dominant color in parallel
// Optimized: tries EXIF embedded thumbnail first, falls back
// to full decode. Uses ThreadPool(4) to limit concurrency.
// ============================================================

use crate::models::photo::{ThumbnailInput, ThumbnailResult};
use image::GenericImageView;
use rayon::prelude::*;
use std::env;
use std::fs;
use std::path::Path;

#[tauri::command]
pub async fn generate_thumbnails(
    pairs: Vec<ThumbnailInput>,
) -> Result<Vec<ThumbnailResult>, String> {
    tokio::task::spawn_blocking(move || generate_thumbnails_sync(pairs))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

fn generate_thumbnails_sync(
    pairs: Vec<ThumbnailInput>,
) -> Result<Vec<ThumbnailResult>, String> {
    let cache_dir = env::temp_dir().join("sift-thumbnails");
    fs::create_dir_all(&cache_dir).map_err(|e| format!("Failed to create cache dir: {}", e))?;

    // Use rayon's default global thread pool (all cores) for maximum throughput
    let results: Vec<ThumbnailResult> = pairs
        .par_iter()
        .filter_map(|input| process_thumbnail(&cache_dir, input).ok())
        .collect();

    Ok(results)
}

/// Try to extract the EXIF-embedded JPEG thumbnail without full image decode.
/// Reads the EXIF data to find JPEGInterchangeFormat offset/length in IFD1,
/// then extracts the raw bytes from the file.
fn try_extract_exif_thumbnail(jpg_path: &str) -> Option<Vec<u8>> {
    let file_bytes = std::fs::read(jpg_path).ok()?;

    // Find EXIF APP1 marker and parse
    let file = std::io::Cursor::new(&file_bytes);
    let exif_reader = exif::Reader::new();
    let exif_data = exif_reader.read_from_container(&mut std::io::BufReader::new(file)).ok()?;

    // Get thumbnail offset and length from IFD1
    let offset_field = exif_data.get_field(exif::Tag::JPEGInterchangeFormat, exif::In(1))?;
    let length_field = exif_data.get_field(exif::Tag::JPEGInterchangeFormatLength, exif::In(1))?;

    let offset = match &offset_field.value {
        exif::Value::Long(v) if !v.is_empty() => v[0] as usize,
        _ => return None,
    };
    let length = match &length_field.value {
        exif::Value::Long(v) if !v.is_empty() => v[0] as usize,
        _ => return None,
    };

    // Sanity check
    if length < 1024 || length > 1_048_576 {
        return None;
    }

    // The offset from EXIF is relative to the TIFF header start.
    // Find the TIFF header within the JPEG file (after APP1 marker + size).
    // Look for "Exif\0\0" followed by TIFF header ("II" or "MM")
    let exif_marker = b"Exif\x00\x00";
    let tiff_offset = file_bytes
        .windows(exif_marker.len())
        .position(|w| w == exif_marker)?
        + exif_marker.len();

    let abs_offset = tiff_offset + offset;
    let abs_end = abs_offset + length;

    if abs_end > file_bytes.len() {
        return None;
    }

    let thumb_bytes = file_bytes[abs_offset..abs_end].to_vec();

    // Verify it starts with JPEG SOI marker
    if thumb_bytes.len() < 2 || thumb_bytes[0] != 0xFF || thumb_bytes[1] != 0xD8 {
        return None;
    }

    Some(thumb_bytes)
}

/// Read EXIF Orientation (1/2/3/4/5/6/7/8) from a JPEG file.
/// Returns 1 (no-op) on any failure or if the tag is missing.
fn read_jpeg_orientation(path: &str) -> u16 {
    let Ok(file) = std::fs::File::open(path) else {
        return 1;
    };
    let mut reader = std::io::BufReader::new(file);
    let Ok(exif_data) = exif::Reader::new().read_from_container(&mut reader) else {
        return 1;
    };
    exif_data
        .get_field(exif::Tag::Orientation, exif::In::PRIMARY)
        .and_then(|f| match &f.value {
            exif::Value::Short(v) => v.first().copied(),
            _ => None,
        })
        .unwrap_or(1)
}

/// Apply EXIF Orientation to a DynamicImage, producing pixels in the
/// natural/upright orientation so downstream encoders (which discard EXIF)
/// still render correctly.
fn apply_orientation(img: image::DynamicImage, orientation: u16) -> image::DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img, // 1 or unknown: no-op
    }
}

fn process_thumbnail(
    cache_dir: &Path,
    input: &ThumbnailInput,
) -> Result<ThumbnailResult, String> {
    // Cache key v2: previous cache stored sensor-native (unrotated) pixels with
    // no EXIF, which rendered sideways for portrait shots. v2 stores pixels
    // already rotated to the upright orientation.
    let thumb_filename = format!("{}_v2.jpg", input.id);
    let thumb_path = cache_dir.join(&thumb_filename);

    // Check cache — read dominant color from cached thumbnail (not original)
    if thumb_path.exists() {
        let color = extract_dominant_color_from_file(thumb_path.to_str().unwrap_or(""))?;
        return Ok(ThumbnailResult {
            id: input.id.clone(),
            path: thumb_path.to_string_lossy().to_string(),
            dominant_color: color,
        });
    }

    // Read orientation once from the main JPEG; both strategies share it.
    // The `image` crate does not apply EXIF orientation on decode, and our
    // thumbnail encoder writes no EXIF, so we must bake rotation into pixels.
    let orientation = read_jpeg_orientation(&input.jpg_path);

    // Strategy 1: Try EXIF embedded thumbnail (fast path, ~5ms)
    if let Some(thumb_bytes) = try_extract_exif_thumbnail(&input.jpg_path) {
        // Validate it's a real JPEG and decode
        if let Ok(thumb_img) = image::load_from_memory_with_format(&thumb_bytes, image::ImageFormat::Jpeg) {
            // Skip tiny embedded thumbs (typically 160x120). Camera vendors
            // letterbox/pillarbox them to fit a fixed frame, baking black bars
            // into pixels that we can't strip. Anything under 240px on the
            // short side is treated as suspicious and falls through to Strategy 2.
            let (tw, th) = thumb_img.dimensions();
            if tw.min(th) >= 240 {
                let rotated = apply_orientation(thumb_img, orientation);
                let resized = rotated.thumbnail(200, 200);
                resized
                    .save(&thumb_path)
                    .map_err(|e| format!("Failed to save EXIF thumbnail: {}", e))?;

                let color = extract_dominant_color_from_image(&resized)?;
                return Ok(ThumbnailResult {
                    id: input.id.clone(),
                    path: thumb_path.to_string_lossy().to_string(),
                    dominant_color: color,
                });
            }
        }
    }

    // Strategy 2: Full decode fallback
    let img = image::open(&input.jpg_path)
        .map_err(|e| format!("Failed to open image: {}", e))?;

    let rotated = apply_orientation(img, orientation);
    let thumb = rotated.thumbnail(200, 200);
    thumb
        .save(&thumb_path)
        .map_err(|e| format!("Failed to save thumbnail: {}", e))?;

    let color = extract_dominant_color_from_image(&thumb)?;

    Ok(ThumbnailResult {
        id: input.id.clone(),
        path: thumb_path.to_string_lossy().to_string(),
        dominant_color: color,
    })
}

/// Extract dominant color from an existing file (e.g. cached thumbnail)
fn extract_dominant_color_from_file(path: &str) -> Result<String, String> {
    let img = image::open(path).map_err(|e| format!("Failed to open for color: {}", e))?;
    extract_dominant_color_from_image(&img)
}

fn extract_dominant_color_from_image(img: &image::DynamicImage) -> Result<String, String> {
    let small = img.resize_exact(8, 8, image::imageops::FilterType::Lanczos3);

    let mut total_r: u64 = 0;
    let mut total_g: u64 = 0;
    let mut total_b: u64 = 0;
    let mut count: u64 = 0;

    for (_x, _y, pixel) in small.pixels() {
        total_r += pixel[0] as u64;
        total_g += pixel[1] as u64;
        total_b += pixel[2] as u64;
        count += 1;
    }

    if count == 0 {
        return Ok("#3B82F6".to_string());
    }

    let avg_r = (total_r / count) as u8;
    let avg_g = (total_g / count) as u8;
    let avg_b = (total_b / count) as u8;

    Ok(format!("#{:02X}{:02X}{:02X}", avg_r, avg_g, avg_b))
}
