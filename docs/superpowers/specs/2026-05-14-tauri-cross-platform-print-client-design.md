# Tauri 2 跨平台打印客户端 — 设计规格书

**日期:** 2026-05-14
**目标:** Windows + Android 双平台打印客户端，对接已有 Java Print Service

---

## 1. 需求概述

| 维度 | 决策 |
|------|------|
| 平台 | Windows (PC) + Android，功能对等 |
| 后端 | 已有 Java Print Service（独立服务器部署，端口 2024），HTTP + WebSocket |
| 功能 | 文件管理 + 预览(PDF/IMG/TEXT/HTML) + 打印 + 打印历史(含 CSV 导出) |
| 存储 | 文件纯本地（`app_data_dir/files/`），打印历史 SQLite，配置 plugin-store(.json) |
| 认证 | 不需要登录 |
| UI 库 | antd 6.x + Vue 3 Composition API |

---

## 2. 整体架构

**方案 B — 平台自适应双布局：** 单一代码库，共享所有业务逻辑层，运行时检测平台并切换布局策略。

- **桌面端：** 侧边栏导航 + 左右分栏主内容区
- **移动端：** 底部 antd Tabs 导航 + 全屏单页操作

### 2.1 项目目录结构

```
src/
├── main.ts                          # 入口，注册 Router/Pinia
├── App.vue                          # 根组件，检测平台，选择布局
│
├── api/
│   ├── http-client.ts               # fetch 封装，baseURL 可配置
│   ├── print-api.ts                 # /print/* 接口
│   ├── template-api.ts              # 模板资源接口
│   ├── font-api.ts                  # 字体资源接口
│   ├── websocket-client.ts          # WebSocket 连接管理
│   └── types.ts                     # API 类型定义
│
├── stores/
│   ├── app-config.ts                # 服务地址等，持久化到 plugin-store
│   ├── printer-list.ts              # 打印机列表，内存缓存
│   ├── file-browser.ts              # 文件列表，实时读文件系统
│   ├── print-task.ts                # 当前打印任务状态（WS 驱动）
│   └── print-history.ts             # 打印历史，通过 Tauri commands 操作 SQLite
│
├── composables/
│   ├── usePlatform.ts               # 平台检测
│   ├── useFileSystem.ts             # 封装 Tauri file commands
│   ├── usePrintService.ts           # 打印流程编排
│   ├── useWebSocket.ts              # WebSocket 连接/重连
│   └── useFilePreview.ts            # 文件类型 → 预览策略
│
├── layouts/
│   ├── DesktopLayout.vue            # antd Layout.Sider + Content
│   └── MobileLayout.vue             # Header + <router-view> + 底部 Tabs
│
├── views/
│   ├── file-manager/
│   │   ├── DesktopFileManager.vue   # 左侧目录树 + 右侧文件列表
│   │   └── MobileFileManager.vue    # 全屏文件列表
│   ├── file-preview/
│   │   ├── PdfPreview.vue           # PDF.js 渲染
│   │   ├── ImagePreview.vue         # 图片缩放预览
│   │   ├── TextPreview.vue          # 纯文本展示
│   │   └── HtmlPreview.vue          # iframe 沙箱
│   ├── print/
│   │   ├── PrintDialog.vue          # 打印参数设置弹窗
│   │   ├── PrintStatusOverlay.vue   # 打印状态浮层
│   │   └── PrinterSelector.vue      # 打印机下拉选择
│   ├── history/
│   │   └── PrintHistory.vue         # 历史列表 + 筛选 + 导出
│   └── settings/
│       └── Settings.vue             # 服务地址配置
│
├── components/
│   ├── FileIcon.vue                 # 扩展名 → 图标
│   ├── FileListItem.vue             # 文件列表项
│   └── EmptyState.vue               # 空状态
│
└── utils/
    ├── platform.ts                  # 平台检测逻辑
    ├── file-types.ts                # 文件类型判断
    └── export-csv.ts                # 打印历史导出 CSV

src-tauri/src/
├── main.rs                          # 入口
├── lib.rs                           # Builder 配置 + command 注册
├── db.rs                            # AppState + SQLite 初始化 + 迁移
├── entities.rs                      # PrintJob 实体
├── repos.rs                         # PrintJobRepo CRUD
└── commands.rs                      # 8 个 Tauri commands
```

### 2.2 技术栈

| 层 | 选型 | 原因 |
|----|------|------|
| 框架 | Vue 3 + Composition API | 项目已有 |
| 路由 | Vue Router (hash 模式) | Tauri WebView 兼容 |
| 状态 | Pinia | Vue 3 官方推荐 |
| UI | antd 6.x | 项目已有，桌面端主力 |
| PDF 预览 | pdfjs-dist (PDF.js) | 浏览器端渲染 |
| 文件访问 | Tauri Rust comands | 跨平台文件系统抽象 |
| 网络 | fetch + WebSocket | 原生 API |
| 存储-历史 | rusqlite (bundled) | 已实现 |
| 存储-配置 | tauri-plugin-store | 已实现 |
| 文件对话框 | tauri-plugin-dialog | 已实现 |

---

## 3. 前端路由设计

### 3.1 路由表

```
路径                        桌面端                          移动端
────────────────────────── ────────────────────────────── ──────────────────────
/                          重定向到 /files                 重定向到 /files
/files                     文件管理（侧边栏导航）           文件管理（Tab: 文件）
/files/preview/:id         文件预览（主内容区）             全屏预览
/history                   打印历史（侧边栏导航）           打印历史（Tab: 历史）
/settings                  设置（侧边栏导航）               设置（Tab: 设置）
```

