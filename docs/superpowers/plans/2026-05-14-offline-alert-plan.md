# 侧边栏离线警告卡片实施计划

**Goal:** 在侧边栏垂直居中位置添加醒目的离线警告卡片，区分"在线/离线/连接中"三种状态，折叠/展开态均能清晰辨识。

**Architecture:** 新建 `OfflineAlert.vue` 组件，复用 `ServiceStatus` 的检测逻辑，通过 `provide/inject` 在 `NavSidebar` 内共享状态，避免重复轮询。

**Tech Stack:** Vue 3 Composition API (`<script setup>`), Ant Design Vue 图标, 现有 CSS 变量

---

## 文件结构

| 文件 | 操作 |
|------|------|
| `src/components/layout/OfflineAlert.vue` | 新建 |
| `src/components/layout/ServiceStatus.vue` | 修改：提取状态为 provide |
| `src/components/layout/NavSidebar.vue` | 修改：插入 OfflineAlert |

---

### Task 1: 修改 ServiceStatus.vue — 提供状态给子组件

**Files:**
- Modify: `src/components/layout/ServiceStatus.vue`

- [ ] **Step 1: 添加 provide**

在 `<script setup>` 底部添加：

```typescript
import { provide } from 'vue'
// ... 现有代码保持不变 ...

// 在 onMounted 之前添加 provide
provide('serviceStatus', status)
```

- [ ] **Step 2: 验证不破坏现有功能**

运行 `pnpm dev`，确认侧边栏底部小红点正常工作，离线时仍为红色，在线时为绿色。

---

### Task 2: 新建 OfflineAlert.vue

**Files:**
- Create: `src/components/layout/OfflineAlert.vue`

- [ ] **Step 1: 编写组件代码**

```vue
<script setup lang="ts">
import { ref, onMounted, onUnmounted, inject, computed } from 'vue'
import { WarningOutlined, SyncOutlined } from '@ant-design/icons-vue'

type Status = 'online' | 'offline' | 'connecting'

const status = inject<Ref<Status>>('serviceStatus', ref('connecting'))

async function checkConnection() {
  try {
    const { getPrintServers } = await import('../../api/print-api')
    await getPrintServers()
    status.value = 'online'
  } catch {
    status.value = 'offline'
  }
}

let timer: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  checkConnection()
  timer = setInterval(checkConnection, 10000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})

const config = computed(() => {
  switch (status.value) {
    case 'offline':
      return {
        icon: WarningOutlined,
        bg: 'rgba(255, 77, 79, 0.08)',
        border: 'rgba(255, 77, 79, 0.25)',
        color: '#ff4d4f',
        text: '服务离线',
        animate: false,
      }
    case 'connecting':
      return {
        icon: SyncOutlined,
        bg: 'rgba(250, 173, 20, 0.08)',
        border: 'rgba(250, 173, 20, 0.25)',
        color: '#faad14',
        text: '连接中...',
        animate: true,
      }
    default:
      return null
  }
})
</script>

<template>
  <div
    v-if="config"
    class="offline-alert"
    :class="{ 'offline-alert--collapsed': false }"
    :style="{
      background: config.bg,
      borderColor: config.border,
    }"
  >
    <span class="alert-icon" :class="{ 'alert-icon--spin': config.animate }" :style="{ color: config.color }">
      <component :is="config.icon" />
    </span>
    <span class="alert-text" :style="{ color: config.color }">{{ config.text }}</span>
  </div>
</template>

<style scoped>
.offline-alert {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 4px 6px;
  padding: 8px 12px;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  cursor: default;
}

.alert-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  flex-shrink: 0;
}

.alert-icon--spin {
  animation: spin 1.2s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.alert-text {
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
}

.offline-alert--collapsed {
  flex-direction: column;
  padding: 6px 0;
  justify-content: center;
}

.offline-alert--collapsed .alert-text {
  display: none;
}
</style>
```

- [ ] **Step 2: 将 OfflineAlert 放在 NavSidebar 中间位置**

由于 NavSidebar 是 `flex-column`，需要在 `nav-menu` 和 `nav-bottom` 之间插入。但需要让中间区域自动分配剩余空间。将 `nav-menu` 的 `flex: 1` 改为包含一个 wrapper，其结构为：

```
flex-column
├── nav-logo
├── (空隙 flex:1，用来放 OfflineAlert 居中)
├── nav-menu (无 flex)
└── nav-bottom
```

实际上，由于 `nav-menu` 本身是 `flex: 1`，可以直接在模板中把 `OfflineAlert` 放在 `nav-menu` 之后、`nav-bottom` 之前插入，用 CSS 自动居中。

在 `<nav-menu>` 和 `<nav-bottom>` 之间添加：

```html
<div class="nav-spacer">
  <OfflineAlert />
</div>
```

CSS：

```css
.nav-spacer {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-spacer .offline-alert {
  width: calc(100% - 12px);
}

.nav-spacer .offline-alert--collapsed {
  width: auto;
}
```

---

### Task 3: 调整 NavSidebar.vue — 插入 OfflineAlert

**Files:**
- Modify: `src/components/layout/NavSidebar.vue:49`

在 `nav-bottom` 之前插入 `<OfflineAlert>`，并确保 `nav-menu` 的 flex 行为保留。

实际上更简单的方案：将 `nav-menu` 改为 `flex: 0 0 auto`（不自动扩展），在它前面加一个 `flex: 1` 的空隙用于居中。

```vue
<!-- nav-menu 改为 flex: 0 0 auto，添加空隙 -->
<div class="nav-spacer" />

<div class="nav-menu">
  <NavItem ... />
</div>

<div class="nav-bottom">
  ...
</div>

<OfflineAlert v-if="false" /> <!-- 占位，实际在 nav-spacer 中渲染 -->
```

更简洁的方案：在 `nav-menu` 和 `nav-bottom` 之间插入 `OfflineAlert`，然后将 `nav-menu` 的 `flex: 1` 保留，但给它加一个 `min-height: 0` 让内容可滚动，这样 `OfflineAlert` 不会撑开菜单高度。

实际上，CSS `margin: auto` 在 flex 子元素上可以垂直居中。只要 `OfflineAlert` 的父容器是 flex 且高度足够，它就会居中。

在 `NavSidebar` 中，`<nav>` 是 flex-column，`nav-menu` 是 `flex: 1`，`nav-bottom` 是固定高度。中间没有任何元素占位。

最简单的方案：把 `OfflineAlert` 放在 `nav-menu` 后面，但它自己用 `margin: auto` 垂直居中：

```vue
<div class="nav-menu">...</div>
<OfflineAlert class="nav-offline-alert" />
<div class="nav-bottom">...</div>
```

然后在 `nav-sidebar.css` 中：

```css
.nav-offline-alert {
  margin-top: auto;
  margin-bottom: auto;
}
```

这样 OfflineAlert 会在 nav-menu 和 nav-bottom 之间垂直居中。

---

### Task 4: 验证和提交

- [ ] **Step 1: 运行 pnpm dev 验证**

确认：
1. 离线时侧边栏中间出现红色警告卡片
2. 展开态显示"服务离线"文字
3. 上线后警告消失
4. 底部小红点仍正常显示

- [ ] **Step 2: 提交**

```bash
git add src/components/layout/OfflineAlert.vue src/components/layout/ServiceStatus.vue src/components/layout/NavSidebar.vue src/assets/styles/nav-sidebar.css
git commit -m "feat: add offline alert card in sidebar center"
```