// ============================================================
// Sift - File Type Utilities
// RAW and JPG extension detection and format mapping
// Uses once_cell for static caching of extension sets
// ============================================================

use once_cell::sync::Lazy;
use std::collections::HashSet;

/// All recognized RAW file extensions (lowercase), cached statically
static RAW_EXTENSIONS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
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
    .collect()
});

/// JPG/JPEG extensions (lowercase), cached statically
static JPG_EXTENSIONS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["jpg", "jpeg"].iter().copied().collect()
});

/// Check if a file extension is a known RAW format
pub fn is_raw(ext: &str) -> bool {
    RAW_EXTENSIONS.contains(ext.to_lowercase().as_str())
}

/// Check if a file extension is a JPG
pub fn is_jpg(ext: &str) -> bool {
    JPG_EXTENSIONS.contains(ext.to_lowercase().as_str())
}

/// Get the RAW format display name from extension
pub fn raw_format_name(ext: &str) -> String {
    ext.to_uppercase()
}