### 3.2 平台检测 & 布局切换

`App.vue` 在 `onMounted` 时通过 `usePlatform()` 检测 Tauri 环境：

```
App.vue
├── platform === "desktop" → DesktopLayout.vue
│   ├── antd Layout.Sider (collapsible, 文件/历史/设置)
│   ├── Layout.Content → <router-view>
│   └── PrintStatusOverlay.vue (全局)
└── platform === "mobile" → MobileLayout.vue
    ├── Header (页面标题)
    ├── <router-view>
    ├── PrintStatusOverlay.vue (全局)
    └── antd Tabs (文件 / 历史 / 设置)
```

---

## 4. 状态管理 & 数据流

### 4.1 Store 设计

| Store | 职责 | 持久化方式 |
|-------|------|-----------|
| `app-config` | Java 服务地址(host:port)、默认打印机偏好 | plugin-store (.json) |
| `printer-list` | 打印机名称数组、默认打印机 | 内存缓存 |
| `file-browser` | 当前目录文件列表、选中文件、排序方式 | 无 |
| `print-task` | 当前打印状态、sessionId、WS 连接状态 | 无 |
| `print-history` | 历史记录、筛选条件 | SQLite (Tauri commands) |

### 4.2 打印核心流程

```
用户点击"打印"
  │
  ▼
PrintDialog.vue     ← 用户确认参数 (type, source, copies, color, paperSize, ...)
  │
  ▼
usePrintService.ts  ← 编排
  │
  ├─ 1. 确保 WebSocket 已连接
  │     ws = useWebSocket.connect(serviceUrl)
  │     ws.onmessage → 解析 JSON，更新 print-task.status:
  │       200000 → "preparing"   200001 → "printing"
  │       200002 → "done"        200003 → "error"
  │       200004 → "data_sent"   200005 → "needs_attention"
  │       200006 → "failed"      200007 → "cancelled"
  │
  ├─ 2. 获取 sessionId（WS 连接成功后首条消息返回）
  │
  ├─ 3. POST /print/single (multipart/form-data)
  │     source=blob → Tauri file_read() → Blob
  │     source=path → 直接传路径字符串
  │     source=url  → 直接传 URL
  │     source=text → 直接传文本
  │
  └─ 4. 打印结束后 → print-history.create(record) 写入 SQLite
```

### 4.3 WebSocket 重连策略

| 重试次数 | 延迟 |
|---------|------|
| 第 1 次 | 立即 |
| 第 2-5 次 | 指数退避：1s → 2s → 4s → 8s |
| 第 6 次+ | 每 30s 重试 |

重连成功后旧 sessionId 作废，需用新 sessionId 发起打印。

---

## 5. 文件预览设计

| 文件类型 | 预览方案 | 组件 |
|---------|---------|------|
| PDF | pdfjs-dist 渲染 Canvas | PdfPreview.vue |
| 图片(JPEG/PNG) | `<img>` 原生渲染，支持缩放和拖动 | ImagePreview.vue |
| 纯文本 | `<pre>` 标签展示，等宽字体 | TextPreview.vue |
| HTML | `<iframe sandbox>` 隔离渲染 | HtmlPreview.vue |

预览入口：文件列表中点击文件名 → 路由到 `/files/preview/:id`（移动端）或右侧面板（桌面端）。

---

## 6. 打印历史设计

### 6.1 数据模型

`PrintJob` 实体（后续迭代扩展字段）：

```rust
// src-tauri/src/entities.rs (当前版本)
pub struct PrintJob {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}
```

后续按需扩展：`printer`（打印机名）、`type`（打印类型）、`status`（状态码）、`copies`（份数）等。
导出为 CSV 时，前端调用 `print_jobs_list` 获取数据，通过 `export-csv.ts` 生成 CSV 文件并触发下载。

---

## 7. 错误处理策略

| 错误类型 | 处理方式 |
|---------|---------|
| Java 服务不可达 | antd message.error 提示"无法连接打印服务"，显示服务地址 |
| WebSocket 断开 | 自动重连 + antd Notification 提示"连接已断开，正在重连..." |
| 打印失败 (200003/200006) | PrintStatusOverlay 显示红色错误信息，提供"重试"按钮 |
| 文件读取失败 | Tauri command 返回 Err → antd message.error 显示具体错误 |
| 打印机不存在 | 服务端返回失败 → 提示用户检查打印机名称 |
| 首次启动无服务地址 | Settings 页面自动弹出，引导用户配置 |

---

## 8. 未覆盖的补充设计

### 8.1 文件类型支持范围

`file-types.ts` 根据扩展名判断类型：

| 打印 type | 文件扩展名 |
|-----------|----------|
| PDF | `.pdf` |
| IMG | `.jpg`, `.jpeg`, `.png`, `.bmp`, `.gif` |
| TEXT | `.txt`, `.log`, `.csv`, `.json`, `.xml` |
| HTML | `.html`, `.htm` |

### 8.2 包依赖清单

**前端新增：**
- `vue-router` — 路由
- `pinia` — 状态管理
- `pdfjs-dist` — PDF 预览
- `@tauri-apps/plugin-store` — 配置存储
- `@tauri-apps/plugin-dialog` — 文件对话框

**Rust 已实现：**
- `rusqlite` (bundled) — SQLite
- `tauri-plugin-store` — KV 配置
- `tauri-plugin-dialog` — 文件对话框
- `base64` — 文件编码传输
