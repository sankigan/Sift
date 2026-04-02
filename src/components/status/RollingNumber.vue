<script setup lang="ts">
// ============================================================
// RollingNumber - Digit-by-digit rolling animation
// ============================================================

import { computed, watch, ref } from 'vue'

const props = defineProps<{
  value: number
}>()

const digits = computed(() => {
  return String(props.value).split('').map((d, i) => ({
    key: i,
    digit: parseInt(d),
  }))
})
</script>

<template>
  <span class="rolling-number-container font-mono">
    <span
      v-for="(d, index) in digits"
      :key="index"
      class="inline-block relative overflow-hidden"
      style="width: 0.65em; height: 1.25em"
    >
      <span
        class="rolling-digit absolute left-0 top-0 w-full"
        :style="{ transform: `translateY(${-d.digit * 1.25}em)`, transition: 'transform 0.4s cubic-bezier(0.22, 1, 0.36, 1)' }"
      >
        <span
          v-for="n in 10"
          :key="n - 1"
          class="block text-center"
          style="height: 1.25em; line-height: 1.25em"
        >
          {{ n - 1 }}
        </span>
      </span>
    </span>
  </span>
</template>
