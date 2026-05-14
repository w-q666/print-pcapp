<script setup lang="ts">
import { Button, Popconfirm } from 'ant-design-vue'
import { EyeOutlined, DeleteOutlined } from '@ant-design/icons-vue'
import FileIcon from './FileIcon.vue'

defineProps<{ fileName: string }>()

const emit = defineEmits<{
  preview: [name: string]
  delete: [name: string]
}>()
</script>

<template>
  <div class="file-list-item">
    <div class="file-info">
      <FileIcon :file-name="fileName" />
      <span class="file-name">{{ fileName }}</span>
    </div>
    <div class="file-actions">
      <Button type="text" size="small" aria-label="预览" @click="emit('preview', fileName)">
        <template #icon><EyeOutlined /></template>
      </Button>
      <Popconfirm
        title="确定删除此文件？"
        ok-text="删除"
        cancel-text="取消"
        @confirm="emit('delete', fileName)"
      >
        <Button type="text" size="small" danger aria-label="删除">
          <template #icon><DeleteOutlined /></template>
        </Button>
      </Popconfirm>
    </div>
  </div>
</template>

<style scoped>
.file-list-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: 6px;
  transition: background-color 0.2s;
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

.file-name {
  font-size: 14px;
  color: rgba(0, 0, 0, 0.85);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}
</style>
