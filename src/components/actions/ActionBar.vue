<script setup lang="ts">
// ============================================================
// ActionBar - Floating action buttons (Star / Delete / Skip)
// ============================================================

import { Star, Trash2, SkipForward } from 'lucide-vue-next';
import { useSessionStore } from '@/stores/sessionStore';
import { ref, computed } from 'vue';
import { PhotoStatus } from '@/types';

const session = useSessionStore();

const starActive = ref(false);
const deleteActive = ref(false);

const isStarred = computed(() => session.currentPair?.status === PhotoStatus.Starred);
const isDeleted = computed(() => session.currentPair?.status === PhotoStatus.Deleted);
const isSkipped = computed(() => session.currentPair?.status === PhotoStatus.Skipped);

async function handleStar() {
  starActive.value = true;
  session.markStar();
  setTimeout(() => (starActive.value = false), 400);
}

function handleDelete() {
  deleteActive.value = true;
  session.markDelete();
  setTimeout(() => (deleteActive.value = false), 300);
}

function handleSkip() {
  session.markSkip();
}
</script>

<template>
  <div
    class="absolute bottom-[120px] left-1/2 -translate-x-1/2 z-40
           px-2 py-1.5 rounded-xl
           bg-black/50 backdrop-blur-2xl
           shadow-[0_8px_32px_rgba(0,0,0,0.4)]
           border border-white/10
           flex items-center gap-1"
  >
    <!-- Star Button -->
    <button
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg group btn-spring
             transition-colors"
      :class="isStarred ? 'bg-sift-star/15 hover:bg-sift-star/25' : 'hover:bg-sift-star/15'"
      @click="handleStar"
    >
      <Star
        :size="16"
        class="transition-all"
        :class="[
          starActive ? 'scale-110' : '',
          isStarred
            ? 'text-sift-star fill-sift-star'
            : 'text-sift-star',
        ]"
      />
      <span
        class="text-[11px] group-hover:text-white transition-colors drop-shadow-[0_1px_2px_rgba(0,0,0,0.8)]"
        :class="isStarred ? 'text-sift-star' : 'text-white/60'"
      >{{ isStarred ? '取消标记' : '标记' }}</span>
      <kbd class="text-[9px] text-white/30 bg-white/[0.06] px-1 rounded">F</kbd>
    </button>

    <div class="w-px h-4 bg-white/10" />

    <!-- Delete Button -->
    <button
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg group btn-spring
             transition-colors"
      :class="isDeleted ? 'bg-sift-delete/15 hover:bg-sift-delete/25' : 'hover:bg-sift-delete/15'"
      @click="handleDelete"
    >
      <Trash2 :size="16" class="text-sift-delete" />
      <span
        class="text-[11px] group-hover:text-white transition-colors drop-shadow-[0_1px_2px_rgba(0,0,0,0.8)]"
        :class="isDeleted ? 'text-sift-delete' : 'text-white/60'"
      >{{ isDeleted ? '取消删除' : '删除' }}</span>
      <kbd class="text-[9px] text-white/30 bg-white/[0.06] px-1 rounded">X</kbd>
    </button>

    <div class="w-px h-4 bg-white/10" />

    <!-- Skip Button -->
    <button
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg group btn-spring
             transition-colors"
      :class="isSkipped ? 'bg-white/10 hover:bg-white/15' : 'hover:bg-white/10'"
      @click="handleSkip"
    >
      <SkipForward
        :size="16"
        class="transition-colors"
        :class="isSkipped ? 'text-white/70' : 'text-white/40 group-hover:text-white'"
      />
      <span
        class="text-[11px] group-hover:text-white transition-colors drop-shadow-[0_1px_2px_rgba(0,0,0,0.8)]"
        :class="isSkipped ? 'text-white/70' : 'text-white/60'"
      >{{ isSkipped ? '取消跳过' : '跳过' }}</span>
      <kbd class="text-[9px] text-white/30 bg-white/[0.06] px-1 rounded">&rarr;</kbd>
    </button>
  </div>
</template>
