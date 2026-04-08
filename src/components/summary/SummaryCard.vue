<script setup lang="ts">
// ============================================================
// SummaryCard - Culling completion summary with donut chart
// ============================================================

import { computed, ref, watch } from 'vue';
import { useSessionStore } from '@/stores/sessionStore';
import { useViewStore } from '@/stores/viewStore';
import RollingNumber from '@/components/status/RollingNumber.vue';

const session = useSessionStore();
const view = useViewStore();

const animateChart = ref(false);

// SVG donut chart calculations
const total = computed(() => session.totalCount || 1);
const segments = computed(() => {
  const s = session.starredCount;
  const d = session.deletedCount;
  const k = session.skippedCount;

  const circumference = 2 * Math.PI * 45; // r=45
  const gap = 2; // gap between segments in svg units
  let offset = 0;

  const items = [
    { count: s, color: '#F59E0B', label: '已标记', offset: 0, length: 0 },
    { count: d, color: '#EF4444', label: '已删除', offset: 0, length: 0 },
    { count: k, color: '#6B7280', label: '已跳过', offset: 0, length: 0 },
  ].filter((seg) => seg.count > 0);

  for (const seg of items) {
    seg.offset = offset;
    const raw = (seg.count / total.value) * circumference;
    seg.length = Math.max(raw - gap, 2);
    offset += raw;
  }

  return { items, circumference };
});

const starPercent = computed(() => {
  if (total.value === 0) return 0;
  return Math.round((session.starredCount / total.value) * 100);
});

const encourageText = computed(() => {
  const starred = session.starredCount;
  if (starred === 0) return '没有标记任何照片';
  if (starPercent.value <= 20) return '精挑细选，质量至上';
  if (starPercent.value <= 50) return '眼光不错，精选合集';
  return '满载而归，收获满满';
});

function openArchive() {
  view.toggleArchiveDialog();
}

function continueBrowsing() {
  view.showSummaryCard = false;
}

// Show summary card whenever a mark action happens and all photos are processed
watch(
  () => session.markActionCount,
  () => {
    if (session.hasReachedEnd) {
      view.showSummaryCard = true;
      animateChart.value = false;
      setTimeout(() => {
        animateChart.value = true;
      }, 300);
    }
  },
);
</script>

<template>
  <Transition name="modal">
    <div
      v-if="view.showSummaryCard && session.hasReachedEnd"
      class="absolute inset-0 z-50 flex items-center justify-center"
    >
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" />

      <!-- Card -->
      <div
        class="relative max-w-md w-full mx-4 p-6 rounded-2xl
               bg-sift-surface/80 backdrop-blur-xl
               border border-white/5 shadow-2xl summary-pop"
      >
        <!-- Header -->
        <h2 class="text-xl font-bold text-white text-center mb-1">
          全部审阅完毕 ✨
        </h2>
        <p class="text-sm text-sift-muted text-center mb-6">
          已选出
          <span class="text-sift-star font-medium">{{ session.starredCount }}</span>
          张精选照片 · {{ encourageText }}
        </p>

        <!-- Donut Chart -->
        <div class="flex justify-center mb-6">
          <svg width="140" height="140" viewBox="0 0 100 100">
            <!-- Background ring -->
            <circle
              cx="50" cy="50" r="45"
              fill="none"
              stroke="rgba(255,255,255,0.05)"
              stroke-width="8"
            />
            <!-- Segment rings -->
            <circle
              v-for="(seg, i) in segments.items"
              :key="i"
              cx="50" cy="50" r="45"
              fill="none"
              :stroke="seg.color"
              stroke-width="8"
              :stroke-dasharray="animateChart
                ? `${seg.length} ${segments.circumference - seg.length}`
                : `0 ${segments.circumference}`"
              :stroke-dashoffset="-seg.offset"
              transform="rotate(-90 50 50)"
              :stroke-linecap="segments.items.length === 1 ? 'round' : 'butt'"
              class="transition-all duration-700 ease-out"
              :style="{ transitionDelay: `${i * 150 + 300}ms` }"
            />
            <!-- Center text -->
            <text
              x="50" y="47"
              text-anchor="middle"
              dominant-baseline="central"
              class="fill-white font-bold"
              font-size="18"
            >
              {{ starPercent }}%
            </text>
            <text
              x="50" y="61"
              text-anchor="middle"
              dominant-baseline="central"
              class="fill-sift-muted"
              font-size="7"
            >
              标记率
            </text>
          </svg>
        </div>

        <!-- Stats with mini progress bars -->
        <div class="space-y-2.5 mb-6">
          <div
            v-for="(seg, i) in segments.items"
            :key="seg.label"
            class="px-3 py-2.5 rounded-lg bg-sift-card/50"
            :style="{ animationDelay: `${i * 80 + 500}ms` }"
          >
            <div class="flex items-center justify-between mb-1.5">
              <div class="flex items-center gap-2">
                <span
                  class="w-2.5 h-2.5 rounded-full"
                  :style="{ backgroundColor: seg.color }"
                />
                <span class="text-sm text-sift-muted">{{ seg.label }}</span>
              </div>
              <span class="text-sm font-medium text-white">
                <RollingNumber :value="seg.count" />
              </span>
            </div>
            <!-- Mini proportion bar -->
            <div class="h-1 rounded-full bg-white/5 overflow-hidden">
              <div
                class="h-full rounded-full mini-bar"
                :style="{
                  width: animateChart
                    ? `${Math.max((seg.count / total) * 100, 2)}%`
                    : '0%',
                  backgroundColor: seg.color,
                  transitionDelay: `${i * 150 + 600}ms`,
                }"
              />
            </div>
          </div>
        </div>

        <!-- CTA Buttons -->
        <div class="space-y-2.5">
          <button
            class="w-full h-12 rounded-xl bg-gradient-to-r from-sift-accent to-blue-500
                   text-white font-medium text-sm btn-spring btn-glow
                   shadow-lg shadow-sift-accent/20"
            @click="openArchive"
          >
            开始归档
          </button>

          <button
            class="w-full h-10 rounded-xl text-sift-muted text-sm
                   hover:text-white hover:bg-white/5 transition-colors"
            @click="continueBrowsing"
          >
            继续浏览
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>
