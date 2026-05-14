<script setup lang="ts">
import { Form, FormItem, Select, SelectOption, InputNumber, Switch, Radio, RadioGroup } from 'ant-design-vue'
import { useSettings } from '../../stores/settings'

const settings = useSettings()

const paperSizes = [
  { value: 'ISO_A3', label: 'A3' },
  { value: 'ISO_A4', label: 'A4' },
  { value: 'ISO_A5', label: 'A5' },
  { value: 'ISO_B5', label: 'B5' },
  { value: 'NA_LETTER', label: 'Letter' },
  { value: 'NA_LEGAL', label: 'Legal' },
]
</script>

<template>
  <div class="print-settings-tab">
    <Form layout="vertical" style="max-width: 480px">
      <FormItem label="默认打印机">
        <Select
          v-model:value="settings.defaultPrinter"
          placeholder="请选择默认打印机"
          allow-clear
        >
          <SelectOption value="">自动选择</SelectOption>
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
</style>
