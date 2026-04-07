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

    const isCtrlOrCmd = isMac ? e.metaKey : e.ctrlKey

    switch (e.key) {
      case 'f':
      case 'F':
        e.preventDefault()
        session.markStar()
        view.showToast('已标记 ⭐', 'star')
        break

      case 'x':
      case 'X':
      case 'Delete':
      case 'Backspace':
        e.preventDefault()
        session.markDelete().then(() => {
          view.showToast('已删除 🗑️', 'delete')
        })
        break

      case ' ':
        e.preventDefault()
        session.markSkip()
        view.showToast('已跳过 ⏭️', 'skip')
        break

      case 'ArrowRight':
        e.preventDefault()
        if (!isCtrlOrCmd) {
          session.markSkip()
          view.showToast('已跳过 ⏭️', 'skip')
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
