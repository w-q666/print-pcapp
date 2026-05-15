<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Button } from 'ant-design-vue'
import { MinusOutlined, BorderOutlined, SwitcherOutlined, CloseOutlined } from '@ant-design/icons-vue'

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
  await getCurrentWindow().hide()
}
</script>

<template>
  <div class="titlebar">
    <div class="titlebar-drag" data-tauri-drag-region />
    <div class="titlebar-controls">
      <Button type="text" class="titlebar-btn" @click="handleMinimize" title="最小化">
        <template #icon><MinusOutlined /></template>
      </Button>
      <Button type="text" class="titlebar-btn" @click="handleToggleMaximize" :title="isMaximized ? '还原' : '最大化'">
        <template #icon>
          <BorderOutlined v-if="!isMaximized" />
          <SwitcherOutlined v-else />
        </template>
      </Button>
      <Button type="text" class="titlebar-btn titlebar-btn--close" @click="handleClose" title="关闭">
        <template #icon><CloseOutlined /></template>
      </Button>
    </div>
  </div>
</template>
