<script setup lang="ts">
import { onMounted, computed } from 'vue'
import {
  Form,
  FormItem,
  Select,
  SelectOption,
  InputNumber,
  Switch,
  Radio,
  RadioGroup,
  Button,
  Typography,
} from 'ant-design-vue'
import { ReloadOutlined } from '@ant-design/icons-vue'
import { useSettings } from '../../stores/settings'
import { usePrinterList } from '../../stores/printer-list'

const settings = useSettings()
const printerList = usePrinterList()

const paperSizes = [
  { value: 'ISO_A3', label: 'A3' },
  { value: 'ISO_A4', label: 'A4' },
  { value: 'ISO_A5', label: 'A5' },
  { value: 'ISO_B5', label: 'B5' },
  { value: 'NA_LETTER', label: 'Letter' },
  { value: 'NA_LEGAL', label: 'Legal' },
]

const blacklistOptions = computed(() =>
  printerList.allPrinters.map((p) => ({ label: p, value: p })),
)

onMounted(async () => {
  if (printerList.allPrinters.length === 0) {
    await printerList.refresh()
  }
})
</script>

<template>
  <div class="print-settings-tab">
    <Form layout="vertical" style="max-width: 560px">
      <FormItem label="从列表中隐藏的打印机">
        <div class="blacklist-row">
          <Select
            v-model:value="settings.printerBlacklist"
            mode="multiple"
            :options="blacklistOptions"
            placeholder="先刷新打印机列表，再选择要隐藏的项"
            style="flex: 1; min-width: 220px"
            :max-tag-count="4"
            allow-clear
          />
          <Button :loading="printerList.loading" @click="printerList.refresh()">
            <template #icon><ReloadOutlined /></template>
            刷新列表
          </Button>
        </div>
        <Typography.Text type="secondary" style="font-size: 12px; display: block; margin-top: 6px">
          隐藏后不在侧边栏与打印对话框中显示。「自动分配」仅在未被隐藏的打印机中随机选择。
        </Typography.Text>
      </FormItem>

      <FormItem label="默认打印机">
        <Select
          v-model:value="settings.defaultPrinter"
          placeholder="请选择默认打印机"
          allow-clear
        >
          <SelectOption value="">自动选择</SelectOption>
          <SelectOption v-for="p in printerList.visiblePrinters" :key="p" :value="p">
            {{ p }}
          </SelectOption>
        </Select>
      </FormItem>

      <FormItem label="纸张大小">
        <Select v-model:value="settings.defaultPaperSize">
          <SelectOption
            v-for="ps in paperSizes"
            :key="ps.value"
            :value="ps.value"
          >
            {{ ps.label }}
          </SelectOption>
        </Select>
      </FormItem>

      <FormItem label="打印份数">
        <InputNumber
          v-model:value="settings.defaultCopies"
          :min="1"
          :max="99"
          style="width: 120px"
        />
      </FormItem>

      <FormItem label="默认彩色打印">
        <Switch v-model:checked="settings.defaultColor" />
      </FormItem>

      <FormItem label="打印方向">
        <RadioGroup v-model:value="settings.defaultDirection">
          <Radio value="PORTRAIT">纵向</Radio>
          <Radio value="LANDSCAPE">横向</Radio>
        </RadioGroup>
      </FormItem>
    </Form>
  </div>
</template>

<style scoped>
.print-settings-tab {
  padding: 8px 0;
}

.blacklist-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}
</style>
