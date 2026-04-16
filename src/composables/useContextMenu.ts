// ============================================================
// useContextMenu - Context menu state and actions for photos
// Platform-aware labels, re-right-click switches context directly
// ============================================================

import { ref, nextTick, type Component, markRaw } from 'vue';
import { FolderOpen, Copy, FileImage } from 'lucide-vue-next';
import { showInFolder, copyImageToClipboard } from '@/services/tauriCommands';
import type { PhotoPair } from '@/types';

export interface MenuItem {
  id: string;
  label: string;
  icon: Component;
  action: () => void;
  separator?: boolean;
}

const visible = ref(false);
const x = ref(0);
const y = ref(0);
const items = ref<MenuItem[]>([]);

// Detect platform for label text
const isMac = navigator.userAgent.includes('Mac');
const finderLabel = isMac ? '在 Finder 中显示' : '在资源管理器中显示';
const finderRawLabel = isMac ? '在 Finder 中显示 RAW 文件' : '在资源管理器中显示 RAW 文件';
const finderJpgLabel = isMac ? '在 Finder 中显示预览 JPG' : '在资源管理器中显示预览 JPG';

function show(event: MouseEvent, pair: PhotoPair) {
  // 先同步关闭（跳过 Transition leave 动画），再重新打开
  // 解决连续右键时 Transition 内部状态冲突导致菜单不显示的问题
  visible.value = false;

  const menuItems: MenuItem[] = [];

  const primaryPath = pair.source === 'rawPreview' && pair.rawPath
    ? pair.rawPath
    : pair.jpgPath;

  menuItems.push({
    id: 'show-in-finder',
    label: finderLabel,
    icon: markRaw(FolderOpen),
    action: () => {
      showInFolder(primaryPath).catch(console.error);
    },
  });

  if (pair.source === 'rawPreview' && pair.rawPath && pair.jpgPath) {
    menuItems.push({
      id: 'show-jpg-in-finder',
      label: finderJpgLabel,
      icon: markRaw(FileImage),
      action: () => {
        showInFolder(pair.jpgPath).catch(console.error);
      },
    });
  } else if (pair.source !== 'rawPreview' && pair.rawPath) {
    menuItems.push({
      id: 'show-raw-in-finder',
      label: finderRawLabel,
      icon: markRaw(FileImage),
      action: () => {
        showInFolder(pair.rawPath!).catch(console.error);
      },
    });
  }

  menuItems.push({
    id: 'copy-image',
    label: '复制图片',
    icon: markRaw(Copy),
    separator: true,
    action: () => {
      copyImageToClipboard(pair.jpgPath).catch(console.error);
    },
  });

  items.value = menuItems;
  x.value = event.clientX;
  y.value = event.clientY;

  // 使用 nextTick 确保 DOM 先响应 visible=false 的变更，再设为 true
  nextTick(() => {
    visible.value = true;
  });
}

function hide() {
  visible.value = false;
}

export function useContextMenu() {
  return {
    visible,
    x,
    y,
    items,
    show,
    hide,
  };
}
