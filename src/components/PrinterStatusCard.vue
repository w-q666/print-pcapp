<script setup lang="ts">
import { onMounted } from 'vue'
import { Card, Tag, Button, Empty } from 'ant-design-vue'
import { PrinterOutlined, ReloadOutlined } from '@ant-design/icons-vue'
import { usePrinterList } from '../stores/printer-list'

const printerList = usePrinterList()

onMounted(() => {
  if (printerList.printers.length === 0) {
    printerList.refresh()
  }
})
</script>

<template>
  <Card size="small">
    <template #title>
      <PrinterOutlined style="margin-right: 8px" />打印机
    </template>
    <template #extra>
      <Button type="text" size="small" :loading="printerList.loading" @click="printerList.refresh()">
        <template #icon><ReloadOutlined /></template>
      </Button>
    </template>

    <Empty v-if="printerList.printers.length === 0 && !printerList.loading" description="未发现打印机" />

    <div v-else class="printer-list">
      <div v-for="name in printerList.printers" :key="name" class="printer-item">
        <span class="printer-name">{{ name }}</span>
        <Tag :color="name === printerList.defaultPrinter ? 'blue' : 'default'">
          {{ name === printerList.defaultPrinter ? '默认' : '就绪' }}
        </Tag>
      </div>
    </div>
  </Card>
</template>

<style scoped>
.printer-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.printer-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
  border-bottom: 1px solid #f0f0f0;
}

.printer-item:last-child {
  border-bottom: none;
}

.printer-name {
  font-size: 13px;
  color: rgba(0, 0, 0, 0.85);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  margin-right: 8px;
}
</style>
