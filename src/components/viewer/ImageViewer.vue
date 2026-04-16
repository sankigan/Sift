<script setup lang="ts">
// ============================================================
// ImageViewer - Core image preview with zoom, pan, ambient glow
// Supports compare mode: side-by-side dual image layout
// ============================================================

import { ref, computed, watch } from 'vue';
import { useSessionStore } from '@/stores/sessionStore';
import { useViewStore } from '@/stores/viewStore';
import { useImageLoader } from '@/composables/useImageLoader';
import { useImageZoom } from '@/composables/useImageZoom';
import { useAmbientColor } from '@/composables/useAmbientColor';
import { convertFileSrc } from '@tauri-apps/api/core';
import { ArrowLeftRight, Columns2 } from 'lucide-vue-next';
import SkeletonImage from '@/components/common/SkeletonImage.vue';
import ContextMenu from '@/components/common/ContextMenu.vue';
import { useContextMenu } from '@/composables/useContextMenu';
import { extractFileName } from '@/utils/path';

const session = useSessionStore();
const view = useViewStore();
const contextMenu = useContextMenu();

const containerRef = ref<HTMLElement | null>(null);
const { isLoading, currentSrc, thumbnailSrc, loadError, naturalWidth, naturalHeight } = useImageLoader();
const { imageStyle, zoomPercent, isZoomed, handleWheel, handleDoubleClick, handleMouseDown } =
  useImageZoom(containerRef, naturalWidth, naturalHeight);
const { ambientStyle } = useAmbientColor();

const showLeftNav = ref(false);
const showRightNav = ref(false);

// Compare mode: compute the base image src from compareIndex
const compareFullSrc = computed(() => {
  if (!view.compareMode || view.compareIndex === null) return '';
  const pair = session.pairs[view.compareIndex];
  if (!pair) return '';
  return convertFileSrc(pair.jpgPath);
});

const compareFileName = computed(() => {
  if (view.compareIndex === null) return '';
  const pair = session.pairs[view.compareIndex];
  if (!pair) return '';
  if (pair.source === 'rawPreview' && pair.rawPath) {
    return extractFileName(pair.rawPath);
  }
  return extractFileName(pair.jpgPath);
});

const currentFileName = computed(() => {
  const pair = session.currentPair;
  if (!pair) return '';
  if (pair.source === 'rawPreview' && pair.rawPath) {
    return extractFileName(pair.rawPath);
  }
  return extractFileName(pair.jpgPath);
});

// Watch: if compareIndex pair gets deleted, exit compare
watch(
  () => {
    if (view.compareMode && view.compareIndex !== null) {
      return session.pairs[view.compareIndex]?.status;
    }
    return null;
  },
  (status) => {
    if (status === 'deleted' && view.compareMode) {
      view.exitCompare();
      view.showToast('基准图已被标记删除，已退出对比模式', 'info');
    }
  }
);

function handleSwapCompare() {
  view.swapCompare(session.currentIndex);
}

function handleCompare() {
  view.toggleCompare(session.currentIndex);
}

