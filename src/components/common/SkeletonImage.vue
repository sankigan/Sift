<script setup lang="ts">
// ============================================================
// SkeletonImage - Pulse loading placeholder
// ============================================================

defineProps<{
  width?: string
  height?: string
}>()
</script>

<template>
  <div
    class="skeleton-pulse rounded-lg"
    :style="{
      width: width || '100%',
      height: height || '100%',
    }"
  />
</template>
