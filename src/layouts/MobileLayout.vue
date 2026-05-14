<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { Tabs, TabPane } from 'ant-design-vue'
import { FolderOutlined, HistoryOutlined, SettingOutlined, FileTextOutlined } from '@ant-design/icons-vue'
import PrintStatusOverlay from '../views/print/PrintStatusOverlay.vue'

const router = useRouter()
const route = useRoute()

const activeKey = computed(() => route.name as string)

const tabItems = [
  { key: 'files', label: '文件', icon: FolderOutlined },
  { key: 'history', label: '历史', icon: HistoryOutlined },
  { key: 'log', label: '日志', icon: FileTextOutlined },
  { key: 'settings', label: '设置', icon: SettingOutlined },
]

function onTabChange(key: string | number) {
  router.push({ name: String(key) })
}
</script>

<template>
  <div style="display: flex; flex-direction: column; height: 100vh;">
    <PrintStatusOverlay />
    <div style="padding: 12px 16px; background: #fff; border-bottom: 1px solid #f0f0f0;">
      <h3 style="margin: 0;">{{ route.meta.title }}</h3>
    </div>
    <div style="flex: 1; overflow: auto;">
      <router-view />
    </div>
    <Tabs
      :active-key="activeKey"
      centered
      size="large"
      @change="onTabChange"
      style="position: sticky; bottom: 0; background: #fff; border-top: 1px solid #f0f0f0;"
    >
      <TabPane v-for="item in tabItems" :key="item.key">
        <template #tab>
          <div style="text-align: center;">
            <component :is="item.icon" />
            <div style="font-size: 12px;">{{ item.label }}</div>
          </div>
        </template>
      </TabPane>
    </Tabs>
  </div>
</template>
