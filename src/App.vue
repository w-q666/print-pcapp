<script setup lang="ts">
import { onMounted, onUnmounted, ref, nextTick } from 'vue'
import { notification, ConfigProvider, Spin, Progress, Typography } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { usePlatform } from './composables/usePlatform'
import { useAppConfig } from './stores/app-config'
import { useSettings } from './stores/settings'
import { getPrintServers } from './api/print-api'
import { setBaseURL } from './api/http-client'
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

const discovering = ref(false)
const discoverProgress = ref(0)
const discoverDetail = ref('正在扫描本机网卡对应网段及配置的 IP 范围（最多 10 秒）…')

let progressTimer: ReturnType<typeof setInterval> | null = null

function startDiscoverProgressUi() {
  discoverProgress.value = 0
  discoverDetail.value = '正在扫描本机网卡对应网段及配置的 IP 范围（最多 10 秒）…'
  const t0 = Date.now()
  progressTimer = setInterval(() => {
    const elapsed = Date.now() - t0
    discoverProgress.value = Math.min(95, Math.floor((elapsed / 10000) * 100))
  }, 120)
}

function stopDiscoverProgressUi(done: boolean) {
  if (progressTimer) {
    clearInterval(progressTimer)
    progressTimer = null
  }
  discoverProgress.value = done ? 100 : discoverProgress.value
}

onUnmounted(() => {
  stopDiscoverProgressUi(false)
})

async function checkServiceConnection() {
  try {
    await getPrintServers()
  } catch {
    notification.warning({
      message: '服务连接失败',
      description: '无法连接到打印服务。若服务在其它网段，请在「系统配置 → 系统设置」将默认服务 IP 设为该主机，或填写正确的扫描范围。',
      duration: 8,
    })
  }
}

async function bootstrapPrintDiscovery() {
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
    discoverDetail.value = `已发现服务：${result.foundHost}（本次探测 ${result.scannedCount} 次，耗时 ${result.elapsedMs} ms）`
    return
  }

  if (result) {
    discoverDetail.value = `未发现可用服务（已探测 ${result.scannedCount} 个地址，耗时 ${result.elapsedMs} ms）。请确认 Java 服务已启动，或将「默认服务 IP」直接设为打印主机（如 192.168.137.29）。`
  } else {
    discoverDetail.value = '发现过程出错，将尝试连接当前默认地址。'
  }

  await checkServiceConnection()
}

onMounted(async () => {
  await detect()
  await appConfig.loadFromStore()
  await settings.loadFromStore()

  discovering.value = true
  startDiscoverProgressUi()
  await nextTick()
  try {
    await bootstrapPrintDiscovery()
  } finally {
    stopDiscoverProgressUi(true)
    await new Promise((r) => setTimeout(r, 320))
    discovering.value = false
  }
})
</script>

<template>
  <ConfigProvider>
    <DesktopLayout v-if="platform === 'desktop'" />
    <MobileLayout v-else />
  </ConfigProvider>

  <Teleport to="body">
    <div v-if="discovering" class="print-discovery-mask" role="dialog" aria-live="polite" aria-busy="true">
      <Spin size="large" tip="正在局域网发现 Java 打印服务" />
      <Progress
        :percent="discoverProgress"
        :status="discoverProgress >= 100 ? 'success' : 'active'"
        :stroke-color="{ from: '#722ed1', to: '#1890ff' }"
        style="width: min(360px, 80vw); margin-top: 20px"
      />
      <Typography.Paragraph type="secondary" style="margin-top: 14px; max-width: 420px; text-align: center">
        {{ discoverDetail }}
      </Typography.Paragraph>
    </div>
  </Teleport>
</template>

<style>
.print-discovery-mask {
  position: fixed;
  inset: 0;
  z-index: 100000;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.92);
  backdrop-filter: blur(2px);
}
</style>
