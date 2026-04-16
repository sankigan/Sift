<script setup lang="ts">
// ============================================================
// App.vue - Root component with page routing
// ============================================================

import { ref, watch, defineAsyncComponent, onMounted, onUnmounted } from 'vue';
import { useViewStore } from '@/stores/viewStore';
import { useKeyboard } from '@/composables/useKeyboard';
import { cleanupCache } from '@/services/tauriCommands';
import FolderPicker from '@/components/welcome/FolderPicker.vue';

const ImageViewer = defineAsyncComponent(() => import('@/components/viewer/ImageViewer.vue'));
const NavigationBar = defineAsyncComponent(() => import('@/components/viewer/NavigationBar.vue'));
const ThumbnailStrip = defineAsyncComponent(() => import('@/components/viewer/ThumbnailStrip.vue'));
const ActionBar = defineAsyncComponent(() => import('@/components/actions/ActionBar.vue'));
const ToastNotification = defineAsyncComponent(() => import('@/components/actions/ToastNotification.vue'));
const StatusBar = defineAsyncComponent(() => import('@/components/status/StatusBar.vue'));
const FilterGallery = defineAsyncComponent(() => import('@/components/status/FilterGallery.vue'));
const ExifPanel = defineAsyncComponent(() => import('@/components/exif/ExifPanel.vue'));
const SummaryCard = defineAsyncComponent(() => import('@/components/summary/SummaryCard.vue'));
const ArchiveDialog = defineAsyncComponent(() => import('@/components/archive/ArchiveDialog.vue'));

const view = useViewStore();
useKeyboard();

const cullingReady = ref(false);

// Cleanup cache on window close
function handleBeforeUnload() {
  cleanupCache().catch(() => {});
}

onMounted(() => {
  window.addEventListener('beforeunload', handleBeforeUnload);
});

onUnmounted(() => {
  window.removeEventListener('beforeunload', handleBeforeUnload);
});

watch(
  () => view.currentView,
  (v) => {
    if (v === 'culling') {
      cullingReady.value = false;
      requestAnimationFrame(() => {
        cullingReady.value = true;
      });
    }
  }
);
</script>

<template>
  <div class="w-full h-full bg-sift-bg overflow-hidden">
    <!-- Welcome Page -->
    <Transition name="page-fade">
      <FolderPicker v-if="view.currentView === 'welcome'" key="welcome" />

      <!-- Culling Workspace -->
      <div v-else-if="view.currentView === 'culling'" key="culling" class="w-full h-full relative">
        <!-- Skeleton Shell (shows immediately while components load) -->
        <div v-if="!cullingReady" class="culling-skeleton">
          <div class="skel-nav" />
          <div class="skel-viewer">
            <div class="skeleton-pulse skel-image" />
          </div>
          <div class="skel-thumbnails">
            <div v-for="i in 8" :key="i" class="skeleton-pulse skel-thumb" />
          </div>
          <div class="skel-status" />
        </div>

        <!-- Real Content -->
        <template v-if="cullingReady">
          <!-- Navigation Bar -->
          <NavigationBar />

          <!-- Image Viewer (with padding for nav/thumbnail/status bars) -->
          <div class="absolute inset-0 pt-12 pb-[104px]">
            <ImageViewer />
          </div>

          <!-- Action Bar -->
          <ActionBar />

          <!-- Thumbnail Strip -->
          <ThumbnailStrip />

          <!-- Status Bar -->
          <StatusBar />

          <!-- Filter Gallery -->
          <FilterGallery />

          <!-- EXIF Panel -->
          <ExifPanel />

          <!-- Summary Card (shows when all photos reviewed) -->
          <SummaryCard />

          <!-- Archive Dialog -->
          <ArchiveDialog />

          <!-- Toast Notifications -->
          <ToastNotification />
        </template>
      </div>
    </Transition>
  </div>
</template>
