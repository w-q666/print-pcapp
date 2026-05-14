# 布局重构设计文档

> 参照项目：[Clash Verge Rev](https://github.com/clash-verge-rev/clash-verge-rev)
> 日期：2026-05-14
> 状态：已确认

## 1. 目标

将当前 Ant Design 经典后台布局（深色侧边栏 + 顶部 Header）全面重构为 Clash Verge Rev 风格的现代桌面应用布局。完全抛弃原有 UI 风格，所有布局组件重写。

### 核心原则

- 参照 Clash Verge Rev 的布局范式，不保留原项目 UI 风格
- 页面内部数据展示组件继续使用 Ant Design Vue（Table、Card、Form 等）
- 布局壳层（侧边栏、标题栏、页面容器）全部自定义实现
- 不同屏幕尺寸自适应兼容

## 2. 窗口与标题栏

### 2.1 无边框窗口

修改 `src-tauri/tauri.conf.json`：

```json
{
  "windows": [
    {
      "decorations": false,
      "width": 900,
      "height": 620,
      "minWidth": 680,
      "minHeight": 480,
      "title": "网络打印服务"
    }
  ]
}
```

### 2.2 自定义标题栏 — `TitleBar.vue`

```
┌──────────────────────────────────────────────────────────┐
│ [data-tauri-drag-region ──────────────] [─] [□] [×]      │
└──────────────────────────────────────────────────────────┘
```

- 高度：36px，横跨窗口全宽
- 左侧：可拖拽区域（`data-tauri-drag-region`）
- 右侧：最小化 / 最大化 / 关闭按钮
- 使用 `@tauri-apps/api/window` 的 `getCurrentWindow()` 获取窗口实例，调用 `.minimize()` / `.toggleMaximize()` / `.close()`
- 窗口控制按钮样式对齐 Windows 系统风格（hover 高亮，关闭按钮 hover 红色）

## 3. 整体布局结构

### 3.1 DOM 结构

```
div.app-layout
├── TitleBar                              // 36px 固定标题栏
└── div.layout-body                       // flex row, 填满剩余高度
    ├── NavSidebar                        // 200px | 64px 侧边栏
    └── main.layout-content               // flex: 1 内容区
        └── <router-view />               // 各页面（包裹在 BasePage 中）
```

### 3.2 CSS 布局

```css
.app-layout {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.layout-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.layout-content {
  flex: 1;
  position: relative;
  overflow: hidden;
}
```

## 4. 侧边导航栏 — `NavSidebar.vue`

### 4.1 结构

```
nav.nav-sidebar (.nav-collapsed 折叠态)
├── div.nav-logo                          // Logo 区域 (58px)
│   └── 应用图标 + "网络打印服务" 文字（折叠时隐藏文字）
├── div.nav-menu                          // 菜单列表区（flex: 1, 可滚动）
│   ├── NavItem 文件管理
│   ├── NavItem 打印任务
│   ├── NavItem 打印历史
│   └── NavItem 系统日志
├── div.nav-bottom                        // 底部固定区
│   └── NavItem 系统配置
└── div.nav-status                        // 服务状态 (60px)
    └── ServiceStatus 组件
```

### 4.2 展开与折叠

| 状态 | 宽度 | 内容 |
|---|---|---|
| 展开 | 200px | 图标 + 文字 |
| 折叠 | 64px | 仅图标 + Tooltip |

- 折叠切换方式：侧边栏底部的折叠按钮（chevron 图标）
- 过渡动画：`transition: width 200ms ease`
- 折叠态通过 `.nav-collapsed` CSS class 控制
- 侧边栏与内容区之间用 `border-right: 1px solid var(--border-color)` 分割

### 4.3 导航项 — `NavItem.vue`

Props：`icon`、`label`、`to`（路由路径）、`collapsed`（是否折叠）

```
div.nav-item (.nav-item--active 选中态)
├── span.nav-item-icon   // 24px 图标
└── span.nav-item-label  // 文字（折叠时隐藏，Tooltip 替代）
```

**样式：**
- 默认态：透明背景，`color: var(--text-secondary)`
- Hover 态：`background: var(--hover-bg)`，圆角 8px
- 选中态：`background: var(--primary-color)`，`color: #fff`，圆角 8px
- 图标统一使用 `@ant-design/icons-vue` 的线性图标（Outlined 系列）
- 整个 item 高度 40px，左右 padding 16px（展开），居中对齐（折叠）

### 4.4 菜单项定义

```ts
const navItems = [
  { key: 'files',   path: '/files',   icon: FolderOutlined,   label: '文件管理' },
  { key: 'print',   path: '/print',   icon: PrinterOutlined,  label: '打印任务' },
  { key: 'history', path: '/history', icon: HistoryOutlined,  label: '打印历史' },
  { key: 'log',     path: '/log',     icon: FileTextOutlined, label: '系统日志' },
]

const bottomItems = [
  { key: 'settings', path: '/settings', icon: SettingOutlined, label: '系统配置' },
]
```

## 5. 页面容器 — `BasePage.vue`

提供统一的页面标题栏 + 可滚动内容区：

```
div.base-page
├── header.base-page-header               // 48px 固定
│   ├── h2.base-page-title                // 页面标题（左侧）
│   └── div.base-page-actions             // 操作按钮 slot（右侧）
└── div.base-page-content                 // flex: 1, overflow-y: auto
    └── <slot />                          // 页面内容
```

Props：
- `title: string` — 页面标题
- Slots：
  - `actions` — 右侧操作按钮区
  - `default` — 页面主体内容

## 6. 服务状态指示 — `ServiceStatus.vue`

显示 Java 打印服务的连接状态：

```
div.service-status
├── span.status-dot (.online | .offline | .connecting)
└── span.status-text   // "在线" / "离线" / "连接中"（折叠时隐藏）
```

- 绿色圆点 = 在线，红色 = 离线，黄色 = 连接中
- 每 10 秒轮询 `getPrintServers()` 检测连接
- 折叠态只显示圆点

## 7. 路由配置

```ts
const routes = [
  { path: '/', redirect: '/files' },
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
]
```

## 8. 新增页面：打印任务 — `PrintQueue.vue`

将原来的 `PrintStatusOverlay`（右下角悬浮通知条）升级为独立页面：

- 显示当前打印队列（等待中 + 打印中）
- 每个任务卡片：文件名、状态（进度条/完成/失败）、打印机、提交时间
- 实时 WebSocket 状态推送更新
- 操作：取消任务、重试失败任务

`PrintStatusOverlay` 仍然保留为全局悬浮组件，但简化为仅显示最新一条打印状态的小通知，点击跳转到 `/print` 页面。

## 9. 屏幕兼容策略

### 9.1 断点定义

| 断点 | 窗口宽度 | 行为 |
|---|---|---|
| Desktop 宽 | ≥ 1024px | 侧边栏展开 (200px) |
| Desktop 窄 | 768px ~ 1023px | 侧边栏自动折叠 (64px) |
| Mobile | < 768px | 切换为 MobileLayout（底部 Tab） |

### 9.2 侧边栏响应式

```ts
// composable: useNavCollapse
const isAutoCollapsed = computed(() => windowWidth.value < 1024 && windowWidth.value >= 768)
const isManualCollapsed = ref(false)
const collapsed = computed(() => isAutoCollapsed.value || isManualCollapsed.value)
```

- 窄屏时自动折叠，用户无法手动展开
- 宽屏时用户可手动切换折叠/展开，手动状态持久化到 Tauri Store

### 9.3 页面内部响应式

各页面内部也需适配：

- **文件管理**（HomePage）：宽屏左右分栏（16/8），窄屏上下堆叠（24/24）
- **打印历史**：表格在窄屏下隐藏次要列（类型、来源）
- **系统配置**：Tab 内的表单在窄屏下单列排布

## 10. 样式架构

### 10.1 文件结构

```
src/assets/styles/
├── variables.css          // CSS 自定义属性（主题色、间距、圆角等）
├── layout.css             // 布局壳层样式（app-layout, layout-body）
├── nav-sidebar.css        // 侧边栏 + 折叠态样式
├── base-page.css          // BasePage 组件样式
├── titlebar.css           // 标题栏样式
└── global.css             // 全局重置 + 字体 + 滚动条
```

### 10.2 CSS 变量

```css
:root {
  /* 布局 */
  --sidebar-width: 200px;
  --sidebar-collapsed-width: 64px;
  --titlebar-height: 36px;
  --page-header-height: 48px;

  /* 颜色 */
  --bg-primary: #ffffff;
  --bg-sidebar: #f7f8fa;
  --bg-content: #ffffff;
  --border-color: #e8e8ec;
  --text-primary: #1d2129;
  --text-secondary: #86909c;
  --primary-color: #1677ff;
  --hover-bg: rgba(0, 0, 0, 0.04);
  --success-color: #52c41a;
  --error-color: #ff4d4f;
  --warning-color: #faad14;

  /* 动画 */
  --transition-duration: 200ms;

  /* 圆角 */
  --radius-sm: 6px;
  --radius-md: 8px;
  --radius-lg: 12px;
}
```

### 10.3 滚动条样式

自定义细滚动条（Clash Verge 风格）：

```css
::-webkit-scrollbar { width: 6px; height: 6px; }
::-webkit-scrollbar-thumb { background: rgba(0,0,0,0.15); border-radius: 3px; }
::-webkit-scrollbar-track { background: transparent; }
```

## 11. 组件变更总表

| 操作 | 文件路径 | 说明 |
|---|---|---|
| **重写** | `src/layouts/DesktopLayout.vue` | 新布局壳层 |
| **新建** | `src/components/layout/TitleBar.vue` | 自定义标题栏 |
| **新建** | `src/components/layout/NavSidebar.vue` | 侧边导航栏 |
| **新建** | `src/components/layout/NavItem.vue` | 单个导航项 |
| **新建** | `src/components/layout/BasePage.vue` | 页面统一容器 |
| **新建** | `src/components/layout/ServiceStatus.vue` | 服务状态指示 |
| **新建** | `src/views/print/PrintQueue.vue` | 打印任务队列页面 |
| **新建** | `src/composables/useNavCollapse.ts` | 侧边栏折叠逻辑 |
| **新建** | `src/assets/styles/variables.css` | CSS 变量 |
| **新建** | `src/assets/styles/layout.css` | 布局样式 |
| **新建** | `src/assets/styles/nav-sidebar.css` | 侧边栏样式 |
| **新建** | `src/assets/styles/base-page.css` | BasePage 样式 |
| **新建** | `src/assets/styles/titlebar.css` | 标题栏样式 |
| **新建** | `src/assets/styles/global.css` | 全局样式 |
| **修改** | `src/router/index.ts` | 新增 `/print` 路由 |
| **修改** | `src/App.vue` | 引入全局样式，移除旧内联样式 |
| **修改** | `src-tauri/tauri.conf.json` | 无边框窗口配置 |
| **修改** | `src/views/home/HomePage.vue` | 包裹 BasePage，优化响应式 |
| **修改** | `src/views/history/PrintHistory.vue` | 包裹 BasePage |
| **修改** | `src/views/log/SystemLog.vue` | 包裹 BasePage |
| **修改** | `src/views/settings/Settings.vue` | 包裹 BasePage，移除内部标题 |
| **修改** | `src/views/print/PrintStatusOverlay.vue` | 简化为小通知条 |
| **保留** | `src/layouts/MobileLayout.vue` | 移动端布局保持不变 |
| **保留** | 所有 `src/components/` 组件 | 数据展示组件不变 |
| **保留** | 所有 `src/stores/` | 状态管理不变 |
| **保留** | 所有 `src/composables/` | 组合式函数不变 |
| **保留** | 所有 `src/api/` | API 层不变 |

## 12. 不在本次范围

- 暗色主题 / 跟随系统主题（后续可扩展，CSS 变量已预留）
- 拖拽排序导航菜单（Clash Verge 有此功能，但对打印应用不必要）
- 国际化（当前仅中文）
- 自定义主题色选择器
