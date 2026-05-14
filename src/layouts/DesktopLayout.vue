<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import {
  Layout, LayoutSider, LayoutHeader, LayoutContent,
  Menu, MenuItem,
} from 'ant-design-vue'
import { FolderOutlined, HistoryOutlined, SettingOutlined, FileTextOutlined } from '@ant-design/icons-vue'
import PrintStatusOverlay from '../views/print/PrintStatusOverlay.vue'

const router = useRouter()
const route = useRoute()

const selectedKeys = computed(() => [route.name as string])

const menuItems = [
  { key: 'files', icon: FolderOutlined, label: '文件管理' },
  { key: 'history', icon: HistoryOutlined, label: '打印历史' },
  { key: 'log', icon: FileTextOutlined, label: '系统日志' },
  { key: 'settings', icon: SettingOutlined, label: '系统配置' },
]

function onMenuClick({ key }: { key: string | number }) {
  router.push({ name: String(key) })
}
</script>

<template>
  <Layout style="min-height: 100vh">
    <PrintStatusOverlay />
    <LayoutSider breakpoint="lg" collapsible>
      <div style="height: 64px; display: flex; align-items: center; justify-content: center; color: #fff; font-size: 16px; font-weight: 600;">
        网络打印服务
      </div>
      <Menu
        theme="dark"
        mode="inline"
        :selected-keys="selectedKeys"
        @click="onMenuClick"
      >
        <MenuItem v-for="item in menuItems" :key="item.key">
          <component :is="item.icon" />
          <span>{{ item.label }}</span>
        </MenuItem>
      </Menu>
    </LayoutSider>
    <Layout>
      <LayoutHeader style="background: #fff; padding: 0 24px; display: flex; align-items: center;">
        <h3 style="margin: 0;">{{ route.meta.title }}</h3>
      </LayoutHeader>
      <LayoutContent style="margin: 16px">
        <router-view />
      </LayoutContent>
    </Layout>
  </Layout>
</template>
