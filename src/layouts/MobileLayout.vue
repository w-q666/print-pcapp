<script setup lang="ts">
import { computed, h } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { Tabs, TypographyTitle } from 'ant-design-vue'
import {
  FolderOutlined, PrinterOutlined, HistoryOutlined,
  SettingOutlined, FileTextOutlined,
} from '@ant-design/icons-vue'
import PrintStatusOverlay from '../views/print/PrintStatusOverlay.vue'

const router = useRouter()
const route = useRoute()

const activeKey = computed(() => route.name as string)

const tabItemDefs = [
  { key: 'files', label: '文件', icon: FolderOutlined },
  { key: 'print', label: '打印', icon: PrinterOutlined },
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
      <TypographyTitle :level="3" style="margin: 0;">{{ route.meta.title }}</TypographyTitle>
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
  background: var(--bg-primary, #fff);
  border-bottom: 1px solid var(--border-color, #e8e8ec);
}

.mobile-content {
  flex: 1;
  overflow: auto;
}

.mobile-tabs {
  position: sticky;
  bottom: 0;
  background: var(--bg-primary, #fff);
  border-top: 1px solid var(--border-color, #e8e8ec);
}
</style>
