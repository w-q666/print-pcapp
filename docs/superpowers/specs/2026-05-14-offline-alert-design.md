# 侧边栏离线警告卡片设计

## 背景

当前 `ServiceStatus.vue` 在侧边栏最底部显示一个小圆点指示服务状态，离线时不够醒目，容易被忽略。尤其是折叠态下仅一个小红点，用户体验不足。

## 设计目标

在侧边栏垂直居中位置添加醒目的离线警告提示，区分"在线/离线/连接中"三种状态，在折叠和展开两种模式下均能清晰辨识。

## 当前布局（NavSidebar.vue）

```
flex-column
├── nav-logo        (48px, fixed)
├── nav-menu        (flex: 1, scrollable)
├── nav-bottom      (fixed, border-top)
└── nav-status      (40px, fixed bottom)
```

## 方案 A（采纳）

### 位置

在 `nav-menu`（flex:1，撑满中间）和 `nav-bottom`（固定底部配置项）之间插入 `<OfflineAlert>`。由于 `nav-menu` 是 `flex:1`，其高度会根据其他 flex 项目的固定高度自动调整。

### 组件结构

新建 `src/components/layout/OfflineAlert.vue`。

通过 `useServiceStatus()` composable 获取状态，无需通过 props 层层传递。

### 展示形态

**展开态（200px）**：
- 宽度撑满侧边栏内容区，高度 36px
- 背景：`rgba(255, 77, 79, 0.1)`（淡红色）
- 边框：`1px solid rgba(255, 77, 79, 0.3)`
- 内容：左侧 WarningOutlined 图标（红色）+ "服务离线" 文字（红色，字重 500）
- 圆角：`var(--radius-md)`
- 折叠态：`margin-top: auto; margin-bottom: auto` 垂直居中

**折叠态（64px）**：
- 图标居中显示在一个 32×32 的红色圆圈内，背景同上

**连接中态**：同样显示警告卡片，内容为"连接中..."，使用 `--warning-color`，背景淡黄色，动画 pulse

**在线态**：卡片不显示（`v-if` 控制）

### 行为

- `v-if="status !== 'online'"` 控制显示
- 上线后自动消失，无手动关闭按钮（反映真实状态）
- 底部 `ServiceStatus` 小圆点保留，作为辅助提示

## 改动文件

1. `src/components/layout/OfflineAlert.vue` — 新建
2. `src/components/layout/ServiceStatus.vue` — 抽取为 `useServiceStatus()` composable（可选，或者直接在 OfflineAlert 内部调用 `getPrintServers`）
3. `src/components/layout/NavSidebar.vue` — 在 nav-menu 和 nav-bottom 之间插入 `<OfflineAlert>`
4. `src/assets/styles/nav-sidebar.css` — 可选调整样式

## 实现细节

ServiceStatus 目前通过 `setInterval(10000)` 检测连接，可直接复用该状态。建议 `OfflineAlert` 也通过同样的 `getPrintServers` 调用来检测，共享状态避免重复逻辑。可以在 `ServiceStatus` 中 `provide('serviceStatus', status)`，在 `OfflineAlert` 中 `inject('serviceStatus')`。