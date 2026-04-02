// ============================================================
// Sift - File Type Utilities
// RAW and JPG extension detection and format mapping
// ============================================================

use std::collections::HashSet;

/// All recognized RAW file extensions (lowercase)
pub fn raw_extensions() -> HashSet<&'static str> {
    let exts: HashSet<&str> = [
        // Canon
        "cr2", "cr3", "crw",
        // Nikon
        "nef", "nrw",
        // Sony
        "arw", "srf", "sr2",
        // Fujifilm
        "raf",
        // Olympus / OM System
        "orf",
        // Panasonic
        "rw2",
        // Pentax
        "pef",
        // Leica
        "rwl",
        // Hasselblad
        "3fr", "fff",
        // Phase One
        "iiq",
        // Samsung
        "srw",
        // Sigma
        "x3f",
        // Kodak
        "kdc", "dcr",
        // Epson
        "erf",
        // Mamiya
        "mef",
        // Leaf
        "mos",
        // Adobe / Universal
        "dng",
    ]
    .iter()
    .copied()
    .collect();
    exts
}

/// JPG/JPEG extensions (lowercase)
pub fn jpg_extensions() -> HashSet<&'static str> {
    ["jpg", "jpeg"].iter().copied().collect()
}

/// Check if a file extension is a known RAW format
pub fn is_raw(ext: &str) -> bool {
    raw_extensions().contains(ext.to_lowercase().as_str())
}

/// Check if a file extension is a JPG
pub fn is_jpg(ext: &str) -> bool {
    jpg_extensions().contains(ext.to_lowercase().as_str())
}

/// Get the RAW format display name from extension
pub fn raw_format_name(ext: &str) -> String {
    ext.to_uppercase()
}
