# Plan 7: 打印执行流程与端到端集成

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 组装所有子系统为完整的可用产品：打印对话框、实时状态浮层、主界面（上传 + 侧栏二维码/统计/打印机状态），端到端打印闭环。

**Architecture:** 主界面整合 Plan 4（文件上传）、Plan 3（打印流程）、Plan 5（二维码），打印对话框编排参数选择 → 提交 → 状态跟踪 → 记录历史。

**Tech Stack:** Vue 3, antd 6.x, Pinia, TypeScript

**Dependencies:** 全部前序 Plan（Plan 1-6）

**Files to create:**
- `src/views/print/PrintDialog.vue` — 打印参数设置弹窗
- `src/views/print/PrintStatusOverlay.vue` — 打印状态实时浮层
- `src/views/print/PrinterSelector.vue` — 打印机选择组件
- `src/views/home/HomePage.vue` — 主界面（整合上传 + 侧栏）
- `src/components/QrCodeCard.vue` — 二维码展示卡片
- `src/components/SystemStatusCard.vue` — 系统状态统计卡片
- `src/components/PrinterStatusCard.vue` — 打印机状态卡片

**Files to modify:**
- `src/router/index.ts` — 调整路由（主页 → HomePage）
- `src/layouts/DesktopLayout.vue` — 集成 PrintStatusOverlay

---

### Task 1: 打印机选择组件

**Files:** Create `src/views/print/PrinterSelector.vue`

- [ ] **Step 1: 实现 PrinterSelector.vue**

antd Select 组件，选项从 `usePrinterList` store 获取。

功能：
- 显示所有可用打印机（名称列表）
- 默认选中 `defaultPrinter`
- 「刷新打印机」按钮（调用 `printerList.refresh()`）
- 加载中状态
- 错误提示（如 Java 服务不可达）

Props:
- `modelValue: string` — 当前选中打印机
- `@update:modelValue` — 选择变更

- [ ] **Step 2: Commit**

```bash
git add src/views/print/PrinterSelector.vue
git commit -m "feat(print): add PrinterSelector component"
```

---

### Task 2: 打印对话框

**Files:** Create `src/views/print/PrintDialog.vue`

- [ ] **Step 1: 实现 PrintDialog.vue**

antd Modal 弹窗，显示打印参数配置。

**弹窗触发**：从文件列表中选择文件后，点击「打印」按钮。

**表单项（使用 antd Form）：**

| 表单项 | 组件 | 默认值来源 |
|--------|------|-----------|
| 文件名 | 只读展示 | 选中的文件名 |
| 打印机 | PrinterSelector | settings.defaultPrinter |
| 纸张大小 | Select（PaperSizes 枚举） | settings.defaultPaperSize |
| 打印份数 | InputNumber (min=1, max=99) | settings.defaultCopies |
| 彩色打印 | Switch | settings.defaultColor |
| 打印方向 | Radio.Group (纵向/横向) | settings.defaultDirection |

**底部按钮**：「取消」+ 「开始打印」

点击「开始打印」后：
1. 调用 `usePrintService().print(params)`
2. 关闭 Modal
3. 显示 PrintStatusOverlay

- [ ] **Step 2: Commit**

```bash
git add src/views/print/PrintDialog.vue
git commit -m "feat(print): add PrintDialog with parameter configuration"
```

---

### Task 3: 打印状态浮层

**Files:** Create `src/views/print/PrintStatusOverlay.vue`

- [ ] **Step 1: 实现 PrintStatusOverlay.vue**

全局浮层组件（注册在 Layout 级别），显示当前打印任务的实时状态。

**状态映射（对齐 WebSocket 状态码）：**

