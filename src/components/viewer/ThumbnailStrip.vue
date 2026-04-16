<script setup lang="ts">
// ============================================================
// ThumbnailStrip - Bottom thumbnail filmstrip with status bars
// Navigation via gradient edge hotzone buttons
// ============================================================

import { ref, watch, nextTick, onMounted } from 'vue';
import { ChevronLeft, ChevronRight } from 'lucide-vue-next';
import { useSessionStore } from '@/stores/sessionStore';
import { useViewStore } from '@/stores/viewStore';
import { PhotoStatus } from '@/types';
import { convertFileSrc } from '@tauri-apps/api/core';
import ContextMenu from '@/components/common/ContextMenu.vue';
import { useContextMenu } from '@/composables/useContextMenu';

const session = useSessionStore();
const view = useViewStore();
const contextMenu = useContextMenu();

const scrollContainer = ref<HTMLElement | null>(null);
const thumbnailRefs = ref<HTMLElement[]>([]);
const isHovering = ref(false);

function setThumbnailRef(el: HTMLElement | null, index: number) {
  if (el) {
    thumbnailRefs.value[index] = el;
  }
}

function scrollToActive() {
  const container = scrollContainer.value;
  const activeEl = thumbnailRefs.value[session.currentIndex];
  if (!container || !activeEl) return;

  const containerRect = container.getBoundingClientRect();
  const elRect = activeEl.getBoundingClientRect();

  const elCenter = elRect.left + elRect.width / 2;
  const containerCenter = containerRect.left + containerRect.width / 2;
  const offset = elCenter - containerCenter;

  container.scrollBy({ left: offset, behavior: 'smooth' });
}

/** Page left: scroll by 60% of the container width */
function pageLeft() {
  const container = scrollContainer.value;
  if (!container) return;
  const scrollAmount = container.clientWidth * 0.6;
  container.scrollBy({ left: -scrollAmount, behavior: 'smooth' });
}

/** Page right: scroll by 60% of the container width */
function pageRight() {
  const container = scrollContainer.value;
  if (!container) return;
  const scrollAmount = container.clientWidth * 0.6;
  container.scrollBy({ left: scrollAmount, behavior: 'smooth' });
}

function handleClick(index: number) {
  session.goTo(index);
}

/** Whether we can scroll left */
const canScrollLeft = ref(false);
/** Whether we can scroll right */
const canScrollRight = ref(false);

function updateScrollState() {
  const container = scrollContainer.value;
  if (!container) return;
  canScrollLeft.value = container.scrollLeft > 0;
  canScrollRight.value =
    container.scrollLeft + container.clientWidth < container.scrollWidth - 1;
}

function getStatusColor(status: PhotoStatus): string {
  switch (status) {
    case PhotoStatus.Starred:
      return 'bg-sift-star';
    case PhotoStatus.Deleted:
      return 'bg-sift-delete';
    case PhotoStatus.Skipped:
      return 'bg-sift-skip';
    default:
      return '';
  }
}

function getDominantColor(pair: { dominantColor?: string }): string {
  return pair.dominantColor || '#2a2a2a';
}

watch(
  () => session.currentIndex,
  async () => {
    await nextTick();
    scrollToActive();
  }
);

onMounted(async () => {
  await nextTick();
  scrollToActive();

  const container = scrollContainer.value;
  if (container) {
    container.addEventListener('scroll', updateScrollState, { passive: true });
    updateScrollState();
  }
});
</script>

<template>
  <div
    v-if="session.pairs.length > 0"
    class="fixed bottom-10 left-0 right-0 h-16 z-40 bg-[#121212] border-t border-white/5"
    @mouseenter="isHovering = true"
    @mouseleave="isHovering = false"
  >
    <!-- Fade mask container -->
    <div class="relative w-full h-full thumbnail-strip-mask">
      <!-- Left page hotzone -->
      <div
        v-show="canScrollLeft"
        class="absolute left-0 top-0 bottom-0 w-14 z-10
               flex items-center justify-center cursor-pointer
               bg-gradient-to-r from-[#121212] via-[#121212]/60 to-transparent
               transition-opacity duration-200"
        :class="isHovering ? 'opacity-100' : 'opacity-0'"
        @click="pageLeft"
      >
        <ChevronLeft
          :size="16"
          :stroke-width="1.5"
          class="text-white/50 hover:text-white/90 transition-colors"
        />
      </div>

      <!-- Right page hotzone -->
      <div
        v-show="canScrollRight"
        class="absolute right-0 top-0 bottom-0 w-14 z-10
               flex items-center justify-center cursor-pointer
               bg-gradient-to-l from-[#121212] via-[#121212]/60 to-transparent
               transition-opacity duration-200"
        :class="isHovering ? 'opacity-100' : 'opacity-0'"
        @click="pageRight"
      >
        <ChevronRight
          :size="16"
          :stroke-width="1.5"
          class="text-white/50 hover:text-white/90 transition-colors"
        />
      </div>

      <!-- Scrollable thumbnail list -->
      <div
        ref="scrollContainer"
        class="flex items-center gap-1.5 h-full px-16 overflow-x-auto thumbnail-scroll"
      >
        <div
          v-for="(pair, index) in session.pairs"
          :key="pair.id"
          :ref="(el) => setThumbnailRef(el as HTMLElement, index)"
          class="relative shrink-0 h-11 w-16 rounded-md overflow-hidden cursor-pointer
                 transition-[opacity,transform,ring-color] duration-200 ease-out group"
          :class="[
            index === session.currentIndex
              ? 'ring-2 ring-sift-accent ring-offset-1 ring-offset-[#121212] opacity-100 scale-105'
              : view.compareMode && index === view.compareIndex
                ? 'ring-2 ring-blue-400 ring-offset-1 ring-offset-[#121212] opacity-90 scale-[1.03]'
                : 'opacity-50 hover:opacity-80',
          ]"
          @click="handleClick(index)"
          @contextmenu.prevent="contextMenu.show($event, pair)"
        >
          <!-- Dominant color placeholder -->
          <div
            class="absolute inset-0"
            :style="{ backgroundColor: getDominantColor(pair) }"
          />

          <!-- Thumbnail image: prefer thumbnail, fallback to jpgPath with lazy loading -->
          <img
            :src="convertFileSrc(pair.thumbnailPath || pair.jpgPath)"
            :alt="`Photo ${index + 1}`"
            loading="lazy"
            decoding="async"
            class="absolute inset-0 w-full h-full object-cover"
            @error="($event.target as HTMLImageElement).style.display = 'none'"
          />

          <!-- Status color bar -->
          <div
            v-if="pair.status !== PhotoStatus.Unprocessed"
            class="absolute bottom-0 left-0 right-0 h-1 rounded-b-md"
            :class="getStatusColor(pair.status)"
          />

          <!-- Compare base badge -->
          <div
            v-if="view.compareMode && index === view.compareIndex"
            class="absolute top-0 left-0 right-0 flex justify-center z-10"
          >
            <span class="text-[8px] bg-blue-500/80 text-white px-1 rounded-b leading-tight">基准</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Context Menu -->
    <ContextMenu />
  </div>
</template>

<style scoped>
.thumbnail-strip-mask {
  mask-image: linear-gradient(
    to right,
    transparent 0px,
    black 60px,
    black calc(100% - 60px),
    transparent 100%
  );
  -webkit-mask-image: linear-gradient(
    to right,
    transparent 0px,
    black 60px,
    black calc(100% - 60px),
    transparent 100%
  );
}

.thumbnail-scroll {
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.thumbnail-scroll::-webkit-scrollbar {
  display: none;
}
</style>
