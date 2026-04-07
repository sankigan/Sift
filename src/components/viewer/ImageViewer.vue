<script setup lang="ts">
// ============================================================
// ImageViewer - Core image preview with zoom, pan, ambient glow
// ============================================================

import { ref } from 'vue';
import { useSessionStore } from '@/stores/sessionStore';
import { useViewStore } from '@/stores/viewStore';
import { useImageLoader } from '@/composables/useImageLoader';
import { useImageZoom } from '@/composables/useImageZoom';
import { useAmbientColor } from '@/composables/useAmbientColor';
import SkeletonImage from '@/components/common/SkeletonImage.vue';

const session = useSessionStore();
const view = useViewStore();

const containerRef = ref<HTMLElement | null>(null);
const { isLoading, currentSrc, thumbnailSrc, loadError, naturalWidth, naturalHeight } = useImageLoader();
const { imageStyle, zoomPercent, isZoomed, handleWheel, handleDoubleClick, handleMouseDown } =
  useImageZoom(containerRef, naturalWidth, naturalHeight);
const { ambientStyle } = useAmbientColor();

const showLeftNav = ref(false);
const showRightNav = ref(false);

function handleMouseMoveNav(e: MouseEvent) {
  if (!containerRef.value) return;
  const rect = containerRef.value.getBoundingClientRect();
  const x = e.clientX - rect.left;
  showLeftNav.value = x < 80 && session.currentIndex > 0;
  showRightNav.value = x > rect.width - 80 && !session.isLastPhoto;
}

function handleMouseLeaveNav() {
  showLeftNav.value = false;
  showRightNav.value = false;
}
</script>

<template>
  <div
    ref="containerRef"
    class="relative w-full h-full overflow-hidden bg-sift-bg select-none
           flex items-center justify-center"
    :class="{ 'cursor-grab': isZoomed, 'cursor-default': !isZoomed }"
    @wheel.prevent="handleWheel"
    @dblclick="handleDoubleClick"
    @mousedown="handleMouseDown"
    @mousemove="handleMouseMoveNav"
    @mouseleave="handleMouseLeaveNav"
  >
    <!-- Ambient Background Glow -->
    <div class="ambient-glow" :style="ambientStyle" />

    <!-- Skeleton Loading -->
    <div
      v-if="isLoading && !thumbnailSrc"
      class="absolute inset-0 flex items-center justify-center z-10"
    >
      <SkeletonImage width="70%" height="60%" />
    </div>

    <!-- Thumbnail (shows while full image loads) -->
    <Transition name="page-fade">
      <img
        v-if="isLoading && thumbnailSrc"
        :src="thumbnailSrc"
        class="absolute inset-0 w-full h-full object-contain z-10 blur-sm opacity-60"
        draggable="false"
      />
    </Transition>

    <!-- Main Image -->
    <img
      v-if="!isLoading && currentSrc"
      :key="session.currentIndex"
      :src="currentSrc"
      class="z-20 shrink-0"
      :style="imageStyle"
      draggable="false"
    />

    <!-- Error State -->
    <div
      v-if="loadError"
      class="absolute inset-0 flex flex-col items-center justify-center z-20 text-sift-muted"
    >
      <svg class="w-16 h-16 mb-3 opacity-30" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
          d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
      </svg>
      <p class="text-sm">图片加载失败</p>
    </div>

    <!-- Zoom Level Indicator -->
    <Transition name="nav-fade">
      <div
        v-if="isZoomed"
        class="absolute top-3 left-3 z-30
               px-2.5 py-1 rounded-lg
               bg-black/50 backdrop-blur-sm
               text-[11px] text-white/70 font-mono"
      >
        {{ zoomPercent }}%
      </div>
    </Transition>

    <!-- Left Navigation Arrow -->
    <Transition name="nav-fade">
      <button
        v-if="showLeftNav"
        class="absolute left-3 top-1/2 -translate-y-1/2 z-30
               w-10 h-20 rounded-full
               bg-black/30 backdrop-blur-sm
               flex items-center justify-center
               hover:bg-black/50 transition-colors"
        @click.stop="session.goPrev()"
      >
        <svg class="w-5 h-5 text-white/70" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
      </button>
    </Transition>

    <!-- Right Navigation Arrow -->
    <Transition name="nav-fade">
      <button
        v-if="showRightNav"
        class="absolute right-3 top-1/2 -translate-y-1/2 z-30
               w-10 h-20 rounded-full
               bg-black/30 backdrop-blur-sm
               flex items-center justify-center
               hover:bg-black/50 transition-colors"
        @click.stop="session.goNext()"
      >
        <svg class="w-5 h-5 text-white/70" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
        </svg>
      </button>
    </Transition>

    <!-- Delete Flash Overlay -->
    <div
      v-if="session.currentPair?.status === 'deleted'"
      class="absolute inset-0 bg-sift-delete/20 z-40 pointer-events-none delete-flash"
    />
  </div>
</template>
