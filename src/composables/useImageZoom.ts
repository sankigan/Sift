// ============================================================
// Sift - Image Zoom Composable
// width controls render size (native res), translate for pan
// Container flex centers the image; no manual left/top calc
// ============================================================

import { ref, computed, watch, onMounted, onUnmounted, type Ref } from 'vue';
import { useViewStore } from '@/stores/viewStore';

export function useImageZoom(
  containerRef: Ref<HTMLElement | null>,
  naturalWidth: Ref<number>,
  naturalHeight: Ref<number>,
) {
  const view = useViewStore();

  const isDragging = ref(false);
  const dragStart = ref({ x: 0, y: 0 });
  const offsetStart = ref({ x: 0, y: 0 });

  // Reactive container size via ResizeObserver
  const containerWidth = ref(0);
  const containerHeight = ref(0);
  let resizeObserver: ResizeObserver | null = null;

  function updateContainerSize() {
    const el = containerRef.value;
    if (el) {
      containerWidth.value = el.clientWidth;
      containerHeight.value = el.clientHeight;
    }
  }

  onMounted(() => {
    updateContainerSize();
    resizeObserver = new ResizeObserver(() => updateContainerSize());
    if (containerRef.value) {
      resizeObserver.observe(containerRef.value);
    }
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  });

  onUnmounted(() => {
    resizeObserver?.disconnect();
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  });

  watch(containerRef, (el) => {
    resizeObserver?.disconnect();
    if (el) {
      updateContainerSize();
      resizeObserver?.observe(el);
    }
  });

  /** Base fit width: image scaled to fit container at zoom=1 */
  const fitWidth = computed(() => {
    const cw = containerWidth.value;
    const ch = containerHeight.value;
    const nw = naturalWidth.value;
    const nh = naturalHeight.value;
    if (!cw || !ch || !nw || !nh) return 0;

    const imgRatio = nw / nh;
    const containerRatio = cw / ch;

    if (imgRatio > containerRatio) {
      return cw;
    }
    return ch * imgRatio;
  });

  /** Image style: fit mode at zoom=1, explicit width when zoomed */
  const imageStyle = computed(() => {
    const fw = fitWidth.value;

    // At zoom=1: use max constraints (no reflow, same as object-contain)
    if (view.zoomLevel === 1 || !fw) {
      return {
        maxWidth: '100%',
        maxHeight: '100%',
        width: 'auto',
        transform: 'translate(0px, 0px)',
        willChange: 'auto' as const,
      };
    }

    // Zoomed: explicit width for native-res rendering + translate for pan
    const w = fw * view.zoomLevel;
    return {
      maxWidth: 'none',
      maxHeight: 'none',
      width: `${w}px`,
      transform: `translate(${view.zoomOffsetX * view.zoomLevel}px, ${view.zoomOffsetY * view.zoomLevel}px)`,
      willChange: 'transform' as const,
    };
  });

  /** Zoom percentage for display (relative to fit size) */
  const zoomPercent = computed(() => {
    return Math.round(view.zoomLevel * 100);
  });

  const isZoomed = computed(() => view.zoomLevel !== 1);

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const newZoom = Math.max(0.5, Math.min(10, view.zoomLevel * delta));
    view.zoomLevel = newZoom;
  }

  function handleDoubleClick() {
    if (view.zoomLevel === 1) {
      view.zoomLevel = 2;
    } else {
      view.zoomLevel = 1;
      view.zoomOffsetX = 0;
      view.zoomOffsetY = 0;
    }
  }

  function handleMouseDown(e: MouseEvent) {
    if (view.zoomLevel <= 1) return;
    isDragging.value = true;
    dragStart.value = { x: e.clientX, y: e.clientY };
    offsetStart.value = { x: view.zoomOffsetX, y: view.zoomOffsetY };
    document.body.style.cursor = 'grabbing';
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging.value) return;
    const dx = (e.clientX - dragStart.value.x) / view.zoomLevel;
    const dy = (e.clientY - dragStart.value.y) / view.zoomLevel;
    view.zoomOffsetX = offsetStart.value.x + dx;
    view.zoomOffsetY = offsetStart.value.y + dy;
  }

  function handleMouseUp() {
    isDragging.value = false;
    document.body.style.cursor = '';
  }

  // Reset zoom when switching images
  watch([naturalWidth, naturalHeight], () => {
    view.resetZoom();
  });

  return {
    imageStyle,
    zoomPercent,
    isZoomed,
    isDragging,
    handleWheel,
    handleDoubleClick,
    handleMouseDown,
  };
}
