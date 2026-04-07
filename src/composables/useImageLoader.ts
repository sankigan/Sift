// ============================================================
// Sift - Image Loader Composable
// Progressive loading: skeleton -> thumbnail -> full image
// Preloads ±2 adjacent images
// ============================================================

import { ref, watch, computed } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useSessionStore } from '@/stores/sessionStore'

/** Timeout for loading a single image (ms) */
const LOAD_TIMEOUT = 30_000 // 30s for large RAW JPGs

export function useImageLoader() {
  const session = useSessionStore()

  const isLoading = ref(true)
  const currentSrc = ref('')
  const thumbnailSrc = ref('')
  const loadError = ref(false)

  // Image cache (only store successfully loaded images)
  const imageCache = new Map<string, HTMLImageElement>()
  // Track failed paths to allow retry
  const failedPaths = new Set<string>()

  // Abort controller for cancelling outdated loads
  let loadGeneration = 0

  /** Convert a local file path to a src usable in <img> or Canvas */
  function toSrc(path: string): string {
    return convertFileSrc(path)
  }

  /** Preload an image with timeout */
  function preloadImage(path: string): Promise<HTMLImageElement> {
    // Return cached image if available
    if (imageCache.has(path)) {
      return Promise.resolve(imageCache.get(path)!)
    }

    // Clear from failed set on retry
    failedPaths.delete(path)

    return new Promise((resolve, reject) => {
      const img = new Image()
      let settled = false

      const timer = setTimeout(() => {
        if (!settled) {
          settled = true
          img.src = '' // Cancel the load
          console.warn(`[Sift] Image load timeout (${LOAD_TIMEOUT}ms): ${path}`)
          reject(new Error(`Load timeout: ${path}`))
        }
      }, LOAD_TIMEOUT)

      img.onload = () => {
        if (!settled) {
          settled = true
          clearTimeout(timer)
          imageCache.set(path, img)
          resolve(img)
        }
      }

      img.onerror = (event) => {
        if (!settled) {
          settled = true
          clearTimeout(timer)
          failedPaths.add(path)
          console.error(`[Sift] Image load error: ${path}`, event)
          reject(new Error(`Failed to load: ${path}`))
        }
      }

      img.src = toSrc(path)
    })
  }

  /** Load the current image with progressive display */
  async function loadCurrentImage() {
    const pair = session.currentPair
    if (!pair) {
      currentSrc.value = ''
      return
    }

    // Increment generation to cancel outdated loads
    const thisGeneration = ++loadGeneration

    isLoading.value = true
    loadError.value = false

    // Show thumbnail immediately if available
    if (pair.thumbnailPath) {
      thumbnailSrc.value = toSrc(pair.thumbnailPath)
    } else {
      thumbnailSrc.value = ''
    }

    try {
      // Load full image
      await preloadImage(pair.jpgPath)

      // Only apply if this is still the current load
      if (thisGeneration !== loadGeneration) return

      currentSrc.value = toSrc(pair.jpgPath)
      isLoading.value = false

      // Preload adjacent images
      preloadAdjacent()
    } catch (e) {
      // Only apply error if this is still the current load
      if (thisGeneration !== loadGeneration) return

      console.error('[Sift] loadCurrentImage failed:', e)
      loadError.value = true
      isLoading.value = false
    }
  }

  /** Preload ±2 adjacent images */
  function preloadAdjacent() {
    const idx = session.currentIndex
    const allPairs = session.pairs

    for (let offset = -2; offset <= 2; offset++) {
      if (offset === 0) continue
      const adjIdx = idx + offset
      if (adjIdx >= 0 && adjIdx < allPairs.length) {
        const adjPair = allPairs[adjIdx]
        if (adjPair.jpgPath) {
          preloadImage(adjPair.jpgPath).catch(() => {})
        }
      }
    }
  }

  /** Retry loading the current image (clears cache for this path) */
  function retryLoad() {
    const pair = session.currentPair
    if (pair) {
      // Remove from cache to force reload
      imageCache.delete(pair.jpgPath)
      failedPaths.delete(pair.jpgPath)
      loadCurrentImage()
    }
  }

  // Watch for index changes
  watch(
    () => session.currentIndex,
    () => {
      loadCurrentImage()
    }
  )

  // Initial load (immediate: true ensures first image loads when component mounts with existing data)
  watch(
    () => session.pairs.length,
    (len) => {
      if (len > 0) {
        loadCurrentImage()
      }
    },
    { immediate: true }
  )

  return {
    isLoading,
    currentSrc,
    thumbnailSrc,
    loadError,
    toSrc,
    loadCurrentImage,
    retryLoad,
  }
}
