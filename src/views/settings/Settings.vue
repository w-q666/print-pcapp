<script setup lang="ts">
import { ref, markRaw } from 'vue'
import { storeToRefs } from 'pinia'
import { Tabs, Button, message } from 'ant-design-vue'
import { SaveOutlined } from '@ant-design/icons-vue'
import BasePage from '../../components/layout/BasePage.vue'
import { useSettings } from '../../stores/settings'
import { useAppConfig } from '../../stores/app-config'
import { usePrinterList } from '../../stores/printer-list'
import { useSettingsNav } from '../../stores/settings-nav'
import { validateScanRange, validateDefaultServiceHost, validateServicePort } from '../../utils/ip-range'
import { usePlatform } from '../../composables/usePlatform'
import FileFormatTab from './FileFormatTab.vue'
import PrintSettingsTab from './PrintSettingsTab.vue'
import SystemSettingsTab from './SystemSettingsTab.vue'
import PrintServiceConfigBlock from './PrintServiceConfigBlock.vue'

const settings = useSettings()
const appConfig = useAppConfig()
const printerList = usePrinterList()
const settingsNav = useSettingsNav()
const { activeKey } = storeToRefs(settingsNav)
const { platform } = usePlatform()
const saving = ref(false)

const tabItems = [
  { key: 'printService', label: '打印服务', component: markRaw(PrintServiceConfigBlock) },
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
  const portV = validateServicePort(appConfig.servicePort)
  if (!portV.ok) {
    message.error(portV.message ?? '服务端口不合法')
    return
  }
  const scanV = validateScanRange(appConfig.scanStartIp, appConfig.scanEndIp)
  if (!scanV.ok) {
    message.error(scanV.message ?? '扫描范围不合法')
    return
  }
  saving.value = true
  try {
    try {
      if (printerList.allPrinters.length === 0) {
        await printerList.refresh()
      }
    } catch {
      // 打印服务不可用时仍允许保存网络等配置
    }
    settings.reconcileDefaultPrinterWithVisible(printerList.visiblePrinters)
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
  <BasePage
    title="系统配置"
    :scroll-mode="platform === 'desktop' ? 'content' : 'page'"
  >
    <template #actions>
      <Button type="primary" size="small" :loading="saving" @click="handleSave">
        <template #icon><SaveOutlined /></template>
        保存配置
      </Button>
    </template>

    <div
      class="settings-shell"
      :class="{ 'settings-shell--desktop': platform === 'desktop' }"
    >
      <Tabs
        v-if="platform !== 'desktop'"
        v-model:activeKey="activeKey"
        class="settings-tabs settings-tabs--sticky-mobile"
        size="small"
        :items="tabItems.map((t) => ({ key: t.key, label: t.label }))"
      />
      <div class="settings-shell-body">
        <component :is="tabItems.find((t) => t.key === activeKey)?.component" />
      </div>
    </div>
  </BasePage>
</template>

<style scoped>
.settings-shell--desktop {
  display: flex;
  flex-direction: column;
  flex: 1 1 0;
  min-height: 0;
  height: 100%;
}

.settings-shell--desktop .settings-shell-body {
  flex: 1 1 0;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
}

.settings-tabs--sticky-mobile {
  position: sticky;
  z-index: 15;
  top: var(--page-header-height);
  margin-bottom: 8px;
  padding-top: 4px;
  background: var(--bg-content);
}

.settings-tabs :deep(.ant-tabs-nav) {
  margin: 0;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--border-color);
}

.settings-tabs :deep(.ant-tabs-nav-wrap) {
  overflow-x: auto;
  overflow-y: hidden;
  scrollbar-gutter: stable;
}

.settings-tabs :deep(.ant-tabs-nav-list) {
  flex-wrap: nowrap;
  min-width: min-content;
}

.settings-tabs :deep(.ant-tabs-tab) {
  flex-shrink: 0;
}

.settings-tabs :deep(.ant-tabs-content-holder) {
  display: none;
}
</style>
