<script setup lang="ts">
import {
  FolderOutlined, PrinterOutlined, HistoryOutlined,
  FileTextOutlined, SettingOutlined,
  MenuFoldOutlined,
} from '@ant-design/icons-vue'
import NavItem from './NavItem.vue'
import ServiceStatus from './ServiceStatus.vue'

defineProps<{
  collapsed: boolean
  canToggle: boolean
}>()

const emit = defineEmits<{
  toggle: []
}>()

const mainItems = [
  { key: 'files',   path: '/files',   icon: FolderOutlined,   label: '文件管理' },
  { key: 'print',   path: '/print',   icon: PrinterOutlined,  label: '打印任务' },
  { key: 'history', path: '/history', icon: HistoryOutlined,  label: '打印历史' },
  { key: 'log',     path: '/log',     icon: FileTextOutlined, label: '系统日志' },
]

const bottomItems = [
  { key: 'settings', path: '/settings', icon: SettingOutlined, label: '系统配置' },
]
</script>

<template>
  <nav class="nav-sidebar" :class="{ 'nav-collapsed': collapsed }">
    <div class="nav-logo">
      <PrinterOutlined class="nav-logo-icon" />
      <span class="nav-logo-text">网络打印服务</span>
    </div>

    <div class="nav-menu">
      <NavItem
        v-for="item in mainItems"
        :key="item.key"
        :icon="item.icon"
        :label="item.label"
        :to="item.path"
        :collapsed="collapsed"
      />
    </div>

    <div class="nav-bottom">
      <NavItem
        v-for="item in bottomItems"
        :key="item.key"
        :icon="item.icon"
        :label="item.label"
        :to="item.path"
        :collapsed="collapsed"
      />
      <button
        v-if="canToggle"
        class="nav-collapse-btn"
        @click="emit('toggle')"
        :title="collapsed ? '展开侧边栏' : '折叠侧边栏'"
      >
        <MenuFoldOutlined />
      </button>
    </div>

    <div class="nav-status">
      <ServiceStatus :collapsed="collapsed" />
    </div>
  </nav>
</template>
