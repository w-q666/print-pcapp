<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
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
  <div class="status-card">
    <div class="status-row">
      <div class="status-item">
        <span class="status-value">{{ loading ? '-' : queueCount }}</span>
        <span class="status-label">排队</span>
      </div>
      <div class="status-divider" />
      <div class="status-item">
        <span class="status-value">{{ loading ? '-' : todayCount }}</span>
        <span class="status-label">今日完成</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.status-card {
  background: linear-gradient(135deg, #722ed1, #b37feb);
  border-radius: 8px;
  padding: 14px 10px;
  text-align: center;
}

.status-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0;
}

.status-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.status-value {
  font-size: 22px;
  font-weight: 700;
  color: #fff;
  line-height: 1.2;
}

.status-label {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.7);
  margin-top: 2px;
}

.status-divider {
  width: 1px;
  height: 32px;
  background: rgba(255, 255, 255, 0.25);
}
</style>
