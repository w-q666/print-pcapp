<script setup lang="ts">
import { computed } from 'vue'
import { Card, Empty, Tag, Button, Progress } from 'ant-design-vue'
import {
  ReloadOutlined, CloseCircleOutlined,
  CheckCircleOutlined, PrinterOutlined, ClockCircleOutlined,
  LoadingOutlined,
} from '@ant-design/icons-vue'
import BasePage from '../../components/layout/BasePage.vue'
import { usePrintTask } from '../../stores/print-task'
import { usePrintHistory } from '../../stores/print-history'

const printTask = usePrintTask()
const historyStore = usePrintHistory()

const activeJobs = computed(() =>
  historyStore.records.filter(r => ['queued', 'printing'].includes(r.status))
)

const recentJobs = computed(() =>
  historyStore.records
    .filter(r => ['done', 'failed', 'cancelled'].includes(r.status))
    .slice(0, 10)
)

const statusConfig: Record<string, { color: string; label: string; icon: typeof PrinterOutlined }> = {
  queued: { color: 'orange', label: '排队中', icon: ClockCircleOutlined },
  printing: { color: 'blue', label: '打印中', icon: LoadingOutlined },
  done: { color: 'green', label: '已完成', icon: CheckCircleOutlined },
  failed: { color: 'red', label: '失败', icon: CloseCircleOutlined },
  cancelled: { color: 'default', label: '已取消', icon: CloseCircleOutlined },
}

function refresh() {
  historyStore.fetchRecords()
}

refresh()
</script>

<template>
  <BasePage title="打印任务">
    <template #actions>
      <Button size="small" @click="refresh" :loading="historyStore.loading">
        <template #icon><ReloadOutlined /></template>
        刷新
      </Button>
    </template>

    <div class="queue-grid">
      <Card title="当前队列" size="small" class="queue-card">
        <Empty v-if="activeJobs.length === 0 && !printTask.isActive" description="暂无打印任务" />
        <div v-if="printTask.isActive" class="job-item job-item--active">
          <div class="job-info">
            <PrinterOutlined class="job-icon" style="color: var(--primary-color)" />
            <div class="job-detail">
              <div class="job-name">{{ printTask.currentJobName || '当前任务' }}</div>
              <div class="job-status">{{ printTask.statusMessage || printTask.currentStatus }}</div>
            </div>
          </div>
          <Progress :percent="printTask.currentStatus === 'done' ? 100 : 50" :status="printTask.currentStatus === 'error' ? 'exception' : 'active'" size="small" style="width: 120px" />
        </div>
        <div v-for="job in activeJobs" :key="job.id" class="job-item">
          <div class="job-info">
            <component :is="statusConfig[job.status]?.icon || PrinterOutlined" class="job-icon" />
            <div class="job-detail">
              <div class="job-name">{{ job.name }}</div>
              <div class="job-meta">{{ job.printer }} · {{ job.created_at }}</div>
            </div>
          </div>
          <Tag :color="statusConfig[job.status]?.color">{{ statusConfig[job.status]?.label || job.status }}</Tag>
        </div>
      </Card>

      <Card title="最近完成" size="small" class="queue-card">
        <Empty v-if="recentJobs.length === 0" description="暂无记录" />
        <div v-for="job in recentJobs" :key="job.id" class="job-item">
          <div class="job-info">
            <component :is="statusConfig[job.status]?.icon || PrinterOutlined" class="job-icon" :style="{ color: statusConfig[job.status]?.color === 'default' ? 'var(--text-disabled)' : undefined }" />
            <div class="job-detail">
              <div class="job-name">{{ job.name }}</div>
              <div class="job-meta">{{ job.printer }} · {{ job.finished_at || job.created_at }}</div>
            </div>
          </div>
          <Tag :color="statusConfig[job.status]?.color">{{ statusConfig[job.status]?.label || job.status }}</Tag>
        </div>
      </Card>
    </div>
  </BasePage>
</template>

<style scoped>
.queue-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.queue-card :deep(.ant-card-body) {
  padding: 6px 12px;
}

.job-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid var(--border-color);
}

.job-item:last-child {
  border-bottom: none;
}

.job-item--active {
  background: rgba(22, 119, 255, 0.04);
  border-radius: var(--radius-md);
  padding: 8px 10px;
  margin: -2px -2px 4px;
}

.job-info {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  flex: 1;
}

.job-icon {
  font-size: 18px;
  flex-shrink: 0;
  color: var(--text-secondary);
}

.job-detail {
  min-width: 0;
}

.job-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.job-meta, .job-status {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 1px;
}

@media (max-width: 680px) {
  .queue-grid {
    grid-template-columns: 1fr;
  }
}
</style>
