// ============================================================
// Sift - View Store (Pinia)
// UI state: current view, panels, zoom
// ============================================================

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { AppView } from '@/types'
import { PhotoStatus } from '@/types'

export const useViewStore = defineStore('view', () => {
  const currentView = ref<AppView>('welcome')
  const showExifPanel = ref(false)
  const showArchiveDialog = ref(false)
  const showSummaryCard = ref(false)

  // Zoom state
  const zoomLevel = ref(1)
  const zoomOffsetX = ref(0)
  const zoomOffsetY = ref(0)

  // Toast
  const toastMessage = ref('')
  const toastType = ref<'star' | 'delete' | 'skip' | 'undo' | 'info'>('info')
  const toastVisible = ref(false)

  // Filter gallery
  const filterCategory = ref<PhotoStatus | null>(null)

  function setView(view: AppView) {
    currentView.value = view
  }

  function toggleExifPanel() {
    showExifPanel.value = !showExifPanel.value
  }

  function toggleArchiveDialog() {
    showArchiveDialog.value = !showArchiveDialog.value
  }

  function resetZoom() {
    zoomLevel.value = 1
    zoomOffsetX.value = 0
    zoomOffsetY.value = 0
  }

  function showToast(message: string, type: 'star' | 'delete' | 'skip' | 'undo' | 'info' = 'info') {
    toastMessage.value = message
    toastType.value = type
    toastVisible.value = true
    setTimeout(() => {
      toastVisible.value = false
    }, 3000)
  }

  function openFilterGallery(category: PhotoStatus) {
    filterCategory.value = category
  }

  function closeFilterGallery() {
    filterCategory.value = null
  }

  return {
    currentView,
    showExifPanel,
    showArchiveDialog,
    showSummaryCard,
    zoomLevel,
    zoomOffsetX,
    zoomOffsetY,
    toastMessage,
    toastType,
    toastVisible,
    filterCategory,
    setView,
    toggleExifPanel,
    toggleArchiveDialog,
    resetZoom,
    showToast,
    openFilterGallery,
    closeFilterGallery,
  }
})
