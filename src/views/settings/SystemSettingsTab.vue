<script setup lang="ts">
import { computed } from 'vue'
import {
  Form,
  FormItem,
  Select,
  SelectOption,
  InputNumber,
  Switch,
  Input,
  Divider,
  Typography,
  Space,
} from 'ant-design-vue'
import { useSettings } from '../../stores/settings'
import { useAppConfig } from '../../stores/app-config'
import { validateScanRange, validateDefaultServiceHost } from '../../utils/ip-range'

const settings = useSettings()
const appConfig = useAppConfig()

const logLevels = [
  { value: 'DEBUG', label: 'DEBUG' },
  { value: 'INFO', label: 'INFO' },
  { value: 'WARN', label: 'WARN' },
  { value: 'ERROR', label: 'ERROR' },
]

const hostValidate = computed(() => validateDefaultServiceHost(appConfig.serviceHost))
const scanValidate = computed(() => validateScanRange(appConfig.scanStartIp, appConfig.scanEndIp))

const scanHint = computed(() => {
  if (!scanValidate.value.ok) return scanValidate.value.message ?? ''
  if ((scanValidate.value.count ?? 0) === 0) {
    return '留空时：启动时将根据本机所有非回环 IPv4 网卡合并扫描（每网段 1–254，合计最多 300 个地址）。若服务在其它网段，请填写范围或将「默认服务 IP」设为打印主机。'
  }
  return `共 ${scanValidate.value.count} 个 IP（上限 300），同网段连续`
})
</script>

<template>
  <div class="system-settings-tab">
    <Form layout="vertical" style="max-width: 520px">
      <Typography.Title :level="5" style="margin-top: 0">
        打印服务
      </Typography.Title>

      <FormItem
        label="默认服务 IP"
        :validate-status="hostValidate.ok ? '' : 'error'"
        :help="hostValidate.ok ? '' : hostValidate.message"
      >
        <Space align="center" wrap>
          <Input
            v-model:value="appConfig.serviceHost"
            placeholder="localhost 或 IPv4"
            style="width: 200px"
          />
          <Typography.Text type="secondary">
            : {{ appConfig.servicePort }}（固定）
          </Typography.Text>
        </Space>
      </FormItem>

      <FormItem
        label="局域网扫描范围"
        :validate-status="scanValidate.ok ? '' : 'error'"
        :help="scanValidate.ok ? scanHint : scanValidate.message"
      >
        <Space align="center" wrap>
          <Input
            v-model:value="appConfig.scanStartIp"
            placeholder="起始 IPv4"
            style="width: 160px"
          />
          <Typography.Text type="secondary">~</Typography.Text>
          <Input
            v-model:value="appConfig.scanEndIp"
            placeholder="结束 IPv4"
            style="width: 160px"
          />
        </Space>
      </FormItem>

      <Divider style="margin: 16px 0" />

      <Typography.Title :level="5">
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
