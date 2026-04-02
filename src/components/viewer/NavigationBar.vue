<script setup lang="ts">
// ============================================================
// NavigationBar - Top bar with folder name, RAW badge, EXIF toggle
// ============================================================

import { computed } from 'vue'
import { ArrowLeft, Info } from 'lucide-vue-next'
import { useSessionStore } from '@/stores/sessionStore'
import { useViewStore } from '@/stores/viewStore'

const session = useSessionStore()
const view = useViewStore()

const folderName = computed(() => {
  const path = session.folderPath
  return path.split('/').pop() || path.split('\\').pop() || path
})

function goBack() {
  session.resetSession()
  view.setView('welcome')
}
</script>

<template>
  <div
    class="fixed top-0 left-0 right-0 h-12 z-50
           bg-[#121212]/90 backdrop-blur-md
           flex items-center justify-between px-4
           border-b border-white/5"
  >
    <!-- Left: Back + Folder name -->
    <div class="flex items-center gap-3">
      <button
        class="p-1.5 rounded-lg hover:bg-white/10 transition-colors btn-spring"
        @click="goBack"
      >
        <ArrowLeft :size="16" class="text-sift-muted" />
      </button>
      <span class="text-sm text-sift-muted truncate max-w-[200px]">
        {{ folderName }}
      </span>
    </div>

    <!-- Center: file name + RAW badge -->
    <div class="flex items-center gap-2" v-if="session.currentPair">
      <span class="text-sm text-white/90 font-medium truncate max-w-[300px]">
        {{ session.currentPair.jpgPath.split('/').pop()?.split('\\').pop() }}
      </span>
      <span
        v-if="session.currentPair.rawFormat"
        class="px-1.5 py-0.5 rounded text-[10px] font-bold uppercase
               bg-sift-success/20 text-sift-success"
      >
        {{ session.currentPair.rawFormat }}
      </span>
      <span
        v-else
        class="px-1.5 py-0.5 rounded text-[10px] font-medium
               bg-sift-muted/20 text-sift-muted"
      >
        无 RAW
      </span>
    </div>

    <!-- Right: EXIF toggle -->
    <div class="flex items-center gap-2">
      <button
        class="p-1.5 rounded-lg transition-colors btn-spring"
        :class="view.showExifPanel ? 'bg-sift-accent/20 text-sift-accent' : 'hover:bg-white/10 text-sift-muted'"
        @click="view.toggleExifPanel()"
      >
        <Info :size="16" />
      </button>
    </div>
  </div>
</template>
