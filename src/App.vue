<script setup lang="ts">
// ============================================================
// App.vue - Root component with page routing
// ============================================================

import { useViewStore } from '@/stores/viewStore'
import { useKeyboard } from '@/composables/useKeyboard'
import FolderPicker from '@/components/welcome/FolderPicker.vue'
import ImageViewer from '@/components/viewer/ImageViewer.vue'
import NavigationBar from '@/components/viewer/NavigationBar.vue'
import ActionBar from '@/components/actions/ActionBar.vue'
import ToastNotification from '@/components/actions/ToastNotification.vue'
import StatusBar from '@/components/status/StatusBar.vue'
import ExifPanel from '@/components/exif/ExifPanel.vue'
import SummaryCard from '@/components/summary/SummaryCard.vue'
import ArchiveDialog from '@/components/archive/ArchiveDialog.vue'

const view = useViewStore()
useKeyboard()
</script>

<template>
  <div class="w-full h-full bg-sift-bg overflow-hidden">
    <!-- Welcome Page -->
    <Transition name="page-fade" mode="out-in">
      <FolderPicker v-if="view.currentView === 'welcome'" key="welcome" />

      <!-- Culling Workspace -->
      <div v-else-if="view.currentView === 'culling'" key="culling" class="w-full h-full relative">
        <!-- Navigation Bar -->
        <NavigationBar />

        <!-- Image Viewer (with padding for nav/status bars) -->
        <div class="absolute inset-0 pt-12 pb-10">
          <ImageViewer />
        </div>

        <!-- Action Bar -->
        <ActionBar />

        <!-- Status Bar -->
        <StatusBar />

        <!-- EXIF Panel -->
        <ExifPanel />

        <!-- Summary Card (shows when all photos reviewed) -->
        <SummaryCard />

        <!-- Archive Dialog -->
        <ArchiveDialog />

        <!-- Toast Notifications -->
        <ToastNotification />
      </div>
    </Transition>
  </div>
</template>
