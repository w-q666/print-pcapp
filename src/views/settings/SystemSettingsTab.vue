<script setup lang="ts">
import {
  Form,
  FormItem,
  Select,
  SelectOption,
  InputNumber,
  Switch,
  Typography,
} from 'ant-design-vue'
import { useSettings } from '../../stores/settings'

const settings = useSettings()

const logLevels = [
  { value: 'DEBUG', label: 'DEBUG' },
  { value: 'INFO', label: 'INFO' },
  { value: 'WARN', label: 'WARN' },
  { value: 'ERROR', label: 'ERROR' },
]
</script>

<template>
  <div class="system-settings-tab">
    <Form layout="vertical" style="max-width: 520px">
      <Typography.Title :level="5" style="margin-top: 0">
        其他设置
      </Typography.Title>

      <FormItem label="LAN 服务端口">
        <InputNumber
          v-model:value="settings.lanPort"
          :min="1024"
          :max="65535"
          style="width: 160px"
        />
      </FormItem>

      <FormItem label="日志级别">
        <Select v-model:value="settings.logLevel" style="width: 160px">
          <SelectOption
            v-for="l in logLevels"
            :key="l.value"
            :value="l.value"
          >
            {{ l.label }}
          </SelectOption>
        </Select>
      </FormItem>

      <FormItem label="开机自启动">
        <Switch v-model:checked="settings.autoStart" />
      </FormItem>
    </Form>
  </div>
</template>

<style scoped>
.system-settings-tab {
  padding: 8px 0;
}
</style>
