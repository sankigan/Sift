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
    class="absolute bottom-20 left-1/2 -translate-x-1/2 z-40
           px-6 py-3 rounded-2xl
           bg-black/40 backdrop-blur-xl
           shadow-2xl shadow-black/30
           border border-white/5
           flex items-center gap-8"
  >
    <!-- Star Button -->
    <button
      class="flex flex-col items-center gap-1.5 group btn-spring"
      @click="handleStar"
    >
      <div
        class="w-12 h-12 rounded-xl flex items-center justify-center
               transition-all duration-200
               bg-sift-star/10 group-hover:bg-sift-star/25"
        :class="{ 'star-burst': starActive }"
      >
        <Star
          :size="20"
          class="transition-colors"
          :class="session.currentPair?.status === 'starred'
            ? 'text-sift-star fill-sift-star'
            : 'text-sift-star'"
        />
      </div>
      <div class="flex items-center gap-1">
        <span class="text-[11px] text-sift-muted group-hover:text-white transition-colors">标记</span>
        <kbd class="text-[9px] text-sift-muted/50 bg-white/5 px-1 rounded">F</kbd>
      </div>
    </button>

    <!-- Delete Button -->
    <button
      class="flex flex-col items-center gap-1.5 group btn-spring"
      @click="handleDelete"
    >
      <div
        class="w-12 h-12 rounded-xl flex items-center justify-center
               transition-all duration-200
               bg-sift-delete/10 group-hover:bg-sift-delete/25"
        :class="{ 'scale-110': deleteActive }"
      >
        <Trash2 :size="20" class="text-sift-delete" />
      </div>
      <div class="flex items-center gap-1">
        <span class="text-[11px] text-sift-muted group-hover:text-white transition-colors">删除</span>
        <kbd class="text-[9px] text-sift-muted/50 bg-white/5 px-1 rounded">X</kbd>
      </div>
    </button>

    <!-- Skip Button -->
    <button
      class="flex flex-col items-center gap-1.5 group btn-spring"
      @click="handleSkip"
    >
      <div
        class="w-12 h-12 rounded-xl flex items-center justify-center
               transition-all duration-200
               bg-sift-skip/10 group-hover:bg-sift-skip/25"
      >
        <SkipForward :size="20" class="text-sift-muted group-hover:text-white transition-colors" />
      </div>
      <div class="flex items-center gap-1">
        <span class="text-[11px] text-sift-muted group-hover:text-white transition-colors">跳过</span>
        <kbd class="text-[9px] text-sift-muted/50 bg-white/5 px-1 rounded">→</kbd>
      </div>
    </button>
  </div>
</template>
