# 未对接接口前端集成设计

> 日期：2026-05-14
> 状态：待实现

## 概述

将 Java Print 服务的 6 类未对接接口集成到现有前端 UI 中，包括：模版资源管理（6 个模版端点 + 字体资源端点）、条码打印、获取服务端路径。

### 目标

- 完成所有 Java Print HTTP 接口的前端对接
- 新增"模版中心"页面，支持浏览/预览/数据填充/打印完整流程
- 新增条码标签打印功能（独立入口 + 文件关联入口）
- 在系统配置中展示 Java 服务端运行路径

### 不在范围内

- Java 服务端的 HTML 打印功能实现（后端尚未完成，前端做好对接准备）
- 代理转发（`/**`）的前端集成（服务端内部路由，不需要前端调用）
- 模版文件的创建/编辑/上传（模版由 Java 服务端管理）

---

## 1. 导航与路由

### 路由变更

新增一条顶级路由：

| 路径 | 名称 | 组件 | meta.title | meta.icon |
|------|------|------|-----------|-----------|
| `/templates` | `templates` | `TemplateCenter.vue` | 模版中心 | `SnippetsOutlined` |

路由顺序调整为：

```
/files        → 文件管理      (FolderOutlined)
/templates    → 模版中心      (SnippetsOutlined)     ← 新增
/print        → 打印任务      (PrinterOutlined)      ← 扩展条码打印
/history      → 打印历史      (HistoryOutlined)
/log          → 系统日志      (FileTextOutlined)
/settings     → 系统配置      (SettingOutlined)       ← 扩展 getPath
```

### 侧边栏导航

主导航区从 4 项变为 5 项（"模版中心"排在"文件管理"之后）。底部"系统配置"不变。

### 移动端底部 Tab

底部 Tab 保持 5 项不变（文件管理、打印任务、打印历史、系统日志、系统配置）。模版中心在移动端通过打印任务页面顶部的"模版中心"链接按钮进入，不占据底部 Tab 位。

---

## 2. 模版中心页面 (`/templates`)

### 布局

采用与文件管理页面一致的两栏网格布局（`grid-template-columns: 1fr 380px`）。

```
┌──────────────────────────────────────────────────────────┐
│ BasePage: "模版中心"               [刷新] [字体管理]       │
├──────────────────────────────────────┬───────────────────┤
│  模版分类 Tabs                        │  模版预览区        │
│  ┌─────────────────────────────────┐ │  ┌───────────────┐│
│  │委托│报告│原始记录│不合格│封皮│费用│ │  │  iframe 渲染   ││
│  └─────────────────────────────────┘ │  │  模版 HTML     ││
│                                      │  │               ││
│  模版文件列表                         │  │               ││
│  ┌─────────────────────────────────┐ │  └───────────────┘│
│  │ 📄 template1.html  [预览][打印] │ │                   │
│  │ 📄 template2.html  [预览][打印] │ │  数据填充          │
│  │ 📄 template3.html  [预览][打印] │ │  ┌───────────────┐│
│  └─────────────────────────────────┘ │  │ 动态表单字段    ││
│                                      │  │ [填充并预览]    ││
│  模版文件配置                         │  │ [发送打印]      ││
│  ┌─────────────────────────────────┐ │  └───────────────┘│
│  │ [+ 添加模版文件名]              │ │                   │
│  └─────────────────────────────────┘ │                   │
└──────────────────────────────────────┴───────────────────┘
```

### 模版分类

6 个分类对应 6 个 Java 服务端接口端点：

| Tab 名称 | API 路径前缀 | 资源目录 |
|----------|-------------|---------|
| 委托模版 | `/powerAttorney/` | `public/powerAttorney/` |
| 报告模版 | `/report/` | `public/report/` |
| 原始记录 | `/originalRecords/` | `public/originalRecords/` |
| 不合格报告 | `/public/unquaLedger/` | `public/unquaLedger/` |
| 封皮模版 | `/cover/` | `public/cover/` |
| 费用列表 | `/feeList/` | `public/feeList/` |

### 模版文件列表管理

Java 服务端没有提供"列出目录下所有模版文件"的接口，因此模版文件名列表在**前端维护**：

- 存储位置：Tauri Store (`settings.json`) 的 `templateFiles` 字段
- 数据结构：

```typescript
interface TemplateConfig {
  [category: string]: string[]  // category → 文件名数组
}

// 示例
{
  "powerAttorney": ["template1.html", "default.html"],
  "report": ["standard.html", "simplified.html"],
  "originalRecords": ["record.html"],
  "unquaLedger": ["notice.html"],
  "cover": ["cover-a4.html"],
  "feeList": ["fee-standard.html"]
}
```

