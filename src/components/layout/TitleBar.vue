<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { storeToRefs } from 'pinia'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Button, Segmented } from 'ant-design-vue'
import { MinusOutlined, BorderOutlined, SwitcherOutlined, CloseOutlined } from '@ant-design/icons-vue'
import { useSettingsNav } from '../../stores/settings-nav'

const route = useRoute()
const settingsNav = useSettingsNav()
const { activeKey } = storeToRefs(settingsNav)

const pageTitle = computed(() => {
  const t = route.meta?.title
  return typeof t === 'string' ? t : ''
})

const isSettingsRoute = computed(() => route.name === 'settings')

const settingsSegmentedOptions = [
  { label: '打印服务', value: 'printService' },
  { label: '文件格式', value: 'fileFormat' },
  { label: '打印设置', value: 'printSettings' },
  { label: '系统设置', value: 'systemSettings' },
]

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
  <div class="titlebar" :class="{ 'titlebar--settings': isSettingsRoute }">
    <div class="titlebar-main-row">
      <div class="titlebar-drag-zone" data-tauri-drag-region>
        <span class="titlebar-app">网络打印服务</span>
        <template v-if="pageTitle">
          <span class="titlebar-sep" aria-hidden="true">|</span>
          <span class="titlebar-page-title" :title="pageTitle">{{ pageTitle }}</span>
        </template>
      </div>
      <div id="titlebar-page-actions" class="titlebar-page-actions" />
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
    <div v-if="isSettingsRoute" class="titlebar-settings-row">
      <Segmented
        v-model:value="activeKey"
        block
        size="small"
        :options="settingsSegmentedOptions"
        class="titlebar-settings-segmented"
      />
    </div>
  </div>
</template>