function handleMouseMoveNav(e: MouseEvent) {
  if (view.compareMode) return;
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

function handleContextMenu(e: MouseEvent, pairOverride?: typeof session.currentPair) {
  const pair = pairOverride || session.currentPair;
  if (!pair) return;
  contextMenu.show(e, pair);
}
</script>

<template>
  <div
    ref="containerRef"
    class="relative w-full h-full overflow-hidden bg-sift-bg select-none"
    :class="{
      'cursor-grab': isZoomed,
      'cursor-default': !isZoomed,
    }"
    @wheel.prevent="handleWheel"
    @dblclick="handleDoubleClick"
    @mousedown="handleMouseDown"
    @mousemove="handleMouseMoveNav"
    @mouseleave="handleMouseLeaveNav"
  >
    <!-- ==================== COMPARE MODE ==================== -->
    <template v-if="view.compareMode">
      <div class="absolute inset-0 flex">
        <!-- Left: Base Image (compareIndex) -->
        <div class="relative w-1/2 h-full flex items-center justify-center overflow-hidden
                    border-r border-white/10"
             @contextmenu.prevent="view.compareIndex !== null && session.pairs[view.compareIndex] ? handleContextMenu($event, session.pairs[view.compareIndex]) : undefined"
        >
          <!-- Top bar: label + filename -->
          <div class="absolute top-3 left-3 right-3 z-30 flex items-center justify-between gap-2">
            <div class="flex items-center gap-1.5
                        px-2.5 py-1 rounded-lg
                        bg-black/50 backdrop-blur-sm shrink-0">
              <div class="w-1.5 h-1.5 rounded-full bg-blue-400" />
              <span class="text-[11px] text-white/70">基准图</span>
            </div>
            <div class="px-2 py-1 rounded-lg
                        bg-black/50 backdrop-blur-sm
                        text-[10px] text-white/50 font-mono truncate">
              {{ compareFileName }}
            </div>
          </div>
          <img
            v-if="compareFullSrc"
            :src="compareFullSrc"
            class="shrink-0"
            :style="imageStyle"
            draggable="false"
          />
        </div>

        <!-- Right: Current Image (currentIndex) -->
        <div class="relative w-1/2 h-full flex items-center justify-center overflow-hidden"
             @contextmenu.prevent="handleContextMenu($event)"
        >
          <!-- Top bar: label + filename -->
          <div class="absolute top-3 left-3 right-3 z-30 flex items-center justify-between gap-2">
            <div class="flex items-center gap-1.5
                        px-2.5 py-1 rounded-lg
                        bg-black/50 backdrop-blur-sm shrink-0">
              <div class="w-1.5 h-1.5 rounded-full bg-sift-accent" />
              <span class="text-[11px] text-white/70">当前浏览</span>
            </div>
            <div class="px-2 py-1 rounded-lg
                        bg-black/50 backdrop-blur-sm
                        text-[10px] text-white/50 font-mono truncate">
              {{ currentFileName }}
            </div>
          </div>
          <!-- Loading state -->
          <div
            v-if="isLoading && !thumbnailSrc"
            class="flex flex-col items-center justify-center gap-4"
          >
            <SkeletonImage width="80%" height="70%" />
            <div class="flex items-center gap-2 text-sift-muted/50 text-xs">
              <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
              </svg>
              <span>加载中...</span>
            </div>
          </div>
          <!-- Thumbnail blur -->
          <img
            v-if="isLoading && thumbnailSrc"
            :src="thumbnailSrc"
            class="max-w-full max-h-full object-contain blur-sm opacity-60"
            draggable="false"
          />
          <!-- Full image -->
          <img
            v-if="!isLoading && currentSrc"
            :key="session.currentIndex"
            :src="currentSrc"
            class="shrink-0"
            :style="imageStyle"
            draggable="false"
          />
          <!-- Error -->
          <div
            v-if="loadError"
            class="flex flex-col items-center justify-center text-sift-muted"
          >
            <svg class="w-12 h-12 mb-2 opacity-30" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
            <p class="text-xs">加载失败</p>
          </div>
        </div>

        <!-- Center Swap Button: on the divider, vertically centered -->
        <button
          class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-40
                 w-8 h-8 rounded-full
                 bg-black/60 backdrop-blur-sm border border-white/15
                 flex items-center justify-center
                 hover:bg-white/20 hover:scale-110
                 transition-all group"
          title="将当前浏览图设为基准图"
          @click="handleSwapCompare"
        >
          <ArrowLeftRight :size="14" class="text-white/50 group-hover:text-white transition-colors" />
        </button>

        <!-- Zoom indicator in compare mode -->
        <Transition name="nav-fade">
          <div
            v-if="isZoomed"
            class="absolute bottom-16 left-1/2 -translate-x-1/2 z-30
                   px-2.5 py-1 rounded-lg
                   bg-black/50 backdrop-blur-sm
                   text-[11px] text-white/70 font-mono"
          >
            {{ zoomPercent }}%
          </div>
        </Transition>
      </div>

      <!-- Delete flash in compare mode -->
      <div
        v-if="session.currentPair?.status === 'deleted'"
        class="absolute inset-0 bg-sift-delete/20 z-40 pointer-events-none delete-flash"
      />
    </template>

    <!-- ==================== NORMAL MODE ==================== -->
    <template v-else>
      <div class="absolute inset-0 flex items-center justify-center"
           @contextmenu.prevent="handleContextMenu($event)"
      >
        <!-- Ambient Background Glow -->
        <div class="ambient-glow" :style="ambientStyle" />

        <!-- Skeleton Loading -->
        <div
          v-if="isLoading && !thumbnailSrc"
          class="absolute inset-0 flex flex-col items-center justify-center z-10 gap-4"
        >
          <SkeletonImage width="60%" height="70%" />
          <div class="flex items-center gap-2 text-sift-muted/50 text-xs">
            <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
            </svg>
            <span>加载中...</span>
          </div>
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

    <!-- ==================== COMPARE TOGGLE BUTTON (always visible) ==================== -->
    <div
      class="absolute bottom-4 right-4 z-40
             px-2 py-1.5 rounded-xl
             bg-black/50 backdrop-blur-2xl
             shadow-[0_8px_32px_rgba(0,0,0,0.4)]
             border border-white/10"
    >
      <button
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg
               transition-all group btn-spring"
        :class="view.compareMode
          ? 'bg-sift-accent/15 hover:bg-sift-accent/25'
          : 'hover:bg-white/10'"
        @click.stop="handleCompare"
      >
        <Columns2
          :size="16"
          class="transition-colors"
          :class="view.compareMode ? 'text-sift-accent' : 'text-white/40 group-hover:text-white'"
        />
        <span
          class="text-[11px] transition-colors drop-shadow-[0_1px_2px_rgba(0,0,0,0.8)]"
          :class="view.compareMode ? 'text-sift-accent' : 'text-white/60 group-hover:text-white'"
        >{{ view.compareMode ? '退出对比' : '对比' }}</span>
        <kbd class="text-[9px] text-white/30 bg-white/[0.06] px-1 rounded">C</kbd>
      </button>
    </div>

    <!-- Context Menu -->
    <ContextMenu />
  </div>
</template>