- 每个分类 Tab 下方提供"添加模版文件名"的输入框 + 添加按钮
- 文件名可删除（Popconfirm 确认）
- 初始默认为空列表，用户按实际情况配置

### 模版预览

- 选中某个模版文件 → 调用 `GET /{category}/{filename}` 获取 HTML 内容
- 在右侧区域用 `<iframe srcdoc>` 渲染（复用现有 HtmlPreview 的沙箱模式）
- iframe 中自动注入字体 `@font-face` 声明（基于 `/fonts/list` 返回的字体列表）
- 预览区顶部显示文件名和文件加载状态

### 数据填充

模版 HTML 中的占位符格式约定为 `{{fieldName}}`（双花括号）。

- 前端解析模版 HTML 内容，用正则 `/\{\{(\w+)\}\}/g` 提取所有占位符字段名
- 在右侧下方动态生成对应的 antd `Form` 表单（每个占位符一个 `Input` 字段，label 为字段名）
- 用户填写后点击"填充并预览" → 执行 `html.replace(/\{\{fieldName\}\}/g, value)` → 更新 iframe 预览
- 如果模版中无占位符，数据填充区域不显示

### 打印流程

1. 用户点击"发送打印" → 弹出 PrintDialog（复用现有打印设置弹窗）
2. 确认后调用 `printSingle()` ：
   - `type: 'HTML'`
   - `source: 'text'`
   - `content:` 填充后的 HTML 字符串
3. 打印状态通过 WebSocket 推送到 PrintStatusOverlay（复用现有流程）

> 注：Java 服务端 HTML 打印尚未完整实现，调用后服务端只输出日志。前端做好完整对接，等后端实现即可。

---

## 3. 条码打印

### 组件：`BarcodePrintModal.vue`

独立的 Modal 对话框组件，可从多处触发。

#### 布局

```
┌──────────────── 条码标签打印 ────────────────┐
│                                              │
│  Form (layout="vertical")                    │
│                                              │
│  检测项目名称:  [Input ________________ ]    │
│  条码编码:      [Input ________________ ]    │
│  规格型号:      [Input ________________ ]    │
│                                              │
│  ───── 标签预览 ─────                        │
│  ┌────────────────────────────────┐          │
│  │  {checkName}                    │          │
│  │  {spec}                         │          │
│  │  ||||||||||||||||||||||||||||    │          │
│  │  {code}                         │          │
│  │  状态:待检√在检_已检_备份_      │          │
│  └────────────────────────────────┘          │
│                                              │
│                 [取消]    [打印]              │
└──────────────────────────────────────────────┘
```

#### 表单字段

| 字段 | 组件 | 必填 | 说明 |
|------|------|------|------|
| `checkName` | `Input` | 是 | 检测项目名称，如"钢筋原材" |
| `code` | `Input` | 是 | 条码编码，如"GJYC-2024-001" |
| `spec` | `Input` | 是 | 规格型号，如"HRB400E Ø 初检" |

#### 标签预览

实时预览区域，使用 CSS 模拟标签布局（450×250 比例缩放）。内容随表单输入实时更新。条码部分用 CSS 模拟条纹图案（不需要真正生成条码，仅作视觉参考）。

#### 打印调用

点击"打印" → 表单校验 → 调用 `POST /print/barCodePrint`（FormData 格式）：

```typescript
{ checkName: string, code: string, spec: string }
```

响应为空（void），HTTP 状态码 200 表示请求已发送（不代表打印成功）。前端收到 200 后显示 `message.success('条码打印指令已发送')`。如果请求失败（网络错误或 HTTP 非 200），显示 `message.error('条码打印失败')`。实际打印结果通过服务端控制台日志查看。

### 入口 1：打印任务页面

在 `PrintQueue.vue` 的 BasePage `#actions` slot 新增"条码打印"按钮（`BarcodeOutlined` 图标）。点击打开 `BarcodePrintModal`，三个字段为空，由用户手动填写。

移动端额外在此处新增"模版中心"链接按钮（通过 `usePlatform().isMobile` 判断显示），点击跳转 `/templates` 路由。

### 入口 2：文件管理页面

在 `FileListItem` 的操作按钮组中新增"标签"图标按钮（`TagOutlined`），排在"打印"按钮之后。

