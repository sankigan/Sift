<script setup lang="ts">
// ============================================================
// ContextMenu - Generic right-click context menu
// Dark frosted-glass style, auto-adjusts for edge overflow
// ============================================================

import { ref, watch, nextTick } from 'vue';
import { onClickOutside } from '@vueuse/core';
import { useContextMenu } from '@/composables/useContextMenu';

const { visible, x, y, items, hide } = useContextMenu();

const menuRef = ref<HTMLElement | null>(null);
const adjustedX = ref(0);
const adjustedY = ref(0);

// Adjust menu position to prevent overflow
watch(
  () => visible.value,
  async (val) => {
    if (!val) return;
    adjustedX.value = x.value;
    adjustedY.value = y.value;

    await nextTick();
    if (!menuRef.value) return;

    const rect = menuRef.value.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;

    if (adjustedX.value + rect.width > vw - 8) {
      adjustedX.value = vw - rect.width - 8;
    }
    if (adjustedY.value + rect.height > vh - 8) {
      adjustedY.value = vh - rect.height - 8;
    }
  }
);

onClickOutside(menuRef, (event) => {
  // 忽略右键点击：右键时让 show() 自己处理切换，避免与 contextmenu 事件时序冲突
  if ((event as PointerEvent).button === 2) return;
  if (visible.value) {
    hide();
  }
});

function handleAction(action: () => void) {
  action();
  hide();
}
</script>

<template>
  <Teleport to="body">
    <Transition name="ctx-menu">
      <div
        v-if="visible"
        ref="menuRef"
        class="fixed z-[9999] min-w-[180px] py-1.5
               bg-[#1e1e1e]/90 backdrop-blur-xl
               border border-white/[0.08]
               rounded-xl shadow-[0_8px_40px_rgba(0,0,0,0.6)]
               select-none"
        :style="{ left: `${adjustedX}px`, top: `${adjustedY}px` }"
      >
        <template v-for="(item, index) in items" :key="item.id">
          <!-- Separator -->
          <div
            v-if="item.separator && index > 0"
            class="h-px bg-white/[0.06] my-1 mx-2"
          />
          <!-- Menu item -->
          <button
            class="w-full flex items-center gap-2.5 px-3 py-1.5
                   text-[13px] text-white/80
                   hover:bg-white/[0.08] hover:text-white
                   transition-colors duration-100
                   cursor-default"
            @click.stop="handleAction(item.action)"
          >
            <component
              :is="item.icon"
              :size="15"
              class="text-white/40 shrink-0"
            />
            <span>{{ item.label }}</span>
          </button>
        </template>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.ctx-menu-enter-active {
  transition: opacity 0.12s ease-out, transform 0.12s ease-out;
}
.ctx-menu-leave-active {
  transition: opacity 0.08s ease-in, transform 0.08s ease-in;
}
.ctx-menu-enter-from {
  opacity: 0;
  transform: scale(0.95);
}
.ctx-menu-leave-to {
  opacity: 0;
  transform: scale(0.97);
}
</style>
