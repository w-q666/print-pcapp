<script setup lang="ts">
import { reactive, watch } from 'vue'
import {
  Modal, Form, FormItem, InputNumber, Switch, RadioGroup, RadioButton, Select, Input, message,
} from 'ant-design-vue'
import { useSettings } from '../../stores/settings'
import { usePrintService } from '../../composables/usePrintService'
import { getFileType } from '../../utils/file-types'
import { PaperSizes } from '../../api/types'
import PrinterSelector from './PrinterSelector.vue'

const props = defineProps<{
  open: boolean
  fileName: string
  filePath: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  submitted: []
}>()

const settings = useSettings()
const { print } = usePrintService()

const form = reactive({
  printer: '',
  paperSize: 'ISO_A4',
  copies: 1,
  color: false,
  direction: 'PORTRAIT' as 'PORTRAIT' | 'LANDSCAPE',
})

const paperOptions = PaperSizes.map(s => ({ label: s, value: s }))

watch(() => props.open, (val) => {
  if (val) {
    form.printer = settings.defaultPrinter || ''
    form.paperSize = settings.defaultPaperSize || 'ISO_A4'
    form.copies = settings.defaultCopies || 1
    form.color = settings.defaultColor ?? false
    form.direction = settings.defaultDirection || 'PORTRAIT'
  }
})

const submitting = reactive({ value: false })

async function handleSubmit() {
  const fileType = getFileType(props.fileName)
  if (!fileType) {
    message.error('不支持的文件格式')
    return
  }

  submitting.value = true
  try {
    await print({
      fileName: props.fileName,
      filePath: props.filePath,
      type: fileType,
      source: 'blob',
      printer: form.printer,
      copies: form.copies,
      color: form.color,
      paperSize: form.paperSize,
      direction: form.direction,
    })
    message.success('打印任务已提交')
    emit('update:open', false)
    emit('submitted')
  } catch (e: unknown) {
    message.error(e instanceof Error ? e.message : '打印失败')
  } finally {
    submitting.value = false
  }
}

function handleCancel() {
  emit('update:open', false)
}
</script>

<template>
  <Modal
    :open="open"
    title="打印设置"
    ok-text="开始打印"
    cancel-text="取消"
    :confirm-loading="submitting.value"
    @ok="handleSubmit"
    @cancel="handleCancel"
    :width="520"
  >
    <Form layout="vertical" style="margin-top: 16px">
      <FormItem label="文件名称">
        <Input :value="fileName" disabled />
      </FormItem>
      <FormItem label="打印机">
        <PrinterSelector v-model="form.printer" />
      </FormItem>
      <FormItem label="纸张大小">
        <Select
          v-model:value="form.paperSize"
          :options="paperOptions"
          placeholder="请选择纸张大小"
        />
      </FormItem>
      <FormItem label="打印份数">
        <InputNumber v-model:value="form.copies" :min="1" :max="99" style="width: 120px" />
      </FormItem>
      <FormItem label="彩色打印">
        <Switch v-model:checked="form.color" />
      </FormItem>
      <FormItem label="打印方向">
        <RadioGroup v-model:value="form.direction">
          <RadioButton value="PORTRAIT">纵向</RadioButton>
          <RadioButton value="LANDSCAPE">横向</RadioButton>
        </RadioGroup>
      </FormItem>
    </Form>
  </Modal>
</template>
