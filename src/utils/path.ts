// ============================================================
// Sift - Path Utilities
// Cross-platform path handling for macOS (/) and Windows (\)
// ============================================================

/**
 * Extract the filename from a file path, handling both / and \ separators.
 */
export function extractFileName(filePath: string): string {
  const parts = filePath.split(/[\\/]/);
  return parts[parts.length - 1] || '';
}
