<script setup lang="ts">
// ============================================================
// ToastNotification - Spring animated toast with undo support
// ============================================================

import { computed } from 'vue'
import { Undo2 } from 'lucide-vue-next'
import { useViewStore } from '@/stores/viewStore'
import { useSessionStore } from '@/stores/sessionStore'

const view = useViewStore()
const session = useSessionStore()

const bgColor = computed(() => {
  switch (view.toastType) {
    case 'star': return 'bg-sift-star/20 border-sift-star/30'
    case 'delete': return 'bg-sift-delete/20 border-sift-delete/30'
    case 'undo': return 'bg-sift-accent/20 border-sift-accent/30'
    default: return 'bg-sift-card/80 border-sift-border'
  }
})

function handleUndo() {
  session.undo()
  view.showToast('已撤销 ↩️', 'undo')
}
</script>

<template>
  <Transition name="toast">
    <div
      v-if="view.toastVisible"
      class="fixed bottom-[120px] left-1/2 -translate-x-1/2 z-50
             px-4 py-2.5 rounded-xl border backdrop-blur-lg
             flex items-center gap-3 shadow-lg"
      :class="bgColor"
    >
      <span class="text-sm text-white/90">{{ view.toastMessage }}</span>
      <button
        v-if="view.toastType !== 'undo' && session.undoStack.length > 0"
        class="text-xs text-sift-accent hover:text-sift-accent/80
               flex items-center gap-1 btn-spring"
        @click="handleUndo"
      >
        <Undo2 :size="12" />
        撤销
      </button>
    </div>
  </Transition>
</template>
