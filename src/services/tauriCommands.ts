// ============================================================
// Sift - Tauri IPC Command Wrappers
// ============================================================

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type {
  ScanResult,
  ExifData,
  ThumbnailResult,
  ArchiveResult,
  ArchiveProgress,
  ExportResult,
} from '@/types'

/** Scan a folder for JPG+RAW pairs */
export async function scanFolder(folderPath: string): Promise<ScanResult> {
  return invoke<ScanResult>('scan_folder', { folderPath })
}

/** Delete a photo pair (move to system trash) */
export async function deletePair(
  jpgPath: string,
  rawPath: string | null
): Promise<void> {
  return invoke<void>('delete_pair', { jpgPath, rawPath })
}

/** Archive surviving photos into RAW/ and JPG/ subfolders */
export async function archivePhotos(
  folderPath: string,
  pairs: { jpgPath: string; rawPath: string | null; status: string }[]
): Promise<ArchiveResult> {
  return invoke<ArchiveResult>('archive_photos', { folderPath, pairs })
}

/** Export starred photos to a target directory */
export async function exportPicks(
  pairs: { jpgPath: string; rawPath: string | null }[],
  targetFolder: string
): Promise<ExportResult> {
  return invoke<ExportResult>('export_picks', { pairs, targetFolder })
}

/** Generate thumbnails and extract dominant colors */
export async function generateThumbnails(
  pairs: { id: string; jpgPath: string }[]
): Promise<ThumbnailResult[]> {
  return invoke<ThumbnailResult[]>('generate_thumbnails', { pairs })
}

/** Read EXIF metadata from a JPG file */
export async function readExif(jpgPath: string): Promise<ExifData> {
  return invoke<ExifData>('read_exif', { jpgPath })
}

/** Listen for archive progress events */
export function onArchiveProgress(
  callback: (progress: ArchiveProgress) => void
) {
  return listen<ArchiveProgress>('archive-progress', (event) => {
    callback(event.payload)
  })
}

/** Listen for export progress events */
export function onExportProgress(
  callback: (progress: ArchiveProgress) => void
) {
  return listen<ArchiveProgress>('export-progress', (event) => {
    callback(event.payload)
  })
}
