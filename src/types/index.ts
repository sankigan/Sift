// ============================================================
// Sift - Type Definitions
// ============================================================

/** Photo pair status */
export enum PhotoStatus {
  Unprocessed = 'unprocessed',
  Starred = 'starred',
  Deleted = 'deleted',
  Skipped = 'skipped',
}

/** A paired photo (JPG + optional RAW) */
export interface PhotoPair {
  id: string
  jpgPath: string
  rawPath: string | null
  rawFormat: string | null
  status: PhotoStatus
  thumbnailPath?: string
  /** Hex color string for ambient glow */
  dominantColor?: string
}

/** Result from scanning a folder */
export interface ScanResult {
  pairs: PhotoPair[]
  totalFiles: number
  pairedCount: number
  jpgOnlyCount: number
}

/** Thumbnail generation result */
export interface ThumbnailResult {
  id: string
  path: string
  dominantColor: string
}

/** EXIF metadata */
export interface ExifData {
  camera: string
  lens: string
  iso: number
  aperture: string
  shutterSpeed: string
  focalLength: string
  dateTaken: string
  dimensions: { width: number; height: number }
}

/** Archive operation result */
export interface ArchiveResult {
  movedCount: number
  jpgFolder: string
  rawFolder: string
}

/** Archive progress event payload */
export interface ArchiveProgress {
  current: number
  total: number
  currentFile: string
}

/** Export result */
export interface ExportResult {
  exportedCount: number
  exportFolder: string
}

/** Undo action record */
export interface UndoAction {
  type: 'star' | 'delete' | 'skip'
  index: number
  previousStatus: PhotoStatus
}

/** App page views */
export type AppView = 'welcome' | 'culling' | 'summary'

/** Slide direction for image transitions */
export type SlideDirection = 'left' | 'right'
