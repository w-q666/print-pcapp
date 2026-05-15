<script setup lang="ts">
import { reactive, computed, watch } from 'vue'
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
  fileName?: string
  filePath?: string
  fileNames?: string[]
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  submitted: []
}>()

const settings = useSettings()
const { print } = usePrintService()

const isBatch = computed(() => (props.fileNames?.length ?? 0) > 1)

const displayTitle = computed(() =>
  isBatch.value ? `批量打印设置 (${props.fileNames!.length} 个文件)` : '打印设置',
)

const displayFileName = computed(() => {
  if (isBatch.value) return `已选择 ${props.fileNames!.length} 个文件`
  return props.fileName || props.fileNames?.[0] || ''
})

const form = reactive({
  printer: '__auto__',
  paperSize: 'ISO_A4',
  copies: 1,
  color: false,
  direction: 'PORTRAIT' as 'PORTRAIT' | 'LANDSCAPE',
})

const paperOptions = PaperSizes.map(s => ({ label: s, value: s }))

watch(() => props.open, (val) => {
  if (val) {
    form.printer = settings.defaultPrinter || '__auto__'
    form.paperSize = settings.defaultPaperSize || 'ISO_A4'
    form.copies = settings.defaultCopies || 1
    form.color = settings.defaultColor ?? false
    form.direction = settings.defaultDirection || 'PORTRAIT'
  }
})

const submitting = reactive({ value: false })

async function handleSubmit() {
  const names = props.fileNames?.length ? props.fileNames : (props.fileName ? [props.fileName] : [])
  if (names.length === 0) return

  const unsupported = names.filter(n => !getFileType(n))
  if (unsupported.length > 0) {
    message.error(`不支持的文件格式: ${unsupported.join(', ')}`)
    return
  }

  submitting.value = true
  let successCount = 0
  let failCount = 0

  try {
    for (const name of names) {
      try {
        await print({
          fileName: name,
          filePath: name,
          type: getFileType(name)!,
          source: 'blob',
          printer: form.printer,
          copies: form.copies,
          color: form.color,
          paperSize: form.paperSize,
          direction: form.direction,
        })
        successCount++
      } catch {
        failCount++
      }
    }

    if (failCount === 0) {
      message.success(isBatch.value ? `${successCount} 个文件已加入打印队列` : '打印任务已提交')
    } else {
      message.warning(`${successCount} 个成功，${failCount} 个失败`)
    }

    emit('update:open', false)
    emit('submitted')
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
    :title="displayTitle"
    ok-text="开始打印"
    cancel-text="取消"
    :confirm-loading="submitting.value"
    @ok="handleSubmit"
    @cancel="handleCancel"
    :width="520"
  >
    <Form layout="vertical" style="margin-top: 16px">
      <FormItem :label="isBatch ? '打印文件' : '文件名称'">
        <Input :value="displayFileName" disabled />
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
