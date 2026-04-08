<script setup lang="ts">
// ============================================================
// ArchiveDialog - Modal for archiving and exporting photos
// ============================================================

import { ref, computed } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpen, CheckCircle, Download, Archive } from 'lucide-vue-next'
import { useSessionStore } from '@/stores/sessionStore'
import { useViewStore } from '@/stores/viewStore'
import { archivePhotos, exportPicks, deletePair, onArchiveProgress } from '@/services/tauriCommands'
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

const deletedPairs = computed(() =>
  session.pairs.filter((p) => p.status === PhotoStatus.Deleted)
)

const unprocessedCount = computed(
  () => session.pairs.filter((p) => p.status === PhotoStatus.Unprocessed).length
)

async function handleArchive() {
  isArchiving.value = true
  archiveProgress.value = 0

  try {
    // Step 1: Delete pairs marked as Deleted
    const toDelete = deletedPairs.value
    for (const pair of toDelete) {
      currentFile.value = pair.jpgPath.split('/').pop() || pair.jpgPath
      await deletePair(pair.jpgPath, pair.rawPath)
    }

    // Step 2: Archive surviving pairs
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
      resultMessage.value = `已删除 ${toDelete.length} 组，归档 ${result.movedCount} 个文件`
      isComplete.value = true
    } finally {
      unlisten()
    }
  } catch (e: any) {
    resultMessage.value = `归档失败：${e.message || e}`
  } finally {
    isArchiving.value = false
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
    resultMessage.value = `已导出 ${result.exportedCount} 个标记文件至 ${result.exportFolder}`
    isComplete.value = true
  } catch (e: any) {
    resultMessage.value = `导出失败：${e.message || e}`
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
            <h3 class="text-lg font-bold text-white mb-2">完成！</h3>
            <p class="text-sm text-sift-muted mb-6">{{ resultMessage }}</p>
            <button
              class="w-full h-11 rounded-xl bg-sift-card text-white text-sm
                     hover:bg-sift-border transition-colors btn-spring"
              @click="done"
            >
              返回首页
            </button>
          </div>

          <!-- Archiving Progress -->
          <div v-else-if="isArchiving || isExporting" class="py-4">
            <h3 class="text-lg font-bold text-white text-center mb-4">
              {{ isArchiving ? '归档中...' : '导出中...' }}
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
              归档与导出
            </h3>
            <p class="text-xs text-sift-muted text-center mb-5">
              整理你的照片
            </p>

            <!-- Stats Summary -->
            <div class="grid grid-cols-3 gap-3 mb-5">
              <div class="bg-sift-card/60 rounded-xl p-3 border-l-2 border-sift-star">
                <p class="text-[11px] text-sift-muted">已标记</p>
                <p class="text-lg font-bold text-white">
                  <RollingNumber :value="starredPairs.length" />
                </p>
              </div>
              <div class="bg-sift-card/60 rounded-xl p-3 border-l-2 border-sift-delete">
                <p class="text-[11px] text-sift-muted">待删除</p>
                <p class="text-lg font-bold text-white">
                  <RollingNumber :value="deletedPairs.length" />
                </p>
              </div>
              <div class="bg-sift-card/60 rounded-xl p-3 border-l-2 border-sift-success">
                <p class="text-[11px] text-sift-muted">保留</p>
                <p class="text-lg font-bold text-white">
                  <RollingNumber :value="survivingPairs.length" />
                </p>
              </div>
            </div>

            <!-- Archive path -->
            <div class="bg-sift-card/40 rounded-lg px-3 py-2 mb-5">
              <p class="text-[11px] text-sift-muted mb-1">归档至</p>
              <p class="text-xs text-white truncate">{{ session.folderPath }}/</p>
              <p class="text-[10px] text-sift-muted mt-0.5">→ RAW/ 和 JPG/ 子文件夹</p>
            </div>

            <!-- Unprocessed warning -->
            <div
              v-if="unprocessedCount > 0"
              class="bg-amber-500/10 border border-amber-500/20 rounded-lg px-3 py-2 mb-5
                     flex items-start gap-2"
            >
              <span class="text-amber-400 text-xs mt-0.5">⚠️</span>
              <p class="text-xs text-amber-200/80">
                还有 {{ unprocessedCount }} 张未处理照片，归档后它们将留在原位
              </p>
            </div>

            <!-- Actions -->
            <div class="space-y-3">
              <button
                class="w-full h-12 rounded-xl bg-gradient-to-r from-sift-accent to-blue-500
                       text-white font-medium text-sm flex items-center justify-center gap-2
                       btn-spring btn-glow shadow-lg shadow-sift-accent/20"
                @click="handleArchive"
              >
                <Archive :size="16" />
                归档所有保留的照片
              </button>

              <button
                v-if="starredPairs.length > 0"
                class="w-full h-11 rounded-xl border border-sift-border
                       text-sift-text text-sm flex items-center justify-center gap-2
                       hover:bg-sift-card/50 transition-colors btn-spring"
                @click="handleExport"
              >
                <Download :size="16" />
                导出 {{ starredPairs.length }} 张标记照片
              </button>

              <button
                class="w-full h-10 text-sift-muted text-xs hover:text-white transition-colors"
                @click="close"
              >
                取消
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>
