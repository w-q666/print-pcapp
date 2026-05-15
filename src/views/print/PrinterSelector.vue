<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { Select, Button } from 'ant-design-vue'
import { ReloadOutlined } from '@ant-design/icons-vue'
import { usePrinterList } from '../../stores/printer-list'

const AUTO_ASSIGN = '__auto__'

const props = defineProps<{ modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

const printerList = usePrinterList()

const options = computed(() => [
  { label: '🔀 自动分配', value: AUTO_ASSIGN },
  ...printerList.printers.map(p => ({ label: p, value: p })),
])

function handleChange(value: any) {
  emit('update:modelValue', String(value ?? AUTO_ASSIGN))
}

onMounted(async () => {
  if (printerList.printers.length === 0) {
    await printerList.refresh()
  }
  if (!props.modelValue) {
    emit('update:modelValue', AUTO_ASSIGN)
  }
})
</script>

<template>
  <div class="printer-selector">
    <Select
      :value="modelValue || AUTO_ASSIGN"
      placeholder="请选择打印机"
      style="flex: 1"
      :loading="printerList.loading"
      :options="options"
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
