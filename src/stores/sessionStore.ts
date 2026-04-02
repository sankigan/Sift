// ============================================================
// Sift - Session Store (Pinia)
// Core state: photo pairs, current index, operations, undo
// ============================================================

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  PhotoStatus,
  type PhotoPair,
  type ScanResult,
  type UndoAction,
  type SlideDirection,
} from '@/types'
import { scanFolder, deletePair, generateThumbnails } from '@/services/tauriCommands'

export const useSessionStore = defineStore('session', () => {
  // ---- State ----
  const folderPath = ref<string>('')
  const pairs = ref<PhotoPair[]>([])
  const currentIndex = ref(0)
  const isScanning = ref(false)
  const scanResult = ref<ScanResult | null>(null)
  const undoStack = ref<UndoAction[]>([])
  const slideDirection = ref<SlideDirection>('left')
  const isGeneratingThumbnails = ref(false)

  // ---- Getters ----
  const currentPair = computed(() => pairs.value[currentIndex.value] || null)

  const totalCount = computed(() => pairs.value.length)

  const starredCount = computed(
    () => pairs.value.filter((p) => p.status === PhotoStatus.Starred).length
  )
  const deletedCount = computed(
    () => pairs.value.filter((p) => p.status === PhotoStatus.Deleted).length
  )
  const skippedCount = computed(
    () => pairs.value.filter((p) => p.status === PhotoStatus.Skipped).length
  )
  const unprocessedCount = computed(
    () =>
      pairs.value.filter((p) => p.status === PhotoStatus.Unprocessed).length
  )

  /** Active (non-deleted) pairs for browsing */
  const activePairs = computed(() =>
    pairs.value.filter((p) => p.status !== PhotoStatus.Deleted)
  )

  /** Progress percentage */
  const progress = computed(() => {
    if (totalCount.value === 0) return 0
    const processed = starredCount.value + deletedCount.value + skippedCount.value
    return Math.round((processed / totalCount.value) * 100)
  })

  const isLastPhoto = computed(
    () => currentIndex.value >= pairs.value.length - 1
  )

  const hasReachedEnd = computed(() => {
    // All photos have been processed
    return unprocessedCount.value === 0
  })

  // ---- Actions ----

  /** Scan a folder and populate pairs */
  async function startScan(path: string) {
    isScanning.value = true
    folderPath.value = path
    try {
      const result = await scanFolder(path)
      scanResult.value = result
      pairs.value = result.pairs
      currentIndex.value = 0
      undoStack.value = []
    } catch (e) {
      throw e
    } finally {
      isScanning.value = false // 扫描完立即停止转圈，不等缩略图
    }

    // 缩略图在后台异步生成，不阻塞 UI
    if (pairs.value.length > 0) {
      isGeneratingThumbnails.value = true
      generateThumbnails(
        pairs.value.map((p) => ({ id: p.id, jpgPath: p.jpgPath }))
      )
        .then((thumbs) => {
          for (const thumb of thumbs) {
            const pair = pairs.value.find((p) => p.id === thumb.id)
            if (pair) {
              pair.thumbnailPath = thumb.path
              pair.dominantColor = thumb.dominantColor
            }
          }
        })
        .catch((e) => {
          console.warn('Thumbnail generation failed:', e)
        })
        .finally(() => {
          isGeneratingThumbnails.value = false
        })
    }
  }

  /** Navigate to next photo */
  function goNext() {
    if (currentIndex.value < pairs.value.length - 1) {
      slideDirection.value = 'left'
      currentIndex.value++
    }
  }

  /** Navigate to previous photo */
  function goPrev() {
    if (currentIndex.value > 0) {
      slideDirection.value = 'right'
      currentIndex.value--
    }
  }

  /** Navigate to specific index */
  function goTo(index: number) {
    if (index >= 0 && index < pairs.value.length) {
      slideDirection.value = index > currentIndex.value ? 'left' : 'right'
      currentIndex.value = index
    }
  }

  /** Mark current photo as starred */
  function markStar() {
    const pair = currentPair.value
    if (!pair) return

    const prevStatus = pair.status
    undoStack.value.push({
      type: 'star',
      index: currentIndex.value,
      previousStatus: prevStatus,
    })

    pair.status = PhotoStatus.Starred
    goNext()
  }

  /** Mark current photo as deleted and move to trash */
  async function markDelete() {
    const pair = currentPair.value
    if (!pair) return

    const prevStatus = pair.status
    undoStack.value.push({
      type: 'delete',
      index: currentIndex.value,
      previousStatus: prevStatus,
    })

    try {
      await deletePair(pair.jpgPath, pair.rawPath)
      pair.status = PhotoStatus.Deleted
      goNext()
    } catch (e) {
      // Remove the undo action if delete failed
      undoStack.value.pop()
      throw e
    }
  }

  /** Skip current photo */
  function markSkip() {
    const pair = currentPair.value
    if (!pair) return

    const prevStatus = pair.status
    undoStack.value.push({
      type: 'skip',
      index: currentIndex.value,
      previousStatus: prevStatus,
    })

    pair.status = PhotoStatus.Skipped
    goNext()
  }

  /** Undo last action */
  function undo() {
    const action = undoStack.value.pop()
    if (!action) return

    const pair = pairs.value[action.index]
    if (pair) {
      pair.status = action.previousStatus
      goTo(action.index)
    }
  }

  /** Reset session */
  function resetSession() {
    folderPath.value = ''
    pairs.value = []
    currentIndex.value = 0
    isScanning.value = false
    scanResult.value = null
    undoStack.value = []
  }

  return {
    // State
    folderPath,
    pairs,
    currentIndex,
    isScanning,
    scanResult,
    undoStack,
    slideDirection,
    isGeneratingThumbnails,
    // Getters
    currentPair,
    totalCount,
    starredCount,
    deletedCount,
    skippedCount,
    unprocessedCount,
    activePairs,
    progress,
    isLastPhoto,
    hasReachedEnd,
    // Actions
    startScan,
    goNext,
    goPrev,
    goTo,
    markStar,
    markDelete,
    markSkip,
    undo,
    resetSession,
  }
})