| 状态 | 图标 | 颜色 | 文字 |
|------|------|------|------|
| connecting | LoadingOutlined (spin) | blue | 正在连接打印服务... |
| preparing | LoadingOutlined (spin) | blue | 正在准备打印... |
| printing | PrinterOutlined (spin) | blue | 正在打印... |
| data_sent | CheckCircleOutlined | green | 数据传输完成 |
| done | CheckCircleOutlined | green | 打印完成 |
| error | CloseCircleOutlined | red | 打印异常 |
| needs_attention | ExclamationCircleOutlined | orange | 需要人工干预（卡纸/缺纸等） |
| failed | CloseCircleOutlined | red | 打印失败 |
| cancelled | MinusCircleOutlined | grey | 打印已取消 |

**布局**：右下角固定浮层（antd Notification 风格），或使用 antd Alert 作为顶部横幅。

**交互**：
- 进行中：显示进度动画 + 取消按钮（可选）
- 完成/失败：3 秒后自动隐藏，或用户手动关闭
- 失败状态：显示错误消息 + 「重试」按钮

**数据源**：从 `usePrintTask` store 读取状态。

- [ ] **Step 2: Commit**

```bash
git add src/views/print/PrintStatusOverlay.vue
git commit -m "feat(print): add PrintStatusOverlay with real-time status display"
```

---

### Task 4: 侧栏组件 — 二维码/状态/打印机

**Files:** Create sidebar components

- [ ] **Step 1: QrCodeCard.vue**

参考截图右上角「手机扫码打印」卡片：
- 标题：「手机扫码打印」+ 刷新按钮
- 中央：QR 码图片（从 `lan_server_qrcode` command 获取 base64 → `<img src="data:image/png;base64,...">`)
- 说明文字：「使用手机扫描二维码 快速访问打印服务」
- URL 展示（绿色锁图标 + 地址文字）
- 「复制链接」按钮（navigator.clipboard.writeText）

- [ ] **Step 2: SystemStatusCard.vue**

参考截图「系统状态」卡片（紫色渐变背景）：
- 标题：「系统状态」
- 两列统计：
  - 「队列任务」数字（调用 `print_jobs_count_queue` command）
  - 「今日完成」数字（调用 `print_jobs_count_today` command）
- 定时刷新（每 5 秒轮询或 WebSocket 驱动）

- [ ] **Step 3: PrinterStatusCard.vue**

参考截图「打印机状态」卡片：
- 标题：「打印机状态」+ 刷新按钮
- 打印机列表（从 `usePrinterList` store 获取）
- 每个打印机显示：名称 + 状态 Tag（就绪/离线）+ 队列数
- 点击打印机可打开系统打印机管理（通过 Tauri opener 插件）

- [ ] **Step 4: Commit**

```bash
git add src/components/QrCodeCard.vue src/components/SystemStatusCard.vue src/components/PrinterStatusCard.vue
git commit -m "feat(components): add QR code, system status, and printer status sidebar cards"
```

---

### Task 5: 主界面 HomePage

**Files:** Create `src/views/home/HomePage.vue`, Modify `src/router/index.ts`

- [ ] **Step 1: 实现 HomePage.vue**

参考截图第一张的完整布局：

**桌面端布局（antd Row + Col）：**

```
┌─────────────────────────────────────┬──────────────────┐
│ 网络打印服务         [刷新打印机]    │                  │
├─────────────────────────────────────┤  手机扫码打印     │
│                                     │  [QR Code]       │
│  文件上传                            │  复制链接         │
│  ┌─────────────────────────────┐    ├──────────────────┤
│  │  拖拽文件到此处或点击选择文件  │    │  系统状态         │
│  │                             │    │  队列: 0  今日: 0 │
│  │        [选择文件]            │    ├──────────────────┤
│  └─────────────────────────────┘    │  打印机状态       │
│                                     │  - Samsung ...    │
│  [已上传文件列表]                    │  - MF4800 ...    │
│                                     │                  │
└─────────────────────────────────────┴──────────────────┘
```

左侧（Col span=16）：
- 标题 + 「刷新打印机」按钮
- FileUploadZone 组件
- 已上传文件列表（复用 Plan 4 的文件列表）

右侧（Col span=8）：
- QrCodeCard
- SystemStatusCard
- PrinterStatusCard

**移动端布局**：纵向堆叠，上传区在最上面。

- [ ] **Step 2: 调整路由**

