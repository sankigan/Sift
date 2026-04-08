// ============================================================
// Sift - Thumbnail Command
// Generates thumbnails + extracts dominant color in parallel
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
    // Run the heavy image processing in a background thread
    tokio::task::spawn_blocking(move || generate_thumbnails_sync(pairs))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

fn generate_thumbnails_sync(
    pairs: Vec<ThumbnailInput>,
) -> Result<Vec<ThumbnailResult>, String> {
    let cache_dir = env::temp_dir().join("sift-thumbnails");
    fs::create_dir_all(&cache_dir).map_err(|e| format!("Failed to create cache dir: {}", e))?;

    let results: Vec<ThumbnailResult> = pairs
        .par_iter()
        .filter_map(|input| process_thumbnail(&cache_dir, input).ok())
        .collect();

    Ok(results)
}

fn process_thumbnail(
    cache_dir: &Path,
    input: &ThumbnailInput,
) -> Result<ThumbnailResult, String> {
    let thumb_filename = format!("{}.jpg", input.id);
    let thumb_path = cache_dir.join(&thumb_filename);

    // Check cache
    if thumb_path.exists() {
        let color = extract_dominant_color(&input.jpg_path)?;
        return Ok(ThumbnailResult {
            id: input.id.clone(),
            path: thumb_path.to_string_lossy().to_string(),
            dominant_color: color,
        });
    }

    // Open image
    let img = image::open(&input.jpg_path)
        .map_err(|e| format!("Failed to open image: {}", e))?;

    // Generate 200px wide thumbnail (sufficient for 64px display)
    let thumb = img.thumbnail(200, 200);
    thumb
        .save(&thumb_path)
        .map_err(|e| format!("Failed to save thumbnail: {}", e))?;

    // Extract dominant color from 8x8 resized image
    let color = extract_dominant_color_from_image(&img)?;

    Ok(ThumbnailResult {
        id: input.id.clone(),
        path: thumb_path.to_string_lossy().to_string(),
        dominant_color: color,
    })
}

fn extract_dominant_color(jpg_path: &str) -> Result<String, String> {
    let img = image::open(jpg_path).map_err(|e| format!("Failed to open for color: {}", e))?;
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
