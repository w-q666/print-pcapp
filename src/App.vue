<script setup lang="ts">
import { onMounted } from 'vue'
import { notification } from 'ant-design-vue'
import { usePlatform } from './composables/usePlatform'
import { useAppConfig } from './stores/app-config'
import { useSettings } from './stores/settings'
import { getPrintServers } from './api/print-api'
import DesktopLayout from './layouts/DesktopLayout.vue'
import MobileLayout from './layouts/MobileLayout.vue'

const { platform, detect } = usePlatform()
const appConfig = useAppConfig()
const settings = useSettings()

async function checkServiceConnection() {
  try {
    await getPrintServers()
  } catch {
    notification.warning({
      message: '服务连接失败',
      description: '无法连接到打印服务，请检查 Java 服务是否已启动。部分功能可能不可用。',
      duration: 6,
    })
  }
}

onMounted(async () => {
  await detect()
  await appConfig.loadFromStore()
  await settings.loadFromStore()
  checkServiceConnection()
})
</script>

<template>
  <DesktopLayout v-if="platform === 'desktop'" />
  <MobileLayout v-else />
</template>

<style>
html, body {
  margin: 0;
  padding: 0;
  height: 100%;
}

#app {
  height: 100%;
}
</style>
