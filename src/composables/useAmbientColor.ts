// ============================================================
// Sift - Ambient Color Composable
// Manages dominant color for background glow effect
// ============================================================

import { ref, watch, computed } from 'vue'
import { useSessionStore } from '@/stores/sessionStore'

export function useAmbientColor() {
  const session = useSessionStore()

  const currentColor = ref('#3B82F6') // Default blue accent
  const previousColor = ref('#3B82F6')

  const ambientStyle = computed(() => ({
    background: `radial-gradient(ellipse at center, ${currentColor.value}26 0%, transparent 70%)`,
    filter: 'blur(80px)',
  }))

  // Watch for current pair changes
  watch(
    () => session.currentPair,
    (pair) => {
      if (pair?.dominantColor) {
        previousColor.value = currentColor.value
        currentColor.value = pair.dominantColor
      }
    },
    { immediate: true }
  )

  return {
    currentColor,
    ambientStyle,
  }
}
