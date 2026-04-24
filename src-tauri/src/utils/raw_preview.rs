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
///
/// Orientation handling: some RAW formats (notably Sony ARW) store the
/// embedded preview JPEG with its pixels in sensor-native orientation
/// (always landscape) and keep the "this should be rotated" hint only in
/// the outer TIFF IFD0. To make the preview display correctly in a plain
/// `<img>`, we sync that Orientation into the preview JPEG's own APP1/EXIF
/// before caching.
pub fn extract_raw_preview(raw_path: &str) -> Result<String, String> {
    let path = Path::new(raw_path);
    if !path.exists() {
        return Err(format!("RAW file not found: {}", raw_path));
    }

    let cache_dir = preview_cache_dir()?;
    let cache_name = cache_filename(path);
    let cache_path = cache_dir.join(&cache_name);

    // Orientation stored in the outer RAW's TIFF IFD0 (1 = normal, >1 = needs rotate/flip).
    // Failing to read it is non-fatal; we just treat it as "unknown / normal".
    let outer_orientation = read_raw_orientation(raw_path).unwrap_or(1);

    // Check cache — but invalidate if the cached JPEG doesn't carry the
    // outer orientation (common on Sony ARW previews produced before this fix).
    if cache_path.exists() {
        if outer_orientation <= 1 {
            return Ok(cache_path.to_string_lossy().to_string());
        }
        if let Ok(existing) = fs::read(&cache_path) {
            let existing_orient = jpeg_orientation(&existing).unwrap_or(1);
            if existing_orient == outer_orientation {
                return Ok(cache_path.to_string_lossy().to_string());
            }
            // Cache is stale (preview lacks the correct orientation) — fall through and re-extract.
        }
    }

    // Strategy 1: EXIF-based extraction
    if let Ok(jpeg_data) = extract_via_exif(raw_path) {
        if jpeg_data.len() >= MIN_PREVIEW_SIZE {
            let final_data = ensure_jpeg_orientation(jpeg_data, outer_orientation);
            fs::write(&cache_path, &final_data)
                .map_err(|e| format!("Failed to write preview cache: {}", e))?;
            return Ok(cache_path.to_string_lossy().to_string());
        }
    }

    // Strategy 2: JPEG SOI/EOI binary scan (fallback)
    if let Ok(jpeg_data) = extract_via_jpeg_scan(raw_path) {
        if jpeg_data.len() >= MIN_PREVIEW_SIZE {
            let final_data = ensure_jpeg_orientation(jpeg_data, outer_orientation);
            fs::write(&cache_path, &final_data)
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

// ============================================================
// EXIF Orientation helpers
// ============================================================

/// Read the `Orientation` tag from the outer RAW's primary IFD (IFD0).
/// Returns None if the RAW isn't EXIF-readable or the tag is absent.
/// Value is the standard EXIF Orientation: 1=normal, 3=180°, 6=CW 90°, 8=CCW 90°, etc.
fn read_raw_orientation(raw_path: &str) -> Option<u16> {
    let file = File::open(raw_path).ok()?;
    let mut reader = BufReader::new(file);
    let exif = exif::Reader::new().read_from_container(&mut reader).ok()?;

    let field = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?;
    match &field.value {
        exif::Value::Short(v) => v.first().copied(),
        _ => None,
    }
}

/// Get the Orientation tag already present in a JPEG's APP1/EXIF segment, if any.
fn jpeg_orientation(jpeg: &[u8]) -> Option<u16> {
    let (start, len) = find_jpeg_app1_tiff_range(jpeg)?;
    read_tiff_orientation(&jpeg[start..start + len])
}

/// Ensure the JPEG carries the given outer_orientation in its APP1/EXIF.
///
/// Behaviour:
///   - outer == 1 (or unknown): return the JPEG unchanged.
///   - JPEG already has an APP1/EXIF with a writable Orientation tag: patch it in place.
///   - JPEG has no APP1/EXIF (or APP1 without Orientation tag): inject a minimal
///     APP1 segment containing just the Orientation tag right after SOI.
fn ensure_jpeg_orientation(jpeg: Vec<u8>, outer_orientation: u16) -> Vec<u8> {
    if outer_orientation <= 1 {
        return jpeg;
    }

    // Try to patch an existing Orientation tag in-place.
    if let Some((tiff_start, tiff_len)) = find_jpeg_app1_tiff_range(&jpeg) {
        let mut out = jpeg;
        let tiff = &mut out[tiff_start..tiff_start + tiff_len];
        if patch_tiff_orientation(tiff, outer_orientation) {
            return out;
        }
        // APP1 exists but no Orientation tag — prepend a fresh APP1; readers pick the first.
        return inject_orientation_app1(out, outer_orientation);
    }

    // No APP1/EXIF at all: inject a minimal one.
    inject_orientation_app1(jpeg, outer_orientation)
}

/// Locate the TIFF payload inside a JPEG's first APP1/EXIF segment.
/// Returns (offset_of_tiff_in_jpeg, tiff_len).
fn find_jpeg_app1_tiff_range(jpeg: &[u8]) -> Option<(usize, usize)> {
    // JPEG must start with SOI (FF D8)
    if jpeg.len() < 4 || jpeg[0] != 0xFF || jpeg[1] != 0xD8 {
        return None;
    }

    let mut pos = 2;
    while pos + 4 <= jpeg.len() {
        if jpeg[pos] != 0xFF {
            return None;
        }
        let marker = jpeg[pos + 1];

        // SOS / EOI => past the header area, no APP1 found.
        if marker == 0xDA || marker == 0xD9 {
            return None;
        }

        // Standalone markers (no length)
        if matches!(marker, 0xD0..=0xD7 | 0xD8 | 0x01) {
            pos += 2;
            continue;
        }

        // Fill byte (FF FF)
        if marker == 0xFF {
            pos += 1;
            continue;
        }

        // Length field (big-endian, includes the 2 length bytes themselves)
        let seg_len = ((jpeg[pos + 2] as usize) << 8) | (jpeg[pos + 3] as usize);
        if seg_len < 2 || pos + 2 + seg_len > jpeg.len() {
            return None;
        }

        if marker == 0xE1 {
            // APP1: check for "Exif\0\0" signature
            let payload_start = pos + 4;
            let payload_end = pos + 2 + seg_len;
            let payload = &jpeg[payload_start..payload_end];
            if payload.len() >= 6 && &payload[0..6] == b"Exif\0\0" {
                let tiff_start = payload_start + 6;
                let tiff_len = payload_end - tiff_start;
                if tiff_len >= 8 {
                    return Some((tiff_start, tiff_len));
                }
            }
        }

        pos += 2 + seg_len;
    }
    None
}

/// Read Orientation (tag 0x0112) from a TIFF blob. Returns None if absent/malformed.
fn read_tiff_orientation(tiff: &[u8]) -> Option<u16> {
    let (le, ifd0_off) = parse_tiff_header(tiff)?;
    read_ifd_orientation(tiff, ifd0_off, le)
}

/// Patch an existing Orientation tag's value in a TIFF blob. Returns true if patched.
fn patch_tiff_orientation(tiff: &mut [u8], new_value: u16) -> bool {
    let (le, ifd0_off) = match parse_tiff_header(tiff) {
        Some(x) => x,
        None => return false,
    };
    if ifd0_off + 2 > tiff.len() {
        return false;
    }
    let count = read_u16(&tiff[ifd0_off..ifd0_off + 2], le) as usize;
    for i in 0..count {
        let entry_off = ifd0_off + 2 + i * 12;
        if entry_off + 12 > tiff.len() {
            return false;
        }
        let tag = read_u16(&tiff[entry_off..entry_off + 2], le);
        if tag == 0x0112 {
            // Orientation is SHORT, count 1 — the value sits in the first 2 bytes of the value/offset field.
            let val_off = entry_off + 8;
            write_u16(&mut tiff[val_off..val_off + 2], new_value, le);
            return true;
        }
    }
    false
}

fn read_ifd_orientation(tiff: &[u8], ifd_off: usize, le: bool) -> Option<u16> {
    if ifd_off + 2 > tiff.len() {
        return None;
    }
    let count = read_u16(&tiff[ifd_off..ifd_off + 2], le) as usize;
    for i in 0..count {
        let entry_off = ifd_off + 2 + i * 12;
        if entry_off + 12 > tiff.len() {
            return None;
        }
        let tag = read_u16(&tiff[entry_off..entry_off + 2], le);
        if tag == 0x0112 {
            return Some(read_u16(&tiff[entry_off + 8..entry_off + 10], le));
        }
    }
    None
}

/// Parse TIFF header. Returns (little_endian, ifd0_offset).
fn parse_tiff_header(tiff: &[u8]) -> Option<(bool, usize)> {
    if tiff.len() < 8 {
        return None;
    }
    let le = match &tiff[0..2] {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let magic = read_u16(&tiff[2..4], le);
    if magic != 0x002A {
        return None;
    }
    let ifd0 = read_u32(&tiff[4..8], le) as usize;
    Some((le, ifd0))
}

fn read_u16(b: &[u8], le: bool) -> u16 {
    if le {
        u16::from_le_bytes([b[0], b[1]])
    } else {
        u16::from_be_bytes([b[0], b[1]])
    }
}

fn read_u32(b: &[u8], le: bool) -> u32 {
    if le {
        u32::from_le_bytes([b[0], b[1], b[2], b[3]])
    } else {
        u32::from_be_bytes([b[0], b[1], b[2], b[3]])
    }
}

fn write_u16(b: &mut [u8], v: u16, le: bool) {
    let bytes = if le { v.to_le_bytes() } else { v.to_be_bytes() };
    b[0] = bytes[0];
    b[1] = bytes[1];
}

/// Inject a minimal APP1/EXIF segment (carrying just Orientation) right after SOI.
fn inject_orientation_app1(jpeg: Vec<u8>, orientation: u16) -> Vec<u8> {
    // Build a minimal TIFF:
    //   "II" 2A 00  08 00 00 00           TIFF header, IFD0 @ offset 8 (little-endian)
    //   01 00                             entry count = 1
    //   12 01 03 00 01 00 00 00 <O> 00 00 00   Orientation entry (tag=0x0112, type=SHORT, count=1)
    //   00 00 00 00                       next IFD offset = 0
    let o = orientation.to_le_bytes();
    let tiff: [u8; 26] = [
        b'I', b'I', 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00,
        0x01, 0x00,
        0x12, 0x01, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, o[0], o[1], 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];

    // APP1 payload = "Exif\0\0" + tiff
    // APP1 length field = 2 (length bytes) + 6 ("Exif\0\0") + tiff.len()
    let app1_len = 2 + 6 + tiff.len();
    let len_be = (app1_len as u16).to_be_bytes();

    let mut out = Vec::with_capacity(jpeg.len() + 4 + app1_len);
    // SOI
    out.extend_from_slice(&jpeg[0..2]);
    // APP1 marker + length
    out.push(0xFF);
    out.push(0xE1);
    out.push(len_be[0]);
    out.push(len_be[1]);
    // "Exif\0\0"
    out.extend_from_slice(b"Exif\0\0");
    // TIFF
    out.extend_from_slice(&tiff);
    // Rest of the JPEG (everything after SOI)
    out.extend_from_slice(&jpeg[2..]);

    out
}