将 `/files` 路由改为使用 HomePage.vue（或将主页与文件管理合并）。

```typescript
{
  path: '/files',
  name: 'home',
  component: () => import('../views/home/HomePage.vue'),
  meta: { title: '网络打印服务', icon: 'HomeOutlined' },
}
```

- [ ] **Step 3: Commit**

```bash
git add src/views/home/ src/router/index.ts
git commit -m "feat(home): implement main page with upload, QR code, status, and printers"
```

---

### Task 6: 端到端打印流程串联

- [ ] **Step 1: 文件列表 → 打印对话框**

在文件列表中每个文件的操作栏添加「打印」按钮。
点击后打开 PrintDialog，传入文件信息。

- [ ] **Step 2: PrintDialog → usePrintService → WebSocket**

PrintDialog 确认后：
1. 调用 `usePrintService().print()` 
2. PrintStatusOverlay 自动显示（监听 `print-task` store）
3. WebSocket 推送状态变更 → store 更新 → Overlay 实时反映

- [ ] **Step 3: 打印完成 → 写入历史**

在 `usePrintService` 中监听打印终态（done/failed/cancelled），调用 `print_jobs_update_status` command 更新数据库记录。

- [ ] **Step 4: 刷新统计**

打印完成后自动刷新 SystemStatusCard 的队列数和今日完成数。

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat(integration): wire up end-to-end print flow from file selection to history"
```

---

### Task 7: 全局错误处理与体验优化

- [ ] **Step 1: Java 服务不可达**

在 App.vue `onMounted` 中尝试连接 Java 服务：
- 失败 → antd Notification 提示「无法连接打印服务」+ 显示服务地址 + 引导用户到设置页面
- 首次启动且未配置地址 → 自动导航到 Settings 页面

- [ ] **Step 2: WebSocket 断线提示**

在 Layout 级别监听 WebSocket 连接状态：
- 断开 → 顶部 antd Alert 横幅「与打印服务的连接已断开，正在尝试重连...」
- 重连成功 → 自动消失

- [ ] **Step 3: 上传冲突处理**

如果手机和桌面同时上传同名文件，文件名自动加后缀（`file_1.pdf`, `file_2.pdf`）。

- [ ] **Step 4: Commit**

```bash
git add .
git commit -m "feat(ux): add error handling, connection alerts, and upload conflict resolution"
```

---

### Task 8: 最终集成验证

- [ ] **Step 1: 全量 build**

```bash
pnpm build
```

Expected: 零 TypeScript 错误。

- [ ] **Step 2: cargo test**

```bash
cd src-tauri && cargo test
```

Expected: 所有 Rust 测试通过。

- [ ] **Step 3: 端到端流程测试**

启动 `pnpm tauri dev`：

1. 主界面展示正确（上传区 + 侧栏二维码/统计/打印机）
2. 拖拽文件上传 → 文件列表显示
3. 点击文件「打印」→ 打印对话框弹出 → 选择参数 → 确认
4. 打印状态浮层实时更新（准备 → 打印 → 完成）
5. 打印历史页面显示记录
6. 手机扫描二维码 → 打开上传页 → 上传文件 → 桌面端文件列表刷新
7. 系统配置 → 修改文件格式 → 保存 → 上传不支持的格式被拒绝
8. 系统日志 → 显示操作日志 → 过滤可用

- [ ] **Step 4: 最终 Commit**

```bash
git add .
git commit -m "feat: complete cloud print client v1 — desktop + mobile upload integration"
```

---

### 验收标准

1. `pnpm build` + `cargo test` 全部通过
2. 主界面布局与截图一致：左侧上传 + 右侧二维码/统计/打印机
3. 端到端打印流程可完整走通（上传 → 配置 → 打印 → 状态 → 历史）
4. 手机扫码上传可用
5. WebSocket 断线自动重连 + 用户提示
6. 打印异常状态（卡纸/失败等）在 UI 上有明确反馈
7. 配置修改后持久化且立即生效
8. 日志页面可查看系统运行日志
