# Plan 1: 前端基础设施与布局

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 搭建 Vue 3 前端骨架：Router、Pinia、平台检测、双布局（桌面侧边栏 + 移动底部 Tabs），替换脚手架 demo，产出可导航的空壳页面。

**Architecture:** App.vue 根据 `usePlatform()` 选择 DesktopLayout 或 MobileLayout，两者共用同一路由表和 Pinia stores。antd 6.x 已安装。

**Tech Stack:** Vue 3, Vue Router (hash 模式), Pinia, antd 6.x, TypeScript

**Dependencies:** 无（第一波，可立即开始）

**Files to create:**
- `src/router/index.ts` — 路由配置
- `src/stores/index.ts` — Pinia 注册
- `src/stores/app-config.ts` — 应用配置 store
- `src/composables/usePlatform.ts` — 平台检测
- `src/layouts/DesktopLayout.vue` — 桌面布局
- `src/layouts/MobileLayout.vue` — 移动布局
- `src/views/file-manager/FileManager.vue` — 文件管理占位页
- `src/views/history/PrintHistory.vue` — 打印历史占位页
- `src/views/settings/Settings.vue` — 设置占位页

**Files to modify:**
- `package.json` — 添加 vue-router, pinia
- `src/main.ts` — 注册 Router, Pinia
- `src/App.vue` — 重构为布局选择器

---

### Task 1: 安装前端依赖

**Files:** `package.json`

- [ ] **Step 1: 安装 vue-router 和 pinia**

```bash
pnpm add vue-router pinia
```

- [ ] **Step 2: 验证 pnpm build 通过**

```bash
pnpm build
```

Expected: 编译通过（此时 router/pinia 未使用，不会报错）。

- [ ] **Step 3: Commit**

```bash
git add package.json pnpm-lock.yaml
git commit -m "chore(deps): add vue-router and pinia"
```

---

### Task 2: 平台检测 composable

**Files:** Create `src/composables/usePlatform.ts`

- [ ] **Step 1: 创建 usePlatform.ts**

实现平台检测逻辑：

```typescript
import { ref } from 'vue'

export type Platform = 'desktop' | 'mobile'

export function usePlatform() {
  const platform = ref<Platform>('desktop')

  // Tauri 2 通过 OsType 判断
  async function detect() {
    try {
      const { type } = await import('@tauri-apps/plugin-os')
      const osType = await type()
      platform.value = (osType === 'android' || osType === 'ios') ? 'mobile' : 'desktop'
    } catch {
      // 回退：基于窗口宽度
      platform.value = window.innerWidth < 768 ? 'mobile' : 'desktop'
    }
  }

  return { platform, detect }
}
```

注意：Tauri `@tauri-apps/plugin-os` 需要单独安装。如果不想增加依赖，可以先只用窗口宽度判断，后续再接入 `plugin-os`。推荐先用窗口宽度作为 MVP 方案。

- [ ] **Step 2: Commit**

```bash
git add src/composables/usePlatform.ts
git commit -m "feat(composables): add usePlatform for platform detection"
```

---

### Task 3: 路由配置

**Files:** Create `src/router/index.ts`

- [ ] **Step 1: 创建路由配置**

```typescript
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
      component: () => import('../views/file-manager/FileManager.vue'),
      meta: { title: '文件管理', icon: 'FolderOutlined' },
    },
    {
      path: '/history',
      name: 'history',
      component: () => import('../views/history/PrintHistory.vue'),
      meta: { title: '打印历史', icon: 'HistoryOutlined' },
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
```

- [ ] **Step 2: Commit**

```bash
git add src/router/index.ts
git commit -m "feat(router): add hash-mode router with files/history/settings routes"
```

---

### Task 4: Pinia Store 初始化

**Files:** Create `src/stores/index.ts`, `src/stores/app-config.ts`

- [ ] **Step 1: 创建 stores/index.ts**

```typescript
export { useAppConfig } from './app-config'
```

- [ ] **Step 2: 创建 stores/app-config.ts**

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAppConfig = defineStore('app-config', () => {
  const serviceHost = ref('localhost')
  const servicePort = ref(2024)
  const lanPort = ref(5000)

  const serviceUrl = computed(() => `http://${serviceHost.value}:${servicePort.value}`)
  const wsUrl = computed(() => `ws://${serviceHost.value}:${servicePort.value}/print`)

  async function loadFromStore() {
    try {
      const { Store } = await import('@tauri-apps/plugin-store')
      const store = await Store.load('settings.json')
      const host = await store.get<string>('serviceHost')
      const port = await store.get<number>('servicePort')
      const lp = await store.get<number>('lanPort')
      if (host) serviceHost.value = host
      if (port) servicePort.value = port
      if (lp) lanPort.value = lp
    } catch (e) {
      console.warn('Failed to load config from store:', e)
    }
  }

  async function saveToStore() {
    try {
      const { Store } = await import('@tauri-apps/plugin-store')
      const store = await Store.load('settings.json')
      await store.set('serviceHost', serviceHost.value)
      await store.set('servicePort', servicePort.value)
      await store.set('lanPort', lanPort.value)
      await store.save()
    } catch (e) {
      console.warn('Failed to save config to store:', e)
    }
  }

  return { serviceHost, servicePort, lanPort, serviceUrl, wsUrl, loadFromStore, saveToStore }
})
```

注意：需要 `import { computed } from 'vue'`。

- [ ] **Step 3: Commit**

```bash
git add src/stores/
git commit -m "feat(stores): add Pinia app-config store with plugin-store persistence"
```

---

### Task 5: 占位页面

**Files:** Create view stubs

- [ ] **Step 1: 创建 FileManager.vue**

```vue
<script setup lang="ts">
</script>
<template>
  <div style="padding: 24px">
    <h2>文件管理</h2>
    <p>待实现（Plan 4）</p>
  </div>
