<script setup lang="ts">
import { onMounted, ref, watch, nextTick } from 'vue'
import { Select, SelectOption, Input, Button, Switch, Tag, Space } from 'ant-design-vue'
import {
  ReloadOutlined, DeleteOutlined,
} from '@ant-design/icons-vue'
import { useSystemLog } from '../../stores/system-log'

const store = useSystemLog()
const autoScroll = ref(true)
const logContainer = ref<HTMLDivElement | null>(null)

const categories = [
  { key: null, label: '全部', color: '' },
  { key: 'service', label: '服务', color: 'blue' },
  { key: 'print', label: '打印', color: 'green' },
  { key: 'upload', label: '上传', color: 'orange' },
  { key: 'system', label: '系统', color: 'purple' },
]

const levels = [
  { value: '', label: '全部级别' },
  { value: 'DEBUG', label: 'DEBUG' },
  { value: 'INFO', label: 'INFO' },
  { value: 'WARN', label: 'WARN' },
  { value: 'ERROR', label: 'ERROR' },
]

const limitOptions = [
  { value: 100, label: '100 行' },
  { value: 500, label: '500 行' },
  { value: 1000, label: '1000 行' },
]

function scrollToBottom() {
  if (autoScroll.value && logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight
  }
}

watch(() => store.logs, () => {
  nextTick(scrollToBottom)
})

function handleSearch() {
  store.fetchLogs()
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleLevelChange(val: any) {
  store.filterLevel = val || null
  store.fetchLogs()
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleLimitChange(val: any) {
  store.displayLimit = val
  store.fetchLogs()
}

function levelColor(level: string): string {
  switch (level) {
    case 'ERROR': return '#ff4d4f'
    case 'WARN': return '#faad14'
    case 'INFO': return '#52c41a'
    case 'DEBUG': return '#8c8c8c'
    default: return '#d9d9d9'
  }
}

function formatLogLine(log: { timestamp: string; level: string; category: string; message: string }): string {
  return `[${log.timestamp}] [${log.level.padEnd(5)}] [${log.category}] ${log.message}`
}

onMounted(() => {
  store.fetchLogs()
})
</script>

<template>
  <div class="system-log-page">
    <div class="log-header">
      <h2 style="margin: 0">系统日志</h2>
      <Space>
        <Tag
          v-for="cat in categories"
          :key="String(cat.key)"
          :color="store.filterCategory === cat.key ? (cat.color || 'default') : undefined"
          style="cursor: pointer; user-select: none"
          @click="store.setCategory(cat.key)"
        >
          {{ cat.label }}
        </Tag>
        <Button @click="store.fetchLogs()" :loading="store.loading">
          <template #icon><ReloadOutlined /></template>
          刷新
        </Button>
      </Space>
    </div>

    <div class="log-filters">
      <Select
        :value="store.filterLevel || ''"
        style="width: 130px"
        @change="handleLevelChange"
      >
        <SelectOption v-for="l in levels" :key="l.value" :value="l.value">
          {{ l.label }}
        </SelectOption>
      </Select>

      <Input
        v-model:value="store.filterKeyword"
        placeholder="搜索关键词..."
        style="width: 220px"
        allow-clear
        @press-enter="handleSearch"
      />

      <Select
        :value="store.displayLimit"
        style="width: 120px"
        @change="handleLimitChange"
      >
        <SelectOption v-for="opt in limitOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </SelectOption>
      </Select>

      <Button type="primary" @click="handleSearch">查询</Button>
      <Button danger @click="store.clearLogs()">
        <template #icon><DeleteOutlined /></template>
        清空日志
      </Button>
    </div>

    <div class="log-viewer-header">
      <span class="log-count">共 {{ store.logs.length }} 条日志</span>
      <Space>
        <span style="font-size: 13px">自动滚动</span>
        <Switch v-model:checked="autoScroll" size="small" />
      </Space>
    </div>

    <div ref="logContainer" class="log-viewer">
      <div
        v-for="log in store.logs"
        :key="log.id"
        class="log-line"
      >
        <span class="log-level-dot" :style="{ background: levelColor(log.level) }" />
        <span class="log-text">{{ formatLogLine(log) }}</span>
      </div>
      <div v-if="store.logs.length === 0 && !store.loading" class="log-empty">
        暂无日志记录
      </div>
    </div>
  </div>
</template>

<style scoped>
.system-log-page {
  padding: 16px 24px;
  display: flex;
  flex-direction: column;
  height: 100%;
}
.log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 12px;
}
.log-filters {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 8px;
}
.log-viewer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}
.log-count {
  font-size: 13px;
  color: #888;
}
.log-viewer {
  flex: 1;
  min-height: 300px;
  max-height: calc(100vh - 320px);
  overflow-y: auto;
  background: #1a1a2e;
  border-radius: 6px;
  padding: 12px 16px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.7;
}
.log-line {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  color: #e0e0e0;
}
.log-level-dot {
  flex-shrink: 0;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-top: 7px;
}
.log-text {
  word-break: break-all;
  white-space: pre-wrap;
}
.log-empty {
  color: #666;
  text-align: center;
  padding: 40px 0;
}
</style>
