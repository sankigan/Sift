// ============================================================
// Sift - Image Zoom Composable
// Scroll wheel zoom, double-click 100%, drag to pan
// ============================================================

import { ref, computed, onMounted, onUnmounted, type Ref } from 'vue'
import { useViewStore } from '@/stores/viewStore'

export function useImageZoom(containerRef: Ref<HTMLElement | null>) {
  const view = useViewStore()

  const isDragging = ref(false)
  const dragStart = ref({ x: 0, y: 0 })
  const offsetStart = ref({ x: 0, y: 0 })

  const transform = computed(() => {
    return `scale(${view.zoomLevel}) translate(${view.zoomOffsetX}px, ${view.zoomOffsetY}px)`
  })

  const isZoomed = computed(() => view.zoomLevel !== 1)

  function handleWheel(e: WheelEvent) {
    e.preventDefault()
    const delta = e.deltaY > 0 ? 0.9 : 1.1
    const newZoom = Math.max(0.1, Math.min(10, view.zoomLevel * delta))
    view.zoomLevel = newZoom
  }

  function handleDoubleClick() {
    if (view.zoomLevel === 1) {
      // Zoom to 100% (actual size)
      view.zoomLevel = 2
    } else {
      // Reset to fit
      view.zoomLevel = 1
      view.zoomOffsetX = 0
      view.zoomOffsetY = 0
    }
  }

  function handleMouseDown(e: MouseEvent) {
    if (view.zoomLevel <= 1) return
    isDragging.value = true
    dragStart.value = { x: e.clientX, y: e.clientY }
    offsetStart.value = { x: view.zoomOffsetX, y: view.zoomOffsetY }
    document.body.style.cursor = 'grabbing'
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging.value) return
    const dx = (e.clientX - dragStart.value.x) / view.zoomLevel
    const dy = (e.clientY - dragStart.value.y) / view.zoomLevel
    view.zoomOffsetX = offsetStart.value.x + dx
    view.zoomOffsetY = offsetStart.value.y + dy
  }

  function handleMouseUp() {
    isDragging.value = false
    document.body.style.cursor = ''
  }

  onMounted(() => {
    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)
  })

  onUnmounted(() => {
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', handleMouseUp)
  })

  return {
    transform,
    isZoomed,
    isDragging,
    handleWheel,
    handleDoubleClick,
    handleMouseDown,
  }
}
