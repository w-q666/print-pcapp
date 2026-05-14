<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

const isMaximized = ref(false)
let unlisten: (() => void) | null = null

onMounted(async () => {
  const appWindow = getCurrentWindow()
  isMaximized.value = await appWindow.isMaximized()
  unlisten = await appWindow.onResized(async () => {
    isMaximized.value = await appWindow.isMaximized()
  })
})

onUnmounted(() => {
  unlisten?.()
})

async function handleMinimize() {
  await getCurrentWindow().minimize()
}

async function handleToggleMaximize() {
  await getCurrentWindow().toggleMaximize()
}

async function handleClose() {
  await getCurrentWindow().close()
}
</script>

<template>
  <div class="titlebar">
    <div class="titlebar-drag" data-tauri-drag-region />
    <div class="titlebar-controls">
      <button class="titlebar-btn" @click="handleMinimize" title="最小化">
        <svg viewBox="0 0 16 16" fill="none">
          <path d="M3 8h10" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </button>
      <button class="titlebar-btn" @click="handleToggleMaximize" :title="isMaximized ? '还原' : '最大化'">
        <svg v-if="!isMaximized" viewBox="0 0 16 16" fill="none">
          <rect x="3" y="3" width="10" height="10" rx="1" stroke="currentColor" stroke-width="1.2" />
        </svg>
        <svg v-else viewBox="0 0 16 16" fill="none">
          <rect x="5" y="2" width="8" height="8" rx="1" stroke="currentColor" stroke-width="1.2" />
          <path d="M3 5v7a1 1 0 001 1h7" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </button>
      <button class="titlebar-btn titlebar-btn--close" @click="handleClose" title="关闭">
        <svg viewBox="0 0 16 16" fill="none">
          <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </button>
    </div>
  </div>
</template>
