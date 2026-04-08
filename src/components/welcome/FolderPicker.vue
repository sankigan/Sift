<script setup lang="ts">
// ============================================================
// FolderPicker - Welcome page with folder selection
// ============================================================

import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpen, Camera, Image, Layers, ArrowRight } from 'lucide-vue-next'
import { useSessionStore } from '@/stores/sessionStore'
import { useViewStore } from '@/stores/viewStore'

const session = useSessionStore()
const view = useViewStore()

const isHovering = ref(false)
const scanComplete = ref(false)
const scanError = ref('')

async function selectFolder() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择照片文件夹',
    })

    if (selected && typeof selected === 'string') {
      scanError.value = ''
      scanComplete.value = false
      await session.startScan(selected)
      scanComplete.value = true
    }
  } catch (e: any) {
    scanError.value = e.message || '扫描文件夹失败'
  }
}

function startCulling() {
  view.setView('culling')
}
</script>

<template>
  <div class="h-full w-full flex flex-col items-center justify-center relative overflow-hidden">
    <!-- Background gradient -->
    <div
      class="absolute inset-0 bg-gradient-radial from-sift-surface/50 to-sift-bg"
      style="background: radial-gradient(ellipse at center, rgba(26,26,26,0.5) 0%, #0D0D0D 70%)"
    />

    <!-- Content -->
    <div class="relative z-10 flex flex-col items-center max-w-xl w-full px-6">
      <!-- Logo & Slogan -->
      <div class="mb-12 text-center">
        <h1 class="text-4xl font-bold text-white mb-2 tracking-tight">
          Sift
        </h1>
        <p class="text-sm text-sift-muted font-light">
          RAW + JPG 一键筛选
        </p>
        <div class="h-0.5 w-12 mx-auto mt-3 bg-gradient-to-r from-sift-accent to-blue-400 rounded-full" />
      </div>

      <!-- Scanning State -->
      <div v-if="session.isScanning" class="w-full">
        <div
          class="border border-sift-border rounded-2xl p-12 flex flex-col items-center gap-6
                 bg-sift-surface/30 backdrop-blur-xl"
        >
          <!-- Spinning loader -->
          <div class="relative w-16 h-16">
            <div
              class="absolute inset-0 rounded-full border-2 border-sift-border"
            />
            <div
              class="absolute inset-0 rounded-full border-2 border-transparent border-t-sift-accent
                     animate-spin"
            />
          </div>
          <p class="text-sift-muted text-sm">正在扫描文件...</p>
        </div>
      </div>

      <!-- Scan Complete -->
      <div v-else-if="scanComplete && session.scanResult" class="w-full">
        <div class="w-full space-y-4">
          <!-- Stats Cards -->
          <div class="grid grid-cols-3 gap-3">
            <!-- Paired -->
            <div
              class="bg-sift-card/80 backdrop-blur-sm rounded-xl p-4 border-l-2 border-sift-success
                     transform transition-all duration-500"
              style="animation: slideUp 400ms cubic-bezier(0.34, 1.56, 0.64, 1) 0ms both"
            >
              <div class="flex items-center gap-2 mb-1">
                <Layers :size="14" class="text-sift-success" />
                <span class="text-xs text-sift-muted">已配对</span>
              </div>
              <span class="text-2xl font-bold text-white">
                {{ session.scanResult.pairedCount }}
              </span>
            </div>

            <!-- JPG Only -->
            <div
              class="bg-sift-card/80 backdrop-blur-sm rounded-xl p-4 border-l-2 border-sift-star
                     transform transition-all duration-500"
              style="animation: slideUp 400ms cubic-bezier(0.34, 1.56, 0.64, 1) 100ms both"
            >
              <div class="flex items-center gap-2 mb-1">
                <Image :size="14" class="text-sift-star" />
                <span class="text-xs text-sift-muted">仅 JPG</span>
              </div>
              <span class="text-2xl font-bold text-white">
                {{ session.scanResult.jpgOnlyCount }}
              </span>
            </div>

            <!-- Total Files -->
            <div
              class="bg-sift-card/80 backdrop-blur-sm rounded-xl p-4 border-l-2 border-sift-accent
                     transform transition-all duration-500"
              style="animation: slideUp 400ms cubic-bezier(0.34, 1.56, 0.64, 1) 200ms both"
            >
              <div class="flex items-center gap-2 mb-1">
                <Camera :size="14" class="text-sift-accent" />
                <span class="text-xs text-sift-muted">总计</span>
              </div>
              <span class="text-2xl font-bold text-white">
                {{ session.scanResult.totalFiles }}
              </span>
            </div>
          </div>

          <!-- Folder path -->
          <div class="text-xs text-sift-muted text-center truncate px-4">
            {{ session.folderPath }}
          </div>

          <!-- Start Button -->
          <button
            v-if="session.scanResult.pairs.length > 0"
            class="w-full h-12 rounded-xl bg-gradient-to-r from-sift-accent to-blue-500
                   text-white font-medium text-sm flex items-center justify-center gap-2
                   hover:from-blue-500 hover:to-blue-400 btn-spring btn-glow
                   shadow-lg shadow-sift-accent/20
                   transition-shadow duration-300"
            style="animation: slideUp 400ms cubic-bezier(0.34, 1.56, 0.64, 1) 300ms both"
            @click="startCulling"
          >
            开始筛选
            <ArrowRight :size="16" />
          </button>

          <!-- No photos found -->
          <p
            v-else
            class="text-center text-sift-muted text-sm py-4"
          >
            该文件夹中未找到 JPG 文件
          </p>

          <!-- Rescan button -->
          <button
            class="w-full h-10 rounded-xl border border-sift-border text-sift-muted
                   text-sm hover:text-white hover:border-sift-muted/50 transition-colors
                   btn-spring"
            @click="selectFolder"
          >
            选择其他文件夹
          </button>
        </div>
      </div>

      <!-- Initial State - Folder picker -->
      <div v-else class="w-full">
        <button
          class="w-full border-2 border-dashed rounded-2xl p-16 flex flex-col items-center gap-4
                 transition-all duration-300 cursor-pointer group
                 bg-sift-surface/10"
          :class="[
            isHovering
              ? 'border-sift-accent bg-sift-accent/5 scale-[1.01]'
              : 'border-sift-border hover:border-sift-muted/60'
          ]"
          @click="selectFolder"
          @mouseenter="isHovering = true"
          @mouseleave="isHovering = false"
        >
          <FolderOpen
            :size="48"
            class="transition-colors duration-300"
            :class="isHovering ? 'text-sift-accent' : 'text-sift-muted'"
          />
          <div class="text-center">
            <p class="text-sm text-sift-text mb-1">选择照片文件夹</p>
            <p class="text-xs text-sift-muted">
              支持散落图片或已归档的 JPG / RAW 子文件夹
            </p>
          </div>
        </button>

        <!-- Error message -->
        <p v-if="scanError" class="text-sift-delete text-xs text-center mt-3">
          {{ scanError }}
        </p>
      </div>

      <!-- Supported formats hint -->
      <p class="text-[10px] text-sift-muted/50 mt-8 text-center">
        支持 CR2 · CR3 · NEF · ARW · RAF · ORF · RW2 · DNG · PEF · 3FR · IIQ 等格式
      </p>
    </div>
  </div>
</template>

<style scoped>
@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(16px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
