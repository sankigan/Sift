// ============================================================
// Sift - RAW Preview Extraction Utility
// Extracts embedded JPEG previews from RAW files
// Strategy: EXIF extraction -> JPEG SOI/EOI scan (fallback)
// ============================================================

use std::fs::{self, File};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Minimum JPEG size to consider as a preview (50KB)
/// This filters out small EXIF thumbnails
const MIN_PREVIEW_SIZE: usize = 50 * 1024;

/// Get the cache directory for RAW previews
fn preview_cache_dir() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("sift-raw-previews");
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create preview cache dir: {}", e))?;
    Ok(dir)
}

/// Generate a cache filename for a RAW file based on its path
/// Uses the file stem + a hash of the full path to avoid collisions
fn cache_filename(raw_path: &Path) -> String {
    let stem = raw_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    let path_str = raw_path.to_string_lossy();
    let hash = simple_hash(path_str.as_bytes());
    format!("{}_{:016x}_preview.jpg", stem, hash)
}

/// Simple FNV-1a hash for path deduplication
fn simple_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Extract an embedded JPEG preview from a RAW file.
/// Returns the path to the cached preview JPEG on success.
///
/// Strategy:
/// 1. Try EXIF-based extraction (fast, ~5ms)
/// 2. Fall back to JPEG SOI/EOI binary scan (~50-200ms)
pub fn extract_raw_preview(raw_path: &str) -> Result<String, String> {
    let path = Path::new(raw_path);
    if !path.exists() {
        return Err(format!("RAW file not found: {}", raw_path));
    }

    let cache_dir = preview_cache_dir()?;
    let cache_name = cache_filename(path);
    let cache_path = cache_dir.join(&cache_name);

    // Check cache first
    if cache_path.exists() {
        return Ok(cache_path.to_string_lossy().to_string());
    }

    // Strategy 1: EXIF-based extraction
    if let Ok(jpeg_data) = extract_via_exif(raw_path) {
        if jpeg_data.len() >= MIN_PREVIEW_SIZE {
            fs::write(&cache_path, &jpeg_data)
                .map_err(|e| format!("Failed to write preview cache: {}", e))?;
            return Ok(cache_path.to_string_lossy().to_string());
        }
    }

    // Strategy 2: JPEG SOI/EOI binary scan (fallback)
    if let Ok(jpeg_data) = extract_via_jpeg_scan(raw_path) {
        if jpeg_data.len() >= MIN_PREVIEW_SIZE {
            fs::write(&cache_path, &jpeg_data)
                .map_err(|e| format!("Failed to write preview cache: {}", e))?;
            return Ok(cache_path.to_string_lossy().to_string());
        }
    }

    Err(format!(
        "Could not extract preview from RAW file: {}",
        raw_path
    ))
}

/// Strategy 1: Extract JPEG preview via EXIF data
/// Works for TIFF-based RAW formats: NEF, CR2, ARW, DNG, ORF, PEF, RW2, etc.
/// Checks both PRIMARY and THUMBNAIL IFDs, returns the largest valid JPEG.
fn extract_via_exif(raw_path: &str) -> Result<Vec<u8>, String> {
    let file =
        File::open(raw_path).map_err(|e| format!("Failed to open RAW for EXIF: {}", e))?;
    let mut reader = BufReader::new(file);

    let exif_reader = exif::Reader::new();
    let exif = exif_reader
        .read_from_container(&mut reader)
        .map_err(|e| format!("Failed to read EXIF from RAW: {}", e))?;

    // Scan IFD0~IFD9: different camera brands store full-size JPEG previews in different IFDs
    // (e.g. Nikon NEF uses IFD#2/3, Canon CR2 uses IFD#3). Extra non-existent IFDs simply return None.
    let ifds: Vec<exif::In> = (0..10).map(|i| exif::In(i)).collect();
    let mut best_jpeg: Option<Vec<u8>> = None;
    let mut best_size: usize = 0;

    for &ifd in &ifds {
        let offset = exif
            .get_field(exif::Tag::JPEGInterchangeFormat, ifd)
            .and_then(|f| match &f.value {
                exif::Value::Long(v) => v.first().map(|&x| x as u64),
                _ => None,
            });

        let length = exif
            .get_field(exif::Tag::JPEGInterchangeFormatLength, ifd)
            .and_then(|f| match &f.value {
                exif::Value::Long(v) => v.first().map(|&x| x as usize),
                _ => None,
            });

        if let (Some(offset), Some(length)) = (offset, length) {
            if length < 100 || length <= best_size {
                continue;
            }

            // Read the JPEG data from the RAW file at the specified offset
            if let Ok(jpeg_data) = read_jpeg_at_offset(raw_path, offset, length) {
                if jpeg_data.len() > best_size {
                    best_size = jpeg_data.len();
                    best_jpeg = Some(jpeg_data);
                }
            }
        }
    }

    best_jpeg.ok_or_else(|| "No JPEG preview found in EXIF".to_string())
}

