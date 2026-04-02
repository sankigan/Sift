<script setup lang="ts">
// ============================================================
// SummaryCard - Culling completion summary with donut chart
// ============================================================

import { computed } from 'vue'
import { useSessionStore } from '@/stores/sessionStore'
import { useViewStore } from '@/stores/viewStore'
import RollingNumber from '@/components/status/RollingNumber.vue'

const session = useSessionStore()
const view = useViewStore()

// SVG donut chart calculations
const total = computed(() => session.totalCount || 1)
const segments = computed(() => {
  const s = session.starredCount
  const d = session.deletedCount
  const k = session.skippedCount
  const u = session.unprocessedCount

  const circumference = 2 * Math.PI * 45 // r=45
  let offset = 0

  const result = [
    { count: s, color: '#F59E0B', label: '已标记', offset: 0, length: 0 },
    { count: d, color: '#EF4444', label: '已删除', offset: 0, length: 0 },
    { count: k, color: '#6B7280', label: '已跳过', offset: 0, length: 0 },
    { count: u, color: '#3B82F6', label: '未处理', offset: 0, length: 0 },
  ]

  for (const seg of result) {
    seg.offset = offset
    seg.length = (seg.count / total.value) * circumference
    offset += seg.length
  }

  return { items: result, circumference }
})

function openArchive() {
  view.toggleArchiveDialog()
}
</script>

<template>
  <Transition name="modal">
    <div
      v-if="session.hasReachedEnd"
      class="absolute inset-0 z-50 flex items-center justify-center"
    >
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" />

      <!-- Card -->
      <div
        class="relative max-w-md w-full mx-4 p-6 rounded-2xl
               bg-sift-surface/80 backdrop-blur-xl
               border border-white/5 shadow-2xl"
      >
        <h2 class="text-xl font-bold text-white text-center mb-6">
          全部审阅完毕 ✨
        </h2>

        <!-- Donut Chart -->
        <div class="flex justify-center mb-6">
          <svg width="140" height="140" viewBox="0 0 100 100">
            <circle
              v-for="(seg, i) in segments.items"
              :key="i"
              cx="50" cy="50" r="45"
              fill="none"
              :stroke="seg.color"
              stroke-width="8"
              :stroke-dasharray="`${seg.length} ${segments.circumference - seg.length}`"
              :stroke-dashoffset="-seg.offset"
              transform="rotate(-90 50 50)"
              stroke-linecap="round"
              class="transition-all duration-700"
            />
            <text x="50" y="48" text-anchor="middle" class="fill-white text-lg font-bold" font-size="16">
              {{ session.totalCount }}
            </text>
            <text x="50" y="60" text-anchor="middle" class="fill-sift-muted" font-size="7">
              张
            </text>
          </svg>
        </div>

        <!-- Stats -->
        <div class="space-y-2.5 mb-6">
          <div
            v-for="seg in segments.items"
            :key="seg.label"
            class="flex items-center justify-between px-3 py-2 rounded-lg bg-sift-card/50"
          >
            <div class="flex items-center gap-2">
              <span class="w-2.5 h-2.5 rounded-full" :style="{ backgroundColor: seg.color }" />
              <span class="text-sm text-sift-muted">{{ seg.label }}</span>
            </div>
            <span class="text-sm font-medium text-white">
              <RollingNumber :value="seg.count" />
            </span>
          </div>
        </div>

        <!-- CTA -->
        <button
          class="w-full h-12 rounded-xl bg-gradient-to-r from-sift-accent to-blue-500
                 text-white font-medium text-sm breathe-cta btn-spring
                 shadow-lg shadow-sift-accent/20"
          @click="openArchive"
        >
          开始归档
        </button>
      </div>
    </div>
  </Transition>
</template>
