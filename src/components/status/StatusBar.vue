<script setup lang="ts">
// ============================================================
// StatusBar - Bottom progress bar with rolling numbers
// ============================================================

import { computed } from 'vue';
import { useSessionStore } from '@/stores/sessionStore';
import { useViewStore } from '@/stores/viewStore';
import { PhotoStatus } from '@/types';
import RollingNumber from './RollingNumber.vue';

const session = useSessionStore();
const view = useViewStore();

const progressWidth = computed(() => `${session.progress}%`);

function handleStatusClick(status: PhotoStatus) {
  view.openFilterGallery(status);
}
</script>

<template>
  <div
    class="fixed bottom-0 left-0 right-0 h-10 z-50
           bg-[#121212] border-t border-white/5
           flex items-center px-4"
  >
    <!-- Left: Progress counter -->
    <div class="flex items-center gap-1.5 min-w-[100px]">
      <span class="text-sm font-mono text-white font-medium">
        <RollingNumber :value="session.currentIndex + 1" />
      </span>
      <span class="text-sm text-sift-muted">/</span>
      <span class="text-sm font-mono text-sift-muted">
        {{ session.totalCount }}
      </span>
    </div>

    <!-- Center: Shimmer progress bar -->
    <div class="flex-1 mx-6 relative">
      <div class="h-1 bg-sift-card rounded-full overflow-hidden">
        <div
          class="h-full rounded-full bg-gradient-to-r from-sift-accent to-blue-400
                 transition-all duration-500 ease-out relative overflow-hidden"
          :style="{ width: progressWidth }"
        >
          <!-- Shimmer overlay -->
          <div class="absolute inset-0 shimmer-bar" />
        </div>
      </div>
    </div>

    <!-- Right: Status counts (clickable) -->
    <div class="flex items-center gap-4 min-w-[200px] justify-end">
      <!-- Starred -->
      <button
        class="flex items-center gap-1.5 h-full rounded px-1 -mx-1
               hover:bg-white/5 transition-colors cursor-pointer"
        @click="handleStatusClick(PhotoStatus.Starred)"
      >
        <span class="w-2 h-2 shrink-0 rounded-full bg-sift-star" />
        <RollingNumber :value="session.starredCount" class="text-xs text-sift-muted" />
      </button>

      <!-- Deleted -->
      <button
        class="flex items-center gap-1.5 h-full rounded px-1 -mx-1
               hover:bg-white/5 transition-colors cursor-pointer"
        @click="handleStatusClick(PhotoStatus.Deleted)"
      >
        <span class="w-2 h-2 shrink-0 rounded-full bg-sift-delete" />
        <RollingNumber :value="session.deletedCount" class="text-xs text-sift-muted" />
      </button>

      <!-- Skipped -->
      <button
        class="flex items-center gap-1.5 h-full rounded px-1 -mx-1
               hover:bg-white/5 transition-colors cursor-pointer"
        @click="handleStatusClick(PhotoStatus.Skipped)"
      >
        <span class="w-2 h-2 shrink-0 rounded-full bg-sift-skip" />
        <RollingNumber :value="session.skippedCount" class="text-xs text-sift-muted" />
      </button>

      <!-- Unprocessed -->
      <button
        class="flex items-center gap-1.5 h-full rounded px-1 -mx-1
               hover:bg-white/5 transition-colors cursor-pointer"
        @click="handleStatusClick(PhotoStatus.Unprocessed)"
      >
        <span class="w-2 h-2 shrink-0 rounded-full bg-sift-accent" />
        <RollingNumber :value="session.unprocessedCount" class="text-xs text-sift-muted" />
      </button>
    </div>
  </div>
</template>
