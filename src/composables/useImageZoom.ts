// ============================================================
// Sift - Image Zoom Composable
// width controls render size (native res), translate for pan
// Container flex centers the image; no manual left/top calc
// ============================================================

import { ref, computed, watch, onMounted, onUnmounted, type Ref } from 'vue';
import { useViewStore } from '@/stores/viewStore';
import { useSessionStore } from '@/stores/sessionStore';

export function useImageZoom(
  containerRef: Ref<HTMLElement | null>,
  naturalWidth: Ref<number>,
  naturalHeight: Ref<number>,
) {
  const view = useViewStore();
  const session = useSessionStore();

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

  /** Base fit width: image scaled to fit container at zoom=1 (rotation-aware, compare-aware) */
  const fitWidth = computed(() => {
    // In compare mode, each image occupies half of the container width
    const cw = view.compareMode ? containerWidth.value / 2 : containerWidth.value;
    const ch = containerHeight.value;
    const nw = naturalWidth.value;
    const nh = naturalHeight.value;
    if (!cw || !ch || !nw || !nh) return 0;

    // When rotated 90°/270°, image's bounding box is [nh x nw]
    const swapped = view.rotation % 180 !== 0;
    const ew = swapped ? nh : nw;
    const eh = swapped ? nw : nh;

    const imgRatio = ew / eh;
    const containerRatio = cw / ch;

    // Effective bounding-box width that fits the container
    const fitW = imgRatio > containerRatio ? cw : ch * imgRatio;

    // Return <img>'s own (unrotated) width:
    // when swapped, the <img>'s width corresponds to bounding-box's height
    return swapped ? fitW * (nw / nh) : fitW;
  });

  /** Image style: CSS max-constraints when unrotated at zoom=1, explicit width otherwise */
  const imageStyle = computed(() => {
    const fw = fitWidth.value;
    const rot = view.rotation;
    const rotated = rot % 180 !== 0;
    const tr = isDragging.value
      ? 'none'
      : 'transform 260ms cubic-bezier(0.22, 1, 0.36, 1)';

    // Unrotated zoom=1, or fitWidth not measured yet: pure CSS max-constraints.
    if ((!rotated && view.zoomLevel === 1) || !fw) {
      return {
        maxWidth: '100%',
        maxHeight: '100%',
        width: 'auto',
        transform: `translate(0px, 0px) rotate(${rot}deg)`,
        transition: tr,
        willChange: 'auto' as const,
      };
    }

    // Rotated or zoomed: explicit width so rotation never overflows & zoom uses native res
    const w = fw * view.zoomLevel;
    return {
      maxWidth: 'none',
      maxHeight: 'none',
      width: `${w}px`,
      transform: `translate(${view.zoomOffsetX * view.zoomLevel}px, ${view.zoomOffsetY * view.zoomLevel}px) rotate(${rot}deg)`,
      transition: tr,
      willChange: view.zoomLevel === 1 ? ('auto' as const) : ('transform' as const),
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

  // Reset zoom/rotation when switching images (index-based to handle same-size photos)
  watch(() => session.currentIndex, () => {
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
