// ============================================================
// Sift - Keyboard Composable
// Global keyboard shortcuts for culling workflow
// ============================================================

import { onMounted, onUnmounted } from 'vue'
import { useSessionStore } from '@/stores/sessionStore'
import { useViewStore } from '@/stores/viewStore'

export function useKeyboard() {
  const session = useSessionStore()
  const view = useViewStore()

  const isMac = navigator.platform.toUpperCase().includes('MAC')

  function handleKeyDown(e: KeyboardEvent) {
    // Skip if in input/textarea
    const tag = (e.target as HTMLElement)?.tagName
    if (tag === 'INPUT' || tag === 'TEXTAREA') return

    // Only active in culling view
    if (view.currentView !== 'culling') return

    // Don't handle if archive dialog is open
    if (view.showArchiveDialog) return

    // Don't handle if summary card is open
    if (view.showSummaryCard) return

    // ESC closes filter gallery
    if (e.key === 'Escape') {
      if (view.filterCategory) {
        e.preventDefault();
        view.closeFilterGallery();
        return;
      }
    }

    // Don't handle other keys if filter gallery is open
    if (view.filterCategory) return

    const isCtrlOrCmd = isMac ? e.metaKey : e.ctrlKey

    // Cmd/Ctrl + Enter: open archive dialog
    if (isCtrlOrCmd && e.key === 'Enter') {
      const processed = session.starredCount + session.deletedCount + session.skippedCount;
      if (processed > 0) {
        e.preventDefault();
        view.toggleArchiveDialog();
      }
      return;
    }

    switch (e.key) {
      case 'f':
      case 'F':
        e.preventDefault()
        session.markStar()
        break

      case 'x':
      case 'X':
      case 'Delete':
      case 'Backspace':
        e.preventDefault()
        session.markDelete()
        break

      case ' ':
        e.preventDefault()
        session.markSkip()
        break

      case 'ArrowRight':
        e.preventDefault()
        if (!isCtrlOrCmd) {
          session.markSkip()
        }
        break

      case 'ArrowLeft':
      case 'a':
      case 'A':
        if (!isCtrlOrCmd) {
          e.preventDefault()
          session.goPrev()
        }
        break

      case 'z':
      case 'Z':
        if (isCtrlOrCmd) {
          e.preventDefault()
          session.undo()
          view.showToast('已撤销 ↩️', 'undo')
        }
        break

      case 'i':
      case 'I':
        if (!isCtrlOrCmd) {
          e.preventDefault()
          view.toggleExifPanel()
        }
        break

      case '0':
        e.preventDefault()
        view.resetZoom()
        break

      case '1':
        e.preventDefault()
        view.zoomLevel = 1
        view.zoomOffsetX = 0
        view.zoomOffsetY = 0
        break

      case '=':
      case '+':
        e.preventDefault()
        view.zoomLevel = Math.min(view.zoomLevel * 1.25, 10)
        break

      case '-':
        e.preventDefault()
        view.zoomLevel = Math.max(view.zoomLevel / 1.25, 0.5)
        break
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeyDown)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown)
  })

  return { isMac }
}
