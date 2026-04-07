<script setup lang="ts">
// ============================================================
// StatusBar - Bottom progress bar with rolling numbers
// ============================================================

import { computed } from 'vue'
import { useSessionStore } from '@/stores/sessionStore'
import RollingNumber from './RollingNumber.vue'

const session = useSessionStore()

const progressWidth = computed(() => `${session.progress}%`)
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

    <!-- Right: Status counts -->
    <div class="flex items-center gap-4 min-w-[200px] justify-end">
      <!-- Starred -->
      <div class="flex items-center gap-1.5 h-full">
        <span class="w-2 h-2 shrink-0 rounded-full bg-sift-star" />
        <RollingNumber :value="session.starredCount" class="text-xs text-sift-muted" />
      </div>

      <!-- Deleted -->
      <div class="flex items-center gap-1.5 h-full">
        <span class="w-2 h-2 shrink-0 rounded-full bg-sift-delete" />
        <RollingNumber :value="session.deletedCount" class="text-xs text-sift-muted" />
      </div>

      <!-- Skipped -->
      <div class="flex items-center gap-1.5 h-full">
        <span class="w-2 h-2 shrink-0 rounded-full bg-sift-skip" />
        <RollingNumber :value="session.skippedCount" class="text-xs text-sift-muted" />
      </div>

      <!-- Unprocessed -->
      <div class="flex items-center gap-1.5 h-full">
        <span class="w-2 h-2 shrink-0 rounded-full bg-sift-accent" />
        <RollingNumber :value="session.unprocessedCount" class="text-xs text-sift-muted" />
      </div>
    </div>
  </div>
</template>
