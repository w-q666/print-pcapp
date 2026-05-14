<script setup lang="ts">
import { computed } from 'vue'
import { Button, Popconfirm } from 'ant-design-vue'
import { EyeOutlined, DeleteOutlined, PrinterOutlined } from '@ant-design/icons-vue'
import FileIcon from './FileIcon.vue'
import { getFileType } from '../utils/file-types'

const props = defineProps<{
  fileName: string
  fileSize?: number
  fileDate?: string
}>()

const emit = defineEmits<{
  preview: [name: string]
  delete: [name: string]
  print: [name: string]
}>()

const canPreview = computed(() => getFileType(props.fileName) !== null)

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<template>
  <div class="file-list-item">
    <div class="file-info">
      <FileIcon :file-name="fileName" />
      <div class="file-text">
        <span class="file-name">{{ fileName }}</span>
        <span class="file-meta">
          <template v-if="fileSize !== undefined">{{ formatSize(fileSize) }}</template>
          <template v-if="fileDate"> · {{ fileDate }}</template>
        </span>
      </div>
    </div>
    <div class="file-actions">
      <Button v-if="canPreview" type="text" size="small" @click="emit('preview', fileName)">
        <template #icon><EyeOutlined /></template>
        预览
      </Button>
      <Popconfirm
        title="确定删除此文件？"
        ok-text="删除"
        cancel-text="取消"
        @confirm="emit('delete', fileName)"
      >
        <Button type="text" size="small" danger>
          <template #icon><DeleteOutlined /></template>
          删除
        </Button>
      </Popconfirm>
      <Button
        type="primary"
        size="small"
        ghost
        @click="emit('print', fileName)"
      >
        <template #icon><PrinterOutlined /></template>
        打印
      </Button>
    </div>
  </div>
</template>

<style scoped>
.file-list-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid #f5f5f5;
  transition: background-color 0.2s;
}

.file-list-item:last-child {
  border-bottom: none;
}

.file-list-item:hover {
  background-color: var(--ant-control-item-bg-hover, #f5f5f5);
}

.file-info {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  flex: 1;
}

.file-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.file-name {
  font-size: 13px;
  font-weight: 500;
  color: rgba(0, 0, 0, 0.85);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-meta {
  font-size: 11px;
  color: rgba(0, 0, 0, 0.45);
  margin-top: 1px;
}

.file-actions {
  display: flex;
  gap: 4px;
  align-items: center;
  flex-shrink: 0;
  margin-left: 12px;
}
</style>
