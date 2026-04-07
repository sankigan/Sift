<script setup lang="ts">
// ============================================================
// ActionBar - Floating action buttons (Star / Delete / Skip)
// ============================================================

import { Star, Trash2, SkipForward } from 'lucide-vue-next'
import { useSessionStore } from '@/stores/sessionStore'
import { useViewStore } from '@/stores/viewStore'
import { ref } from 'vue'
import { PhotoStatus } from '@/types'

const session = useSessionStore()
const view = useViewStore()

const starActive = ref(false)
const deleteActive = ref(false)

async function handleStar() {
  starActive.value = true
  session.markStar()
  view.showToast('已标记 ⭐', 'star')
  setTimeout(() => (starActive.value = false), 400)
}

async function handleDelete() {
  deleteActive.value = true
  try {
    await session.markDelete()
    view.showToast('已删除 🗑️', 'delete')
  } catch (e) {
    view.showToast('删除失败', 'info')
  }
  setTimeout(() => (deleteActive.value = false), 300)
}

function handleSkip() {
  session.markSkip()
  view.showToast('已跳过 ⏭️', 'skip')
}

const isMac = navigator.platform.toUpperCase().includes('MAC')
</script>

<template>
  <div
    class="absolute bottom-16 left-1/2 -translate-x-1/2 z-40
           px-2 py-1.5 rounded-xl
           bg-black/40 backdrop-blur-xl
           shadow-2xl shadow-black/30
           border border-white/5
           flex items-center gap-1"
  >
    <!-- Star Button -->
    <button
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg group btn-spring
             hover:bg-sift-star/15 transition-colors"
      @click="handleStar"
    >
      <Star
        :size="16"
        class="transition-colors"
        :class="[
          starActive ? 'scale-110' : '',
          session.currentPair?.status === 'starred'
            ? 'text-sift-star fill-sift-star'
            : 'text-sift-star',
        ]"
      />
      <span class="text-[11px] text-sift-muted group-hover:text-white transition-colors">标记</span>
      <kbd class="text-[9px] text-sift-muted/40 bg-white/5 px-1 rounded">F</kbd>
    </button>

    <div class="w-px h-4 bg-white/10" />

    <!-- Delete Button -->
    <button
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg group btn-spring
             hover:bg-sift-delete/15 transition-colors"
      @click="handleDelete"
    >
      <Trash2 :size="16" class="text-sift-delete" />
      <span class="text-[11px] text-sift-muted group-hover:text-white transition-colors">删除</span>
      <kbd class="text-[9px] text-sift-muted/40 bg-white/5 px-1 rounded">X</kbd>
    </button>

    <div class="w-px h-4 bg-white/10" />

    <!-- Skip Button -->
    <button
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg group btn-spring
             hover:bg-white/10 transition-colors"
      @click="handleSkip"
    >
      <SkipForward :size="16" class="text-sift-muted group-hover:text-white transition-colors" />
      <span class="text-[11px] text-sift-muted group-hover:text-white transition-colors">跳过</span>
      <kbd class="text-[9px] text-sift-muted/40 bg-white/5 px-1 rounded">&rarr;</kbd>
    </button>
  </div>
</template>
