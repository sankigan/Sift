// ============================================================
// Sift - Image Loader Composable
// Progressive loading: skeleton -> thumbnail -> full image
// Preloads ±2 adjacent images
// ============================================================

import { ref, watch, computed } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useSessionStore } from '@/stores/sessionStore'

export function useImageLoader() {
  const session = useSessionStore()

  const isLoading = ref(true)
  const currentSrc = ref('')
  const thumbnailSrc = ref('')
  const loadError = ref(false)

  // Image cache
  const imageCache = new Map<string, HTMLImageElement>()

  /** Convert a local file path to a src usable in <img> or Canvas */
  function toSrc(path: string): string {
    return convertFileSrc(path)
  }

  /** Preload an image and cache it */
  function preloadImage(path: string): Promise<HTMLImageElement> {
    if (imageCache.has(path)) {
      return Promise.resolve(imageCache.get(path)!)
    }
    return new Promise((resolve, reject) => {
      const img = new Image()
      img.onload = () => {
        imageCache.set(path, img)
        resolve(img)
      }
      img.onerror = reject
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
      const img = await preloadImage(pair.jpgPath)
      currentSrc.value = toSrc(pair.jpgPath)
      isLoading.value = false

      // Preload adjacent images
      preloadAdjacent()
    } catch (e) {
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

  // Watch for index changes
  watch(
    () => session.currentIndex,
    () => {
      loadCurrentImage()
    }
  )

  // Initial load
  watch(
    () => session.pairs.length,
    (len) => {
      if (len > 0) {
        loadCurrentImage()
      }
    }
  )

  return {
    isLoading,
    currentSrc,
    thumbnailSrc,
    loadError,
    toSrc,
    loadCurrentImage,
  }
}
