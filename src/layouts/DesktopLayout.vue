<script setup lang="ts">
import { computed, h } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import {
  Layout, LayoutSider, LayoutHeader, LayoutContent,
  Menu,
} from 'ant-design-vue'
import { FolderOutlined, HistoryOutlined, SettingOutlined, FileTextOutlined } from '@ant-design/icons-vue'
import PrintStatusOverlay from '../views/print/PrintStatusOverlay.vue'

const router = useRouter()
const route = useRoute()

const selectedKeys = computed(() => [route.name as string])

const menuItems = [
  { key: 'files', icon: () => h(FolderOutlined), label: '文件管理' },
  { key: 'history', icon: () => h(HistoryOutlined), label: '打印历史' },
  { key: 'log', icon: () => h(FileTextOutlined), label: '系统日志' },
  { key: 'settings', icon: () => h(SettingOutlined), label: '系统配置' },
]

function onMenuClick({ key }: { key: string | number }) {
  router.push({ name: String(key) })
}
</script>

<template>
  <Layout style="min-height: 100vh">
    <PrintStatusOverlay />
    <LayoutSider breakpoint="lg" collapsible>
      <div class="sider-logo">
        网络打印服务
      </div>
      <Menu
        theme="dark"
        mode="inline"
        :selected-keys="selectedKeys"
        :items="menuItems"
        @click="onMenuClick"
      />
    </LayoutSider>
    <Layout>
      <LayoutHeader class="desktop-header">
        <h3 style="margin: 0;">{{ route.meta.title }}</h3>
      </LayoutHeader>
      <LayoutContent style="margin: 16px">
        <router-view />
      </LayoutContent>
    </Layout>
  </Layout>
</template>

<style scoped>
.sider-logo {
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 16px;
  font-weight: 600;
}
.desktop-header {
  background: var(--ant-color-bg-container, #fff);
  padding: 0 24px;
  display: flex;
  align-items: center;
}
</style>
