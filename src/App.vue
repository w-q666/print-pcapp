<script setup lang="ts">
import { onMounted } from 'vue'
import { usePlatform } from './composables/usePlatform'
import { useAppConfig } from './stores/app-config'
import DesktopLayout from './layouts/DesktopLayout.vue'
import MobileLayout from './layouts/MobileLayout.vue'

const { platform, detect } = usePlatform()
const appConfig = useAppConfig()

onMounted(async () => {
  await detect()
  await appConfig.loadFromStore()
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
