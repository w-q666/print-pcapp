import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      redirect: '/files',
    },
    {
      path: '/files',
      name: 'files',
      component: () => import('../views/home/HomePage.vue'),
      meta: { title: '文件管理', icon: 'FolderOutlined' },
    },
    {
      path: '/print',
      name: 'print',
      component: () => import('../views/print/PrintQueue.vue'),
      meta: { title: '打印任务', icon: 'PrinterOutlined' },
    },
    {
      path: '/history',
      name: 'history',
      component: () => import('../views/history/PrintHistory.vue'),
      meta: { title: '打印历史', icon: 'HistoryOutlined' },
    },
    {
      path: '/log',
      name: 'log',
      component: () => import('../views/log/SystemLog.vue'),
      meta: { title: '系统日志', icon: 'FileTextOutlined' },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/settings/Settings.vue'),
      meta: { title: '系统配置', icon: 'SettingOutlined' },
    },
  ],
})

export default router
