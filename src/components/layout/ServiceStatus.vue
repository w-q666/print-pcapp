<script setup lang="ts">
import { inject, onMounted, onUnmounted, ref, type Ref } from 'vue'
import { Tooltip } from 'ant-design-vue'
import { getPrintServers } from '../../api/print-api'

defineProps<{
  collapsed: boolean
}>()

type Status = 'online' | 'offline' | 'connecting'

const status = inject<Ref<Status>>('serviceStatus', ref<Status>('connecting'))
let timer: ReturnType<typeof setInterval> | null = null

async function checkConnection() {
  try {
    await getPrintServers()
    status.value = 'online'
  } catch {
    status.value = 'offline'
  }
}

const statusText: Record<Status, string> = {
  online: '服务在线',
  offline: '服务离线',
  connecting: '连接中...',
}

onMounted(() => {
  checkConnection()
  timer = setInterval(checkConnection, 10000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<template>
  <Tooltip :title="collapsed ? statusText[status] : ''" placement="right">
    <div class="service-status">
      <span class="status-dot" :class="`status-dot--${status}`" />
      <span v-if="!collapsed" class="status-text">{{ statusText[status] }}</span>
    </div>
  </Tooltip>
</template>

<style scoped>
.service-status {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot--online {
  background: var(--success-color);
  box-shadow: 0 0 6px rgba(82, 196, 26, 0.4);
}

.status-dot--offline {
  background: var(--error-color);
  box-shadow: 0 0 6px rgba(255, 77, 79, 0.4);
}

.status-dot--connecting {
  background: var(--warning-color);
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.status-text {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
}
</style>
