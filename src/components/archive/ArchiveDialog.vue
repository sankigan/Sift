<script setup lang="ts">
// ============================================================
// ArchiveDialog - Modal for archiving and exporting photos
// ============================================================

import { ref, computed } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpen, CheckCircle, Download, Archive } from 'lucide-vue-next'
import { useSessionStore } from '@/stores/sessionStore'
import { useViewStore } from '@/stores/viewStore'
import { archivePhotos, exportPicks, onArchiveProgress } from '@/services/tauriCommands'
import { PhotoStatus, type ArchiveProgress } from '@/types'
import RollingNumber from '@/components/status/RollingNumber.vue'

const session = useSessionStore()
const view = useViewStore()

const isArchiving = ref(false)
const isExporting = ref(false)
const isComplete = ref(false)
const archiveProgress = ref(0)
const archiveTotal = ref(0)
const currentFile = ref('')
const exportFolder = ref('')
const resultMessage = ref('')

const survivingPairs = computed(() =>
  session.pairs.filter(
    (p) => p.status === PhotoStatus.Starred || p.status === PhotoStatus.Skipped
  )
)

const starredPairs = computed(() =>
  session.pairs.filter((p) => p.status === PhotoStatus.Starred)
)

async function handleArchive() {
  isArchiving.value = true
  archiveProgress.value = 0

  // Listen for progress
  const unlisten = await onArchiveProgress((progress: ArchiveProgress) => {
    archiveProgress.value = progress.current
    archiveTotal.value = progress.total
    currentFile.value = progress.currentFile
  })

  try {
    const pairsData = survivingPairs.value.map((p) => ({
      jpgPath: p.jpgPath,
      rawPath: p.rawPath,
      status: p.status,
    }))

    const result = await archivePhotos(session.folderPath, pairsData)
    resultMessage.value = `Archived ${result.movedCount} files`
    isComplete.value = true
  } catch (e: any) {
    resultMessage.value = `Archive failed: ${e.message || e}`
  } finally {
    isArchiving.value = false
    unlisten()
  }
}

async function handleExport() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择导出目标',
  })

  if (!selected || typeof selected !== 'string') return

  isExporting.value = true
  try {
    const pairsData = starredPairs.value.map((p) => ({
      jpgPath: p.jpgPath,
      rawPath: p.rawPath,
    }))

    const result = await exportPicks(pairsData, selected)
    resultMessage.value = `Exported ${result.exportedCount} starred files to ${result.exportFolder}`
    isComplete.value = true
  } catch (e: any) {
    resultMessage.value = `Export failed: ${e.message || e}`
  } finally {
    isExporting.value = false
  }
}

function close() {
  view.toggleArchiveDialog()
}

function done() {
  session.resetSession()
  view.setView('welcome')
  view.toggleArchiveDialog()
}

const progressPercent = computed(() => {
  if (archiveTotal.value === 0) return 0
  return Math.round((archiveProgress.value / archiveTotal.value) * 100)
})
</script>

<template>
  <Transition name="backdrop">
    <div
      v-if="view.showArchiveDialog"
      class="fixed inset-0 z-50 flex items-center justify-center"
    >
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" @click="close" />

      <!-- Dialog -->
      <Transition name="modal">
        <div
          class="relative max-w-lg w-full mx-4 p-6 rounded-2xl
                 bg-sift-surface border border-white/5 shadow-2xl"
        >
          <!-- Complete State -->
          <div v-if="isComplete" class="text-center py-4">
            <div class="mx-auto w-16 h-16 rounded-full bg-sift-success/20 flex items-center justify-center mb-4">
              <CheckCircle :size="32" class="text-sift-success" />
            </div>
            <h3 class="text-lg font-bold text-white mb-2">Done!</h3>
            <p class="text-sm text-sift-muted mb-6">{{ resultMessage }}</p>
            <button
              class="w-full h-11 rounded-xl bg-sift-card text-white text-sm
                     hover:bg-sift-border transition-colors btn-spring"
              @click="done"
            >
              Back to Home
            </button>
          </div>

          <!-- Archiving Progress -->
          <div v-else-if="isArchiving || isExporting" class="py-4">
            <h3 class="text-lg font-bold text-white text-center mb-4">
              {{ isArchiving ? 'Archiving...' : 'Exporting...' }}
            </h3>
            <!-- Progress Bar -->
            <div class="h-2 bg-sift-card rounded-full overflow-hidden mb-3">
              <div
                class="h-full rounded-full bg-gradient-to-r from-sift-accent to-blue-400
                       transition-all duration-300 relative overflow-hidden"
                :style="{ width: `${progressPercent}%` }"
              >
                <div class="absolute inset-0 shimmer-bar" />
              </div>
            </div>
            <p class="text-xs text-sift-muted text-center truncate">
              {{ currentFile }}
            </p>
          </div>

          <!-- Default: Archive Options -->
          <div v-else>
            <h3 class="text-lg font-bold text-white text-center mb-1">
              Archive & Export
            </h3>
            <p class="text-xs text-sift-muted text-center mb-5">
              Organize your photos
            </p>

            <!-- Stats Summary -->
            <div class="grid grid-cols-2 gap-3 mb-5">
              <div class="bg-sift-card/60 rounded-xl p-3 border-l-2 border-sift-star">
                <p class="text-[11px] text-sift-muted">Starred</p>
                <p class="text-lg font-bold text-white">
                  <RollingNumber :value="starredPairs.length" />
                </p>
              </div>
              <div class="bg-sift-card/60 rounded-xl p-3 border-l-2 border-sift-success">
                <p class="text-[11px] text-sift-muted">Surviving</p>
                <p class="text-lg font-bold text-white">
                  <RollingNumber :value="survivingPairs.length" />
                </p>
              </div>
            </div>

            <!-- Archive path -->
            <div class="bg-sift-card/40 rounded-lg px-3 py-2 mb-5">
              <p class="text-[11px] text-sift-muted mb-1">Archive to</p>
              <p class="text-xs text-white truncate">{{ session.folderPath }}/</p>
              <p class="text-[10px] text-sift-muted mt-0.5">→ RAW/ and JPG/ subfolders</p>
            </div>

            <!-- Actions -->
            <div class="space-y-3">
              <button
                class="w-full h-12 rounded-xl bg-gradient-to-r from-sift-accent to-blue-500
                       text-white font-medium text-sm flex items-center justify-center gap-2
                       btn-spring shadow-lg shadow-sift-accent/20"
                @click="handleArchive"
              >
                <Archive :size="16" />
                Archive All Surviving Photos
              </button>

              <button
                v-if="starredPairs.length > 0"
                class="w-full h-11 rounded-xl border border-sift-border
                       text-sift-text text-sm flex items-center justify-center gap-2
                       hover:bg-sift-card/50 transition-colors btn-spring"
                @click="handleExport"
              >
                <Download :size="16" />
                Export {{ starredPairs.length }} Starred Photos
              </button>

              <button
                class="w-full h-10 text-sift-muted text-xs hover:text-white transition-colors"
                @click="close"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>