点击后打开 `BarcodePrintModal`，自动尝试从文件名解析填充：
- `checkName`：取文件名的第一个 `-` 或 `_` 之前的部分
- `code`：取文件名（去掉扩展名）
- `spec`：留空，由用户手动填写

用户可修改所有预填字段。

---

## 4. 字体资源

### 字体列表 API

调用 `GET /fonts/any`（任意非 `.ttf` 后缀的参数）返回字体列表 JSON：

```json
{
  "path": "",
  "descriptors": [
    { "name": "微软雅黑", "source": "../fonts/SimHei.ttf" },
    { "name": "宋体", "source": "../fonts/song.ttf" },
    { "name": "黑体", "source": "../fonts/SimHei.ttf" }
  ]
}
```

### 模版预览中的字体加载

在模版预览的 iframe 中注入 `<style>` 标签：

```css
@font-face {
  font-family: '微软雅黑';
  src: url('http://{serviceHost}:{servicePort}/fonts/SimHei.ttf');
}
@font-face {
  font-family: '宋体';
  src: url('http://{serviceHost}:{servicePort}/fonts/song.ttf');
}
/* ... */
```

字体列表在模版中心页面加载时请求一次，缓存在内存中。

### 字体管理弹窗

模版中心 header 的"字体管理"按钮 → 弹出 Modal：

- 只读信息展示：字体列表表格（名称 | 文件路径 | 状态）
- 每个字体尝试加载对应的 `.ttf` 文件，成功显示 ✅，失败显示 ❌
- 用于运维排查字体缺失问题

---

## 5. 获取服务端路径 (getPath)

### 集成位置

系统配置页 → 系统设置 Tab → 新增"服务信息"分组。

### 布局

在现有的 LAN 端口/日志级别/开机自启表单之后，用 `Divider` 分隔，新增：

```
───── 服务信息 ─────
Java 服务地址:  http://localhost:2024   ● 在线
Java 运行路径:  D:\JavaPrint\           [刷新]
```

### 调用方式

`GET /print/getPath/{sessionId}` 的结果通过 WebSocket 推送（不是 HTTP 响应）。调用流程：

1. 确保 WebSocket 已连接（复用 `useWebSocket` 的 `sessionId`）
2. 调用 `GET /print/getPath/{sessionId}`
3. 监听 WebSocket 的 `onMessage`，收到的纯文本字符串即为路径
4. 显示在 UI 中，缓存在内存

注意：需要在 `useWebSocket` 的 `onMessage` 中区分"打印状态推送"和"路径信息推送"。打印状态推送是 JSON 格式（`{code, msg}`），路径信息是纯文本字符串。通过 JSON.parse 尝试解析来区分。

### 刷新

点击"刷新"按钮重新调用 `getPath`，更新显示。

---

## 6. API 层变更

### 新增 API 函数 (`src/api/print-api.ts`)

```typescript
// 模版资源
export async function getTemplate(category: string, filename: string): Promise<string>
// category 映射到 URL 路径：
// powerAttorney → /powerAttorney/{filename}
// report → /report/{filename}
// originalRecords → /originalRecords/{filename}
// unquaLedger → /public/unquaLedger/{filename}
// cover → /cover/{filename}
// feeList → /feeList/{filename}

// 字体
export async function getFontList(): Promise<FontListResponse>
// GET /fonts/list（任意非 .ttf 参数）

// 条码打印
export async function barCodePrint(params: BarcodePrintRequest): Promise<void>
// POST /print/barCodePrint (FormData)

// 服务端路径
export async function getServerPath(sessionId: string): Promise<void>
// GET /print/getPath/{sessionId}（结果通过 WebSocket 返回）
```

### 新增类型 (`src/api/types.ts`)

```typescript
export interface FontDescriptor {
  name: string
  source: string
}

export interface FontListResponse {
  path: string
  descriptors: FontDescriptor[]
}

export interface BarcodePrintRequest {
  checkName: string
  code: string
  spec: string
}

export type TemplateCategory =
  | 'powerAttorney'
  | 'report'
  | 'originalRecords'
  | 'unquaLedger'
  | 'cover'
  | 'feeList'

export const TemplateCategoryLabels: Record<TemplateCategory, string> = {
  powerAttorney: '委托模版',
  report: '报告模版',
  originalRecords: '原始记录',
  unquaLedger: '不合格报告',
  cover: '封皮模版',
  feeList: '费用列表',
}

export const TemplateCategoryPaths: Record<TemplateCategory, string> = {
  powerAttorney: '/powerAttorney',
  report: '/report',
  originalRecords: '/originalRecords',
  unquaLedger: '/public/unquaLedger',
  cover: '/cover',
  feeList: '/feeList',
}
```

