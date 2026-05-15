<script setup lang="ts">
import { ref, markRaw } from 'vue'
import { Tabs, Button, message } from 'ant-design-vue'
import { SaveOutlined } from '@ant-design/icons-vue'
import BasePage from '../../components/layout/BasePage.vue'
import { useSettings } from '../../stores/settings'
import { useAppConfig } from '../../stores/app-config'
import { validateScanRange, validateDefaultServiceHost } from '../../utils/ip-range'
import FileFormatTab from './FileFormatTab.vue'
import PrintSettingsTab from './PrintSettingsTab.vue'
import SystemSettingsTab from './SystemSettingsTab.vue'

const settings = useSettings()
const appConfig = useAppConfig()
const activeTab = ref('fileFormat')
const saving = ref(false)

const tabItems = [
  { key: 'fileFormat', label: '文件格式', component: markRaw(FileFormatTab) },
  { key: 'printSettings', label: '打印设置', component: markRaw(PrintSettingsTab) },
  { key: 'systemSettings', label: '系统设置', component: markRaw(SystemSettingsTab) },
]

async function handleSave() {
  const hostV = validateDefaultServiceHost(appConfig.serviceHost)
  if (!hostV.ok) {
    message.error(hostV.message ?? '默认服务 IP 不合法')
    return
  }
  const scanV = validateScanRange(appConfig.scanStartIp, appConfig.scanEndIp)
  if (!scanV.ok) {
    message.error(scanV.message ?? '扫描范围不合法')
    return
  }
  saving.value = true
  try {
    appConfig.scanRangeInferredOnce = true
    await Promise.all([settings.saveToStore(), appConfig.saveToStore()])
    message.success('配置已保存')
  } catch {
    message.error('保存失败')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <BasePage title="系统配置">
    <template #actions>
      <Button type="primary" :loading="saving" @click="handleSave">
        <template #icon><SaveOutlined /></template>
        保存配置
      </Button>
    </template>

    <div class="settings-content">
      <Tabs
        v-model:activeKey="activeTab"
        :items="tabItems.map(t => ({ key: t.key, label: t.label }))"
        size="small"
      />
      <component :is="tabItems.find(t => t.key === activeTab)?.component" />
    </div>
  </BasePage>
</template>

