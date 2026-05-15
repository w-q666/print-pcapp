<script setup lang="ts">
import { onMounted } from 'vue'
import { notification, ConfigProvider } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { usePlatform } from './composables/usePlatform'
import { useAppConfig } from './stores/app-config'
import { useSettings } from './stores/settings'
import { getPrintServers } from './api/print-api'
import { setBaseURL } from './api/http-client'
import { inferRangeFromLocalIp } from './utils/ip-range'
import DesktopLayout from './layouts/DesktopLayout.vue'
import MobileLayout from './layouts/MobileLayout.vue'

interface DiscoverScanResult {
  foundHost: string | null
  scannedCount: number
  elapsedMs: number
}

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

async function bootstrapPrintDiscovery() {
  if (
    !appConfig.scanRangeInferredOnce
    && !appConfig.scanStartIp.trim()
    && !appConfig.scanEndIp.trim()
  ) {
    try {
      const ip = await invoke<string>('get_network_local_ip')
      const range = inferRangeFromLocalIp(ip)
      if (range) {
        appConfig.scanStartIp = range.start
        appConfig.scanEndIp = range.end
        appConfig.scanRangeInferredOnce = true
        await appConfig.saveToStore()
      }
    } catch {
      /* 无本机 IP 时仅探测默认服务 */
    }
  }

  setBaseURL(`http://${appConfig.serviceHost}:${appConfig.servicePort}`)

  let result: DiscoverScanResult | null = null
  try {
    result = await invoke<DiscoverScanResult>('discover_service', {
      config: {
        defaultHost: appConfig.serviceHost,
        port: appConfig.servicePort,
        startIp: appConfig.scanStartIp.trim() || null,
        endIp: appConfig.scanEndIp.trim() || null,
      },
    })
  } catch {
    result = null
  }

  if (result?.foundHost) {
    appConfig.serviceHost = result.foundHost
    setBaseURL(`http://${appConfig.serviceHost}:${appConfig.servicePort}`)
    await appConfig.saveToStore()
    return
  }

  await checkServiceConnection()
}

onMounted(async () => {
  await detect()
  await appConfig.loadFromStore()
  await settings.loadFromStore()
  await bootstrapPrintDiscovery()
})
</script>

<template>
  <ConfigProvider>
    <DesktopLayout v-if="platform === 'desktop'" />
    <MobileLayout v-else />
  </ConfigProvider>
</template>
