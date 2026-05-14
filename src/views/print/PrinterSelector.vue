<script setup lang="ts">
import { onMounted } from 'vue'
import { Select, Button } from 'ant-design-vue'
import { ReloadOutlined } from '@ant-design/icons-vue'
import { usePrinterList } from '../../stores/printer-list'

const props = defineProps<{ modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

const printerList = usePrinterList()

function handleChange(value: any) {
  emit('update:modelValue', String(value ?? ''))
}

onMounted(async () => {
  if (printerList.printers.length === 0) {
    await printerList.refresh()
    if (!props.modelValue && printerList.defaultPrinter) {
      emit('update:modelValue', printerList.defaultPrinter)
    }
  }
})
</script>

<template>
  <div class="printer-selector">
    <Select
      :value="modelValue || undefined"
      placeholder="请选择打印机"
      style="flex: 1"
      :loading="printerList.loading"
      :options="printerList.printers.map(p => ({ label: p, value: p }))"
      @change="handleChange"
    />
    <Button
      :loading="printerList.loading"
      @click="printerList.refresh()"
      title="刷新打印机列表"
    >
      <template #icon><ReloadOutlined /></template>
    </Button>
  </div>
</template>

<style scoped>
.printer-selector {
  display: flex;
  gap: 8px;
  align-items: center;
}
</style>
