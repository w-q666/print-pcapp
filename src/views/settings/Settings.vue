<script setup lang="ts">
import { ref, markRaw } from 'vue'
import { Tabs, Button, message } from 'ant-design-vue'
import { SaveOutlined } from '@ant-design/icons-vue'
import { useSettings } from '../../stores/settings'
import FileFormatTab from './FileFormatTab.vue'
import PrintSettingsTab from './PrintSettingsTab.vue'
import SystemSettingsTab from './SystemSettingsTab.vue'

const settings = useSettings()
const activeTab = ref('fileFormat')
const saving = ref(false)

const tabItems = [
  { key: 'fileFormat', label: '文件格式', component: markRaw(FileFormatTab) },
  { key: 'printSettings', label: '打印设置', component: markRaw(PrintSettingsTab) },
  { key: 'systemSettings', label: '系统设置', component: markRaw(SystemSettingsTab) },
]

async function handleSave() {
  saving.value = true
  try {
    await settings.saveToStore()
    message.success('配置已保存')
  } catch {
    message.error('保存失败')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="settings-page">
    <div class="settings-header">
      <h2 style="margin: 0">系统配置</h2>
      <Button type="primary" :loading="saving" @click="handleSave">
        <template #icon><SaveOutlined /></template>
        保存配置
      </Button>
    </div>
    <Tabs
      v-model:activeKey="activeTab"
      :items="tabItems.map(t => ({ key: t.key, label: t.label }))"
      style="margin-top: 8px"
    />
    <component :is="tabItems.find(t => t.key === activeTab)?.component" />
  </div>
</template>

<style scoped>
.settings-page {
  padding: 16px 24px;
}
.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
