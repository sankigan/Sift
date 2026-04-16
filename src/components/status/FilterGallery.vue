<script setup lang="ts">
// ============================================================
// FilterGallery - Bottom drawer gallery filtered by photo status
// ============================================================

import { computed } from 'vue';
import { useSessionStore } from '@/stores/sessionStore';
import { useViewStore } from '@/stores/viewStore';
import { PhotoStatus } from '@/types';
import { convertFileSrc } from '@tauri-apps/api/core';
import { Star, Trash2, SkipForward, Circle, X } from 'lucide-vue-next';

const session = useSessionStore();
const view = useViewStore();

const categories = [
  { key: PhotoStatus.Starred, label: '已标记', icon: Star, color: 'text-sift-star', bgColor: 'bg-sift-star' },
  { key: PhotoStatus.Deleted, label: '已删除', icon: Trash2, color: 'text-sift-delete', bgColor: 'bg-sift-delete' },
  { key: PhotoStatus.Skipped, label: '已跳过', icon: SkipForward, color: 'text-sift-muted', bgColor: 'bg-sift-skip' },
  { key: PhotoStatus.Unprocessed, label: '未处理', icon: Circle, color: 'text-sift-accent', bgColor: 'bg-sift-accent' },
];

const isOpen = computed(() => view.filterCategory !== null);

const activeCategory = computed(() => view.filterCategory);

function getCategoryCount(key: PhotoStatus): number {
  return session.pairs.filter(p => p.status === key).length;
}

const filteredPairs = computed(() => {
  if (!view.filterCategory) return [];
  return session.pairs
    .map((pair, index) => ({ pair, originalIndex: index }))
    .filter(({ pair }) => pair.status === view.filterCategory);
});

function getThumbnailSrc(pair: { thumbnailPath?: string; jpgPath: string }): string {
  if (pair.thumbnailPath) {
    return convertFileSrc(pair.thumbnailPath);
  }
  return convertFileSrc(pair.jpgPath);
}

function getFileName(pair: { jpgPath: string; rawPath?: string | null; source?: string }): string {
  // For RAW-only photos, show the RAW filename
  if (pair.source === 'rawPreview' && pair.rawPath) {
    return pair.rawPath.split('/').pop()?.split('\\').pop() || '';
  }
  return pair.jpgPath.split('/').pop()?.split('\\').pop() || '';
}

function handleSelect(originalIndex: number) {
  session.goTo(originalIndex);
  view.closeFilterGallery();
}

function handleBackdropClick() {
  view.closeFilterGallery();
}

function switchCategory(key: PhotoStatus) {
  view.openFilterGallery(key);
}
</script>

<template>
  <Teleport to="body">
    <!-- Backdrop (semi-transparent, no blur) -->
    <Transition name="backdrop">
      <div
        v-if="isOpen"
        class="fixed inset-0 z-[100] bg-black/50"
        @click="handleBackdropClick"
      />
    </Transition>

    <!-- Bottom Drawer -->
    <Transition name="drawer">
      <div
        v-if="isOpen"
        class="fixed bottom-10 left-0 right-0 z-[101]
               max-h-[70vh] flex flex-col
               bg-[#161616]/95 backdrop-blur-2xl
               border-t border-white/[0.06]
               rounded-t-2xl overflow-hidden
               shadow-[0_-8px_40px_rgba(0,0,0,0.5)]"
        @click.stop
      >
        <!-- Drag Handle -->
        <div class="flex justify-center pt-2.5 pb-1">
          <div class="w-8 h-1 rounded-full bg-white/15" />
        </div>

        <!-- Tab Bar -->
        <div class="flex items-center px-4 pb-2 gap-1">
          <button
            v-for="cat in categories"
            :key="cat.key"
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium
                   transition-all duration-200"
            :class="[
              activeCategory === cat.key
                ? `bg-white/10 ${cat.color}`
                : 'text-sift-muted hover:text-sift-text hover:bg-white/5',
            ]"
            @click="switchCategory(cat.key)"
          >
            <component :is="cat.icon" :size="13" />
            <span>{{ cat.label }}</span>
            <span
              class="ml-0.5 text-[10px] px-1.5 py-0.5 rounded-full min-w-[20px] text-center
                     transition-colors duration-200"
              :class="[
                activeCategory === cat.key
                  ? 'bg-white/10 text-white/80'
                  : 'bg-white/5 text-sift-muted/60',
              ]"
            >
              {{ getCategoryCount(cat.key) }}
            </span>
          </button>

          <!-- Close button (pushed right) -->
          <button
            class="ml-auto p-1.5 rounded-lg text-sift-muted hover:text-sift-text
                   hover:bg-white/5 transition-colors"
            @click="view.closeFilterGallery()"
          >
            <X :size="14" />
          </button>
        </div>

        <!-- Divider -->
        <div class="h-px bg-white/[0.06] mx-4" />

        <!-- Grid Content -->
        <div class="flex-1 overflow-y-auto p-4">
          <div
            v-if="filteredPairs.length === 0"
            class="flex flex-col items-center justify-center h-32 gap-2"
          >
            <component
              :is="categories.find(c => c.key === activeCategory)?.icon"
              :size="24"
              class="text-sift-muted/30"
            />
            <span class="text-sift-muted/50 text-xs">暂无图片</span>
          </div>
          <div
            v-else
            class="grid gap-2"
            style="grid-template-columns: repeat(auto-fill, minmax(110px, 1fr))"
          >
            <div
              v-for="{ pair, originalIndex } in filteredPairs"
              :key="pair.id"
              class="group relative aspect-[4/3] rounded-lg overflow-hidden cursor-pointer
                     bg-sift-card/50 transition-all duration-200
                     hover:ring-1.5 hover:ring-white/20 hover:scale-[1.02]"
              :class="[
                originalIndex === session.currentIndex
                  ? 'ring-1.5 ring-sift-accent'
                  : '',
              ]"
              @click="handleSelect(originalIndex)"
            >
              <img
                :src="getThumbnailSrc(pair)"
                :alt="getFileName(pair)"
                class="w-full h-full object-cover"
                loading="lazy"
              />
              <!-- Filename overlay -->
              <div
                class="absolute bottom-0 inset-x-0 bg-gradient-to-t from-black/70 to-transparent
                       px-1.5 py-1 opacity-0 group-hover:opacity-100 transition-opacity"
              >
                <span class="text-[10px] text-white/80 truncate block">
                  {{ getFileName(pair) }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Context Menu -->
    <ContextMenu />
  </Teleport>
</template>
