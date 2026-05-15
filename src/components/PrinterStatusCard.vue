<script setup lang="ts">
import { onMounted } from 'vue'
import { Tag, Button, Empty } from 'ant-design-vue'
import { ReloadOutlined } from '@ant-design/icons-vue'
import { usePrinterList } from '../stores/printer-list'

const printerList = usePrinterList()

onMounted(() => {
  if (printerList.printers.length === 0) {
    printerList.refresh()
  }
})
</script>

<template>
  <div class="printer-card">
    <div class="printer-header">
      <span class="printer-title">打印机</span>
      <Button type="text" size="small" :loading="printerList.loading" @click="printerList.refresh()">
        <template #icon><ReloadOutlined /></template>
      </Button>
    </div>

    <Empty v-if="printerList.printers.length === 0 && !printerList.loading" :image="Empty.PRESENTED_IMAGE_SIMPLE" :description="printerList.allPrinters.length > 0 ? '全部打印机已被隐藏，请到系统配置调整' : '未发现打印机'" :image-style="{ height: '30px' }" />

    <div v-else class="printer-list">
      <div v-for="name in printerList.printers" :key="name" class="printer-item">
        <span class="printer-dot" :class="{ active: name === printerList.effectiveDefaultPrinter }" />
        <span class="printer-name">{{ name }}</span>
        <Tag :color="name === printerList.effectiveDefaultPrinter ? 'blue' : 'default'" class="printer-tag">
          {{ name === printerList.effectiveDefaultPrinter ? '默认' : '就绪' }}
        </Tag>
      </div>
    </div>
  </div>
</template>

<style scoped>
.printer-card {
  border: 1px solid #f0f0f0;
  border-radius: 8px;
  padding: 10px 12px;
  background: #fff;
  overflow: hidden;
}

.printer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.printer-title {
  font-size: 13px;
  font-weight: 600;
  color: rgba(0, 0, 0, 0.85);
}

.printer-list {
  display: flex;
  flex-direction: column;
}

.printer-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 0;
  border-bottom: 1px solid #f5f5f5;
}

.printer-item:last-child {
  border-bottom: none;
}

.printer-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #d9d9d9;
  flex-shrink: 0;
}

.printer-dot.active {
  background: #52c41a;
}

.printer-name {
  font-size: 12px;
  color: rgba(0, 0, 0, 0.85);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.printer-tag {
  font-size: 10px;
  line-height: 1;
  margin: 0;
}
</style>
