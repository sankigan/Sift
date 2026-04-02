<script setup lang="ts">
// ============================================================
// ImageViewer - Core image preview with zoom, pan, ambient glow
// ============================================================

import { ref, watch, computed, onMounted } from 'vue'
import { useSessionStore } from '@/stores/sessionStore'
import { useViewStore } from '@/stores/viewStore'
import { useImageLoader } from '@/composables/useImageLoader'
import { useImageZoom } from '@/composables/useImageZoom'
import { useAmbientColor } from '@/composables/useAmbientColor'
import SkeletonImage from '@/components/common/SkeletonImage.vue'

const session = useSessionStore()
const view = useViewStore()

const containerRef = ref<HTMLElement | null>(null)
const { isLoading, currentSrc, thumbnailSrc, loadError } = useImageLoader()
const { transform, isZoomed, handleWheel, handleDoubleClick, handleMouseDown } =
  useImageZoom(containerRef)
const { ambientStyle } = useAmbientColor()

const showLeftNav = ref(false)
const showRightNav = ref(false)

function handleMouseMoveNav(e: MouseEvent) {
  if (!containerRef.value) return
  const rect = containerRef.value.getBoundingClientRect()
  const x = e.clientX - rect.left
  showLeftNav.value = x < 80 && session.currentIndex > 0
  showRightNav.value = x > rect.width - 80 && !session.isLastPhoto
}

function handleMouseLeaveNav() {
  showLeftNav.value = false
  showRightNav.value = false
}
</script>

<template>
  <div
    ref="containerRef"
    class="relative w-full h-full overflow-hidden bg-sift-bg select-none"
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
    <Transition :name="'slide-' + session.slideDirection">
      <img
        v-if="!isLoading && currentSrc"
        :key="session.currentIndex"
        :src="currentSrc"
        class="absolute inset-0 w-full h-full object-contain z-20
               transition-transform duration-100"
        :style="{ transform }"
        draggable="false"
      />
    </Transition>

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

    <!-- Left Navigation Arrow -->
    <Transition name="page-fade">
      <button
        v-if="showLeftNav"
        class="absolute left-3 top-1/2 -translate-y-1/2 z-30
               w-10 h-20 rounded-full
               bg-black/30 backdrop-blur-sm
               flex items-center justify-center
               hover:bg-black/50 transition-colors btn-spring"
        @click.stop="session.goPrev()"
      >
        <svg class="w-5 h-5 text-white/70" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
      </button>
    </Transition>

    <!-- Right Navigation Arrow -->
    <Transition name="page-fade">
      <button
        v-if="showRightNav"
        class="absolute right-3 top-1/2 -translate-y-1/2 z-30
               w-10 h-20 rounded-full
               bg-black/30 backdrop-blur-sm
               flex items-center justify-center
               hover:bg-black/50 transition-colors btn-spring"
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
