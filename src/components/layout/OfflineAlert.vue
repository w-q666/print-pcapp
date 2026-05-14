<script setup lang="ts">
import { inject, computed, ref } from 'vue'
import { WarningOutlined, SyncOutlined } from '@ant-design/icons-vue'
import type { Ref } from 'vue'

type Status = 'online' | 'offline' | 'connecting'

const status = inject<Ref<Status>>('serviceStatus', ref('connecting'))

const config = computed(() => {
  switch (status.value) {
    case 'offline':
      return {
        icon: WarningOutlined,
        bg: 'rgba(255, 77, 79, 0.08)',
        border: 'rgba(255, 77, 79, 0.25)',
        color: '#ff4d4f',
        text: '服务离线',
        animate: false,
      }
    case 'connecting':
      return {
        icon: SyncOutlined,
        bg: 'rgba(250, 173, 20, 0.08)',
        border: 'rgba(250, 173, 20, 0.25)',
        color: '#faad14',
        text: '连接中...',
        animate: true,
      }
    default:
      return null
  }
})
</script>

<template>
  <div
    v-if="config"
    class="offline-alert"
    :style="{
      background: config.bg,
      borderColor: config.border,
    }"
  >
    <span class="alert-icon" :class="{ 'alert-icon--spin': config.animate }" :style="{ color: config.color }">
      <component :is="config.icon" />
    </span>
    <span class="alert-text" :style="{ color: config.color }">{{ config.text }}</span>
  </div>
</template>

<style scoped>
.offline-alert {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 4px 6px;
  padding: 8px 12px;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
}

.alert-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  flex-shrink: 0;
}

.alert-icon--spin {
  animation: spin 1.2s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.alert-text {
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
}
</style>