<script setup lang="ts">
import { computed, h } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { Tabs } from 'ant-design-vue'
import { FolderOutlined, HistoryOutlined, SettingOutlined, FileTextOutlined } from '@ant-design/icons-vue'
import PrintStatusOverlay from '../views/print/PrintStatusOverlay.vue'

const router = useRouter()
const route = useRoute()

const activeKey = computed(() => route.name as string)

const tabItemDefs = [
  { key: 'files', label: '文件', icon: FolderOutlined },
  { key: 'history', label: '历史', icon: HistoryOutlined },
  { key: 'log', label: '日志', icon: FileTextOutlined },
  { key: 'settings', label: '设置', icon: SettingOutlined },
]

const tabItems = tabItemDefs.map(item => ({
  key: item.key,
  label: h('div', { style: 'text-align: center' }, [
    h(item.icon),
    h('div', { style: 'font-size: 12px' }, item.label),
  ]),
}))

function onTabChange(key: string | number) {
  router.push({ name: String(key) })
}
</script>

<template>
  <div class="mobile-layout">
    <PrintStatusOverlay />
    <div class="mobile-header">
      <h3 style="margin: 0;">{{ route.meta.title }}</h3>
    </div>
    <div class="mobile-content">
      <router-view />
    </div>
    <Tabs
      :active-key="activeKey"
      :items="tabItems"
      centered
      size="large"
      class="mobile-tabs"
      @change="onTabChange"
    />
  </div>
</template>

<style scoped>
.mobile-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
}
.mobile-header {
  padding: 12px 16px;
  background: var(--ant-color-bg-container, #fff);
  border-bottom: 1px solid var(--ant-color-border-secondary, #f0f0f0);
}
.mobile-content {
  flex: 1;
  overflow: auto;
}
.mobile-tabs {
  position: sticky;
  bottom: 0;
  background: var(--ant-color-bg-container, #fff);
  border-top: 1px solid var(--ant-color-border-secondary, #f0f0f0);
}
</style>
