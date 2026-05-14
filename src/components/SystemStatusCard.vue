<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Card, Statistic } from 'ant-design-vue'
import { DashboardOutlined } from '@ant-design/icons-vue'
import { invoke } from '@tauri-apps/api/core'

const queueCount = ref(0)
const todayCount = ref(0)
const loading = ref(true)
let timer: ReturnType<typeof setInterval> | null = null

async function fetchCounts() {
  try {
    const [queue, today] = await Promise.all([
      invoke<number>('print_jobs_count_queue'),
      invoke<number>('print_jobs_count_today'),
    ])
    queueCount.value = queue
    todayCount.value = today
  } catch (e) {
    console.warn('Failed to fetch system status:', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchCounts()
  timer = setInterval(fetchCounts, 10000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<template>
  <Card :loading="loading" size="small" class="status-card">
    <template #title>
      <DashboardOutlined style="margin-right: 8px" />系统状态
    </template>
    <div class="status-grid">
      <div class="status-item">
        <Statistic title="排队任务" :value="queueCount" :value-style="{ color: '#fff', fontSize: '28px' }" />
      </div>
      <div class="status-item">
        <Statistic title="今日完成" :value="todayCount" :value-style="{ color: '#fff', fontSize: '28px' }" />
      </div>
    </div>
  </Card>
</template>

<style scoped>
.status-card :deep(.ant-card-body) {
  background: linear-gradient(135deg, #722ed1, #b37feb);
  border-radius: 0 0 8px 8px;
}

.status-grid {
  display: flex;
  gap: 24px;
}

.status-item {
  flex: 1;
}

.status-item :deep(.ant-statistic-title) {
  color: rgba(255, 255, 255, 0.75);
  font-size: 13px;
}
</style>
