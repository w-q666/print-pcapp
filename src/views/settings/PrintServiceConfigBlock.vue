<script setup lang="ts">
import { computed } from 'vue'
import { Form, FormItem, Input, Typography, Space } from 'ant-design-vue'
import { useAppConfig } from '../../stores/app-config'
import { validateScanRange, validateDefaultServiceHost } from '../../utils/ip-range'

const appConfig = useAppConfig()

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
  <div class="print-service-block">
    <Typography.Title :level="5" style="margin-top: 0">
      打印服务（Java 局域网发现）
    </Typography.Title>
    <Typography.Paragraph type="secondary" style="margin-bottom: 12px; font-size: 13px">
      与下方「文件格式」等标签无关；修改后请点击右上角「保存配置」。
    </Typography.Paragraph>

    <Form layout="vertical" style="max-width: 520px">
      <FormItem
        label="默认服务 IP"
        :validate-status="hostValidate.ok ? '' : 'error'"
        :help="hostValidate.ok ? '' : hostValidate.message"
      >
        <Space align="center" wrap>
          <Input
            v-model:value="appConfig.serviceHost"
            placeholder="localhost 或 IPv4，如 192.168.137.29"
            style="width: 240px"
          />
          <Typography.Text type="secondary">
            : {{ appConfig.servicePort }}（固定）
          </Typography.Text>
        </Space>
      </FormItem>

      <FormItem
        label="局域网扫描范围（可选）"
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
    </Form>
  </div>
</template>

<style scoped>
.print-service-block {
  padding: 0 0 8px;
  margin-bottom: 8px;
}
</style>
