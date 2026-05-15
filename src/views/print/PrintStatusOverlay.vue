<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { Spin } from 'ant-design-vue'
import {
  CheckCircleOutlined, CloseCircleOutlined, PrinterOutlined,
  LoadingOutlined, WarningOutlined,
} from '@ant-design/icons-vue'
import { usePrintTask } from '../../stores/print-task'

const printTask = usePrintTask()
const visible = ref(false)
let hideTimer: ReturnType<typeof setTimeout> | null = null

const statusConfig = computed(() => {
  switch (printTask.currentStatus) {
    case 'queued': return { icon: LoadingOutlined, color: '#722ed1', text: '已加入打印队列' }
    case 'connecting': return { icon: LoadingOutlined, color: '#1890ff', text: '正在连接...' }
    case 'preparing': return { icon: LoadingOutlined, color: '#1890ff', text: '准备打印...' }
    case 'printing': return { icon: PrinterOutlined, color: '#1890ff', text: '正在打印...' }
    case 'data_sent': return { icon: LoadingOutlined, color: '#1890ff', text: '数据已发送...' }
    case 'done': return { icon: CheckCircleOutlined, color: '#52c41a', text: '打印完成' }
    case 'error': return { icon: CloseCircleOutlined, color: '#ff4d4f', text: '打印错误' }
    case 'failed': return { icon: CloseCircleOutlined, color: '#ff4d4f', text: '打印失败' }
    case 'cancelled': return { icon: WarningOutlined, color: '#faad14', text: '已取消' }
    case 'needs_attention': return { icon: WarningOutlined, color: '#faad14', text: '需要处理' }
    default: return { icon: PrinterOutlined, color: '#1890ff', text: '' }
  }
})

const isSpinning = computed(() =>
  ['queued', 'connecting', 'preparing', 'printing', 'data_sent'].includes(printTask.currentStatus)
)

watch(() => printTask.currentStatus, (status) => {
  if (hideTimer) { clearTimeout(hideTimer); hideTimer = null }

  if (status === 'idle') {
    visible.value = false
    return
  }

  visible.value = true

  if (['done', 'failed', 'error', 'cancelled'].includes(status)) {
    hideTimer = setTimeout(() => {
      visible.value = false
      printTask.reset()
    }, 3000)
  }
})
</script>

<template>
  <Transition name="overlay-fade">
    <div v-if="visible" class="print-status-overlay" :style="{ borderLeftColor: statusConfig.color }">
      <div class="overlay-icon">
        <Spin v-if="isSpinning" :indicator="statusConfig.icon" />
        <component v-else :is="statusConfig.icon" :style="{ color: statusConfig.color, fontSize: '24px' }" />
      </div>
      <div class="overlay-content">
        <div class="overlay-job">{{ printTask.currentJobName }}</div>
        <div class="overlay-text" :style="{ color: statusConfig.color }">{{ statusConfig.text }}</div>
        <div v-if="printTask.statusMessage && ['error', 'failed'].includes(printTask.currentStatus)" class="overlay-msg">
          {{ printTask.statusMessage }}
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.print-status-overlay {
  position: fixed;
  right: 24px;
  bottom: 24px;
  z-index: 1050;
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px 20px;
  min-width: 280px;
  max-width: 380px;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.12);
  border-left: 4px solid #1890ff;
}

.overlay-icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  margin-top: 2px;
}

.overlay-content {
  flex: 1;
  min-width: 0;
}

.overlay-job {
  font-size: 14px;
  font-weight: 500;
  color: rgba(0, 0, 0, 0.85);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-bottom: 4px;
}

.overlay-text {
  font-size: 13px;
}

.overlay-msg {
  font-size: 12px;
  color: rgba(0, 0, 0, 0.45);
  margin-top: 4px;
  word-break: break-all;
}

.overlay-fade-enter-active,
.overlay-fade-leave-active {
  transition: all 0.3s ease;
}

.overlay-fade-enter-from,
.overlay-fade-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
</style>