### 注意事项

部分接口的响应**不是标准的 `CommonResult<T>` 格式**，需要在 `http-client.ts` 中新增方法：

- `requestRaw(path)` → 返回 `response.text()`，用于**模版内容接口**（6 个模版端点返回纯 HTML 字符串）
- `requestJson<T>(path)` → 返回 `response.json()` 但不按 `CommonResult` 解析，用于**字体列表接口**（返回 `{path, descriptors}` 格式的 JSON，不是 `{code, msg, data}` 格式）
- 条码打印接口 (`barCodePrint`) 响应为空（void），使用 `postFormData` 但忽略响应体，只检查 HTTP 状态码

---

## 7. Store 层变更

### 新增 Store：`template-center`

```typescript
// src/stores/template-center.ts
{
  // 持久化到 Tauri Store
  templateFiles: Record<TemplateCategory, string[]>  // 每个分类的模版文件名列表

  // 内存状态
  currentCategory: TemplateCategory     // 当前选中的分类 Tab
  currentFile: string | null            // 当前选中的模版文件名
  templateContent: string | null        // 当前模版 HTML 内容
  filledContent: string | null          // 数据填充后的 HTML
  fontList: FontDescriptor[]            // 字体列表（缓存）
  loading: boolean
  error: string | null
}
```

### 扩展现有 Store

- `app-config`：新增 `serverPath: string`（Java 服务端运行路径缓存）

---

## 8. 新增文件清单

| 文件路径 | 类型 | 说明 |
|----------|------|------|
| `src/views/templates/TemplateCenter.vue` | 页面 | 模版中心主页面 |
| `src/views/templates/TemplateList.vue` | 组件 | 左栏：模版文件列表 + 分类 Tab |
| `src/views/templates/TemplatePreview.vue` | 组件 | 右栏：模版预览 iframe |
| `src/views/templates/TemplateDataForm.vue` | 组件 | 右栏：数据填充动态表单 |
| `src/views/templates/FontManagerModal.vue` | 组件 | 字体管理弹窗 |
| `src/views/print/BarcodePrintModal.vue` | 组件 | 条码打印弹窗 |
| `src/stores/template-center.ts` | Store | 模版中心状态管理 |

### 修改文件清单

| 文件路径 | 变更说明 |
|----------|----------|
| `src/router/index.ts` | 新增 `/templates` 路由 |
| `src/components/layout/NavSidebar.vue` | 新增"模版中心"菜单项 |
| `src/layouts/MobileLayout.vue` | 无需修改（模版中心不加入底部 Tab） |
| `src/views/print/PrintQueue.vue` | BasePage actions 新增"条码打印"按钮 |
| `src/views/home/HomePage.vue` | FileListItem 操作栏新增"标签"按钮 |
| `src/views/settings/SystemSettingsTab.vue` | 新增"服务信息"分组（getPath） |
| `src/api/print-api.ts` | 新增 4 个 API 函数 |
| `src/api/http-client.ts` | 新增 `requestRaw()` 方法 |
| `src/api/types.ts` | 新增模版/字体/条码相关类型 |
| `src/stores/app-config.ts` | 新增 `serverPath` 字段 |
| `src/composables/useWebSocket.ts` | 扩展 onMessage 区分路径推送 |

---

## 9. UI 组件规范

所有新增 UI 严格遵循现有项目约定：

- 全部使用 Ant Design Vue 4.x 组件，不使用原生 HTML 表单/排版元素
- 页面用 `<BasePage>` 包裹，操作按钮放 `#actions` slot
- 图标使用 `@ant-design/icons-vue`
- 消息反馈：`message.success/error()`，危险操作用 `Popconfirm`
- 加载状态：按钮 `:loading`，列表 `<Spin :spinning>`
- 样式：`<style scoped>` + CSS 变量引用设计 tokens
- 响应式：680px / 900px 断点

---

## 10. 错误处理

| 场景 | 处理方式 |
|------|----------|
| 模版文件不存在（404） | `message.error('模版文件不存在: {filename}')` |
| 字体文件加载失败 | 静默降级，字体管理弹窗中标记 ❌ |
| 条码打印调用失败 | `message.error('条码打印失败')` |
| getPath WebSocket 未连接 | 先尝试建立连接，超时后提示"请确保 Java 打印服务已启动" |
| HTML 打印调用后无状态推送 | 前端 10 秒超时提示"HTML 打印功能暂未实现，请联系管理员" |