</template>
```

- [ ] **Step 2: 创建 PrintHistory.vue**

```vue
<script setup lang="ts">
</script>
<template>
  <div style="padding: 24px">
    <h2>打印历史</h2>
    <p>待实现（Plan 6）</p>
  </div>
</template>
```

- [ ] **Step 3: 创建 Settings.vue**

```vue
<script setup lang="ts">
</script>
<template>
  <div style="padding: 24px">
    <h2>系统配置</h2>
    <p>待实现（Plan 6）</p>
  </div>
</template>
```

- [ ] **Step 4: Commit**

```bash
git add src/views/
git commit -m "feat(views): add placeholder pages for files, history, settings"
```

---

### Task 6: 桌面布局组件

**Files:** Create `src/layouts/DesktopLayout.vue`

- [ ] **Step 1: 创建 DesktopLayout.vue**

使用 antd Layout 组件：左侧 Sider（可折叠）+ 右侧 Content。

Sider 菜单项：
- 文件管理 (FolderOutlined) → /files
- 打印历史 (HistoryOutlined) → /history
- 系统配置 (SettingOutlined) → /settings

参考设计规格书 2.2 节目录结构中的 `DesktopLayout.vue`。

菜单项根据 `router.currentRoute` 高亮。点击菜单项通过 `router.push()` 导航。

顶部 Header 显示应用标题「网络打印服务」，右侧可选放「刷新打印机」按钮。

- [ ] **Step 2: Commit**

```bash
git add src/layouts/DesktopLayout.vue
git commit -m "feat(layouts): add DesktopLayout with antd Sider navigation"
```

---

### Task 7: 移动端布局组件

**Files:** Create `src/layouts/MobileLayout.vue`

- [ ] **Step 1: 创建 MobileLayout.vue**

Header + `<router-view>` + 底部 antd Tabs。

底部 Tab 项与桌面侧边栏菜单一致：文件 / 历史 / 设置。

Tab 切换通过 `router.push()` 导航，当前路由同步 Tab 高亮。

- [ ] **Step 2: Commit**

```bash
git add src/layouts/MobileLayout.vue
git commit -m "feat(layouts): add MobileLayout with bottom Tabs navigation"
```

---

### Task 8: 重构 App.vue 与 main.ts

**Files:** Modify `src/App.vue`, `src/main.ts`

- [ ] **Step 1: 重构 main.ts**

```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
```

- [ ] **Step 2: 重构 App.vue**

删除所有脚手架 demo 代码。新逻辑：

```vue
<script setup lang="ts">
import { onMounted } from 'vue'
import { usePlatform } from './composables/usePlatform'
import { useAppConfig } from './stores/app-config'
import DesktopLayout from './layouts/DesktopLayout.vue'
import MobileLayout from './layouts/MobileLayout.vue'

const { platform, detect } = usePlatform()
const appConfig = useAppConfig()

onMounted(async () => {
  await detect()
  await appConfig.loadFromStore()
})
</script>

<template>
  <DesktopLayout v-if="platform === 'desktop'" />
  <MobileLayout v-else />
</template>
```

- [ ] **Step 3: 验证 pnpm build 通过**

```bash
pnpm build
```

Expected: vue-tsc 和 vite build 均通过，无类型错误。

- [ ] **Step 4: 验证 pnpm tauri dev 运行正常**

```bash
pnpm tauri dev
```

Expected: 应用启动，显示桌面布局，侧边栏可导航到三个页面。

- [ ] **Step 5: Commit**

```bash
git add src/main.ts src/App.vue
git commit -m "feat(app): refactor App.vue with platform-adaptive layout and router"
```

---

### 验收标准

1. `pnpm build` 编译通过，无类型错误
2. `pnpm tauri dev` 启动后显示桌面布局：左侧 Sider 含三个导航项
3. 点击导航可切换到文件管理/打印历史/系统配置占位页
4. `app-config` store 能从 plugin-store 读写 serviceHost/servicePort
5. 窗口缩小到 768px 以下时切换为移动布局（如使用窗口宽度检测方案）
