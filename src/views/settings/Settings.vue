<script setup lang="ts">
import { ref } from 'vue'
import { Tabs, TabPane, Button, message } from 'ant-design-vue'
import { SaveOutlined } from '@ant-design/icons-vue'
import { useSettings } from '../../stores/settings'
import FileFormatTab from './FileFormatTab.vue'

const settings = useSettings()
const activeTab = ref('fileFormat')
const saving = ref(false)

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
    <Tabs v-model:activeKey="activeTab" style="margin-top: 8px">
      <TabPane key="fileFormat" tab="文件格式">
        <FileFormatTab />
      </TabPane>
      <TabPane key="printSettings" tab="打印设置">
        <div style="padding: 24px; color: #999">打印设置（待实现）</div>
      </TabPane>
      <TabPane key="systemSettings" tab="系统设置">
        <div style="padding: 24px; color: #999">系统设置（待实现）</div>
      </TabPane>
    </Tabs>
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