/// Read and validate JPEG data at a specific offset in a file
fn read_jpeg_at_offset(path: &str, offset: u64, length: usize) -> Result<Vec<u8>, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| format!("Failed to seek to JPEG offset: {}", e))?;

    let mut jpeg_data = vec![0u8; length];
    file.read_exact(&mut jpeg_data)
        .map_err(|e| format!("Failed to read JPEG data: {}", e))?;

    // Verify JPEG signature (FFD8)
    if jpeg_data.len() >= 2 && jpeg_data[0] == 0xFF && jpeg_data[1] == 0xD8 {
        Ok(jpeg_data)
    } else {
        Err("Data at offset is not a valid JPEG".to_string())
    }
}

/// Strategy 2: Binary scan for the largest embedded JPEG
/// Works for non-TIFF RAW formats: CR3, RAF, etc.
/// Scans for JPEG SOI (FF D8) markers followed by valid JPEG markers,
/// then finds the matching EOI (FF D9), and returns the largest valid JPEG block.
fn extract_via_jpeg_scan(raw_path: &str) -> Result<Vec<u8>, String> {
    let data = fs::read(raw_path).map_err(|e| format!("Failed to read RAW file: {}", e))?;

    let mut best_jpeg: Option<Vec<u8>> = None;
    let mut best_size: usize = 0;

    let len = data.len();
    let mut i: usize = 0;

    while i < len.saturating_sub(3) {
        // Look for JPEG SOI marker: FF D8
        if data[i] == 0xFF && data[i + 1] == 0xD8 {
            // Verify this is a real JPEG: the byte after SOI must be FF followed by
            // a valid JPEG marker (APP0=E0, APP1=E1, DQT=DB, SOF0=C0, DHT=C4, etc.)
            let is_valid_jpeg = data[i + 2] == 0xFF && is_valid_jpeg_marker(data[i + 3]);

            if is_valid_jpeg {
                let start = i;
                // Find the corresponding EOI marker: FF D9
                if let Some(end) = find_jpeg_end(&data, start) {
                    let jpeg_len = end - start;
                    if jpeg_len > best_size && jpeg_len >= MIN_PREVIEW_SIZE {
                        best_jpeg = Some(data[start..end].to_vec());
                        best_size = jpeg_len;
                    }
                    // Skip past this JPEG to find potentially larger ones
                    i = end;
                    continue;
                }
            }
        }
        i += 1;
    }

    best_jpeg.ok_or_else(|| "No embedded JPEG found in RAW file".to_string())
}

/// Check if a byte is a valid JPEG marker that can appear right after SOI
fn is_valid_jpeg_marker(marker: u8) -> bool {
    matches!(
        marker,
        0xC0 // SOF0 - Baseline DCT
        | 0xC1 // SOF1 - Extended sequential DCT
        | 0xC2 // SOF2 - Progressive DCT
        | 0xC4 // DHT - Define Huffman Table
        | 0xDB // DQT - Define Quantization Table
        | 0xDD // DRI - Define Restart Interval
        | 0xE0 // APP0 (JFIF)
        | 0xE1 // APP1 (EXIF)
        | 0xE2..=0xEF // APP2-APP15
        | 0xFE // COM - Comment
    )
}

/// Find the end of a JPEG stream starting at `start`.
/// Properly handles JPEG marker segments to find the true EOI (FF D9).
fn find_jpeg_end(data: &[u8], start: usize) -> Option<usize> {
    let len = data.len();
    let mut pos = start + 2; // Skip SOI (FF D8)

    while pos < len.saturating_sub(1) {
        if data[pos] != 0xFF {
            pos += 1;
            continue;
        }

        let marker = data[pos + 1];

        match marker {
            // EOI marker found
            0xD9 => return Some(pos + 2),

            // SOS (Start of Scan) — after this, entropy-coded data begins
            // We need to scan byte-by-byte for the next FF xx (where xx != 0x00)
            0xDA => {
                // Skip the SOS header
                if pos + 3 >= len {
                    return None;
                }
                let seg_len =
                    ((data[pos + 2] as usize) << 8) | (data[pos + 3] as usize);
                pos = pos + 2 + seg_len;

                // Now scan through entropy-coded data
                while pos < len.saturating_sub(1) {
                    if data[pos] == 0xFF {
                        let next = data[pos + 1];
                        if next == 0x00 {
                            // Byte-stuffed FF 00 — skip
                            pos += 2;
                        } else if next == 0xD9 {
                            // EOI found
                            return Some(pos + 2);
                        } else if next >= 0xD0 && next <= 0xD7 {
                            // RST markers — skip
                            pos += 2;
                        } else {
                            // Some other marker — let the outer loop handle it
                            break;
                        }
                    } else {
                        pos += 1;
                    }
                }
            }

            // Standalone markers (no length field): RST0-RST7, SOI, TEM
            0xD0..=0xD7 | 0xD8 | 0x01 => {
                pos += 2;
            }

            // Fill bytes (FF FF)
            0xFF => {
                pos += 1;
            }

            // All other markers have a 2-byte length field
            _ => {
                if pos + 3 >= len {
                    return None;
                }
                let seg_len =
                    ((data[pos + 2] as usize) << 8) | (data[pos + 3] as usize);
                pos = pos + 2 + seg_len;
            }
        }
    }

    None
}
