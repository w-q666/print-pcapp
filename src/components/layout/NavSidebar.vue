<script setup lang="ts">
import { computed, ref, provide } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { Menu, MenuItem, Button } from 'ant-design-vue'
import {
  FolderOutlined, PrinterOutlined, HistoryOutlined,
  FileTextOutlined, SettingOutlined,
  MenuFoldOutlined, MenuUnfoldOutlined,
} from '@ant-design/icons-vue'
import ServiceStatus from './ServiceStatus.vue'
import OfflineAlert from './OfflineAlert.vue'

const router = useRouter()
const route = useRoute()

defineProps<{
  collapsed: boolean
  canToggle: boolean
}>()

const emit = defineEmits<{
  toggle: []
}>()

type ServiceConnStatus = 'online' | 'offline' | 'connecting'
const serviceStatus = ref<ServiceConnStatus>('connecting')
provide('serviceStatus', serviceStatus)

const selectedKeys = computed(() => {
  const name = route.name as string
  return name ? [name] : ['files']
})

const mainItems = [
  { key: 'files',   path: '/files',   icon: FolderOutlined,   label: '文件管理' },
  { key: 'print',   path: '/print',   icon: PrinterOutlined,  label: '打印任务' },
  { key: 'history', path: '/history', icon: HistoryOutlined,  label: '打印历史' },
  { key: 'log',     path: '/log',     icon: FileTextOutlined, label: '系统日志' },
]

const bottomItems = [
  { key: 'settings', path: '/settings', icon: SettingOutlined, label: '系统配置' },
]

function handleMenuClick(info: { key: string | number }) {
  const allItems = [...mainItems, ...bottomItems]
  const item = allItems.find(i => i.key === String(info.key))
  if (item) {
    router.push(item.path)
  }
}
</script>

<template>
  <div class="nav-sidebar" :class="{ 'nav-collapsed': collapsed }">
    <div class="nav-logo">
      <PrinterOutlined class="nav-logo-icon" />
      <span class="nav-logo-text">网络打印服务</span>
    </div>

    <Menu
      :selectedKeys="selectedKeys"
      mode="inline"
      :inline-collapsed="collapsed"
      class="nav-menu"
      @click="handleMenuClick"
    >
      <MenuItem v-for="item in mainItems" :key="item.key">
        <template #icon>
          <component :is="item.icon" />
        </template>
        {{ item.label }}
      </MenuItem>
    </Menu>

    <OfflineAlert :collapsed="collapsed" class="nav-offline-alert" />

    <div class="nav-bottom">
      <Menu
        :selectedKeys="selectedKeys"
        mode="inline"
        :inline-collapsed="collapsed"
        class="nav-bottom-menu"
        @click="handleMenuClick"
      >
        <MenuItem v-for="item in bottomItems" :key="item.key">
          <template #icon>
            <component :is="item.icon" />
          </template>
          {{ item.label }}
        </MenuItem>
      </Menu>
      <Button
        v-if="canToggle"
        type="text"
        class="nav-collapse-btn"
        @click="emit('toggle')"
        :title="collapsed ? '展开侧边栏' : '折叠侧边栏'"
      >
        <template #icon>
          <MenuUnfoldOutlined v-if="collapsed" />
          <MenuFoldOutlined v-else />
        </template>
      </Button>
    </div>

    <div class="nav-status">
      <ServiceStatus :collapsed="collapsed" />
    </div>
  </div>
</template>
