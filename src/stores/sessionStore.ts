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
  type ThumbnailResult,
  type UndoAction,
  type SlideDirection,
} from '@/types'
import { scanFolder, generateThumbnails, cleanupCache } from '@/services/tauriCommands'

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
  const markActionCount = ref(0)

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

  /** Apply thumbnail results to pairs (Map indexed, O(n)) */
  function applyThumbnails(thumbs: ThumbnailResult[]) {
    console.log(`[Sift] Applying ${thumbs.length} thumbnails`);
    const idToIndex = new Map<string, number>();
    pairs.value.forEach((p, i) => idToIndex.set(p.id, i));
    for (const thumb of thumbs) {
      const idx = idToIndex.get(thumb.id);
      if (idx !== undefined) {
        pairs.value[idx].thumbnailPath = thumb.path;
        pairs.value[idx].dominantColor = thumb.dominantColor;
      }
    }
  }

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

    // 分批生成缩略图（后台异步，不阻塞 startScan 返回）
    // 策略：首批 10 张快速处理（覆盖当前视口），剩余每 50 张一批流式处理
    if (pairs.value.length > 0) {
      isGeneratingThumbnails.value = true;
      const allInputs = pairs.value.map((p) => ({ id: p.id, jpgPath: p.jpgPath }));
      const FIRST_BATCH = 10;
      const CHUNK_SIZE = 50;

      (async () => {
        try {
          // 优先处理首批，快速让视口内缩略图可见
          const firstBatch = allInputs.slice(0, FIRST_BATCH);
          const firstThumbs = await generateThumbnails(firstBatch);
          applyThumbnails(firstThumbs);

          // 剩余分批流式处理，每批完成立即更新 UI
          for (let i = FIRST_BATCH; i < allInputs.length; i += CHUNK_SIZE) {
            const chunk = allInputs.slice(i, i + CHUNK_SIZE);
            const chunkThumbs = await generateThumbnails(chunk);
            applyThumbnails(chunkThumbs);
          }
        } catch (e) {
          console.warn('Thumbnail generation failed:', e);
        } finally {
          isGeneratingThumbnails.value = false;
        }
      })();
    }
  }

  /** Navigate to next photo */
  function goNext() {
    if (currentIndex.value < pairs.value.length - 1) {
      slideDirection.value = 'left';
      currentIndex.value++;
    }
  }

  /** Navigate to the next unprocessed photo (skipping already marked ones) */
  function goNextUnprocessed() {
    const len = pairs.value.length;
    // Search forward from current position
    for (let i = currentIndex.value + 1; i < len; i++) {
      if (pairs.value[i].status === PhotoStatus.Unprocessed) {
        slideDirection.value = 'left';
        currentIndex.value = i;
        return;
      }
    }
    // If nothing found ahead, wrap around and search from the beginning
    for (let i = 0; i < currentIndex.value; i++) {
      if (pairs.value[i].status === PhotoStatus.Unprocessed) {
        slideDirection.value = 'left';
        currentIndex.value = i;
        return;
      }
    }
    // No unprocessed photos left — just go to next sequentially
    goNext();
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

  /** Toggle star on current photo */
  function markStar(): 'starred' | 'unstarred' {
    const idx = currentIndex.value;
    const pair = pairs.value[idx];
    if (!pair) return 'starred';

    // Toggle: if already starred, revert to unprocessed
    if (pair.status === PhotoStatus.Starred) {
      pairs.value[idx] = { ...pair, status: PhotoStatus.Unprocessed };
      goNextUnprocessed();
      return 'unstarred';
    }

    undoStack.value.push({
      type: 'star',
      index: idx,
      previousStatus: pair.status,
    });

    pairs.value[idx] = { ...pair, status: PhotoStatus.Starred };
    markActionCount.value++;
    goNextUnprocessed();
    return 'starred';
  }

  /** Toggle delete on current photo (only marks status, actual deletion happens during archive) */
  function markDelete(): 'deleted' | 'undeleted' {
    const idx = currentIndex.value;
    const pair = pairs.value[idx];
    if (!pair) return 'deleted';

    // Toggle: if already deleted, revert to unprocessed
    if (pair.status === PhotoStatus.Deleted) {
      pairs.value[idx] = { ...pair, status: PhotoStatus.Unprocessed };
      goNextUnprocessed();
      return 'undeleted';
    }

    undoStack.value.push({
      type: 'delete',
      index: idx,
      previousStatus: pair.status,
    });

    pairs.value[idx] = { ...pair, status: PhotoStatus.Deleted };
    markActionCount.value++;
    goNextUnprocessed();
    return 'deleted';
  }

  /** Toggle skip on current photo */
  function markSkip(): 'skipped' | 'unskipped' {
    const idx = currentIndex.value;
    const pair = pairs.value[idx];
    if (!pair) return 'skipped';

    // Toggle: if already skipped, revert to unprocessed
    if (pair.status === PhotoStatus.Skipped) {
      pairs.value[idx] = { ...pair, status: PhotoStatus.Unprocessed };
      goNextUnprocessed();
      return 'unskipped';
    }

    undoStack.value.push({
      type: 'skip',
      index: idx,
      previousStatus: pair.status,
    });

    pairs.value[idx] = { ...pair, status: PhotoStatus.Skipped };
    markActionCount.value++;
    goNextUnprocessed();
    return 'skipped';
  }

  /** Undo last action */
  function undo() {
    const action = undoStack.value.pop();
    if (!action) return;

    const pair = pairs.value[action.index];
    if (pair) {
      pairs.value[action.index] = { ...pair, status: action.previousStatus };
      goTo(action.index);
    }
  }

  /** Reset session */
  function resetSession() {
    // Clean up cache directories
    cleanupCache().catch((e) => console.warn('Cache cleanup failed:', e));

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
    markActionCount,
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
