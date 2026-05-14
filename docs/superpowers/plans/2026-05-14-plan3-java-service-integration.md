# Plan 3: Java Print Service 对接

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现前端 API 层，对接已部署的 Java Print Service（HTTP + WebSocket），包含打印流程编排 composable。

**Architecture:** API 层独立于 UI，通过 Pinia stores 暴露状态。WebSocket 用于打印状态实时推送，HTTP 用于打印机查询和打印提交。

**Tech Stack:** TypeScript, Vue 3, Pinia, fetch API, WebSocket API

**Dependencies:** Plan 1（需要 Pinia 和 Router 已就绪）

**External Dependency:** Java Print Service 运行在 `http://{host}:2024`，参考 `docs/接口文档/HTTP接口文档.md` 和 `docs/接口文档/WebSocket接口文档.md`

**Files to create:**
- `src/api/types.ts` — API 类型定义
- `src/api/http-client.ts` — fetch 封装
- `src/api/print-api.ts` — /print/* 接口
- `src/api/websocket-client.ts` — WebSocket 连接管理
- `src/stores/printer-list.ts` — 打印机列表 store
- `src/stores/print-task.ts` — 当前打印任务状态 store
- `src/composables/useWebSocket.ts` — WebSocket 连接/重连
- `src/composables/usePrintService.ts` — 打印流程编排

---

### Task 1: API 类型定义

**Files:** Create `src/api/types.ts`

- [ ] **Step 1: 定义所有 API 类型**

```typescript
// Java Print Service 通用响应
export interface CommonResult<T = unknown> {
  code: number
  msg: string
  data?: T
}

// 打印请求参数
export interface PrintRequest {
  type: 'PDF' | 'IMG' | 'TEXT' | 'HTML'
  source: 'text' | 'path' | 'url' | 'blob'
  content?: string
  file?: File | Blob
  sessionId?: string
  copies?: number
  color?: boolean
  paperSize?: string
  direction?: 'PORTRAIT' | 'LANDSCAPE' | 'REVERSE_LANDSCAPE' | 'REVERSE_PORTRAIT'
  printServer?: string
}

// 打印状态码
export const PrintStatusCode = {
  PREPARING: 200000,
  PRINTING: 200001,
  COMPLETED: 200002,
  ERROR: 200003,
  DATA_TRANSFERRED: 200004,
  NEEDS_ATTENTION: 200005,
  FAILED: 200006,
  CANCELLED: 200007,
  FILE_NOT_FOUND: 200008,
  FILE_ERROR: 200009,
} as const

export type PrintStatus =
  | 'idle'
  | 'connecting'
  | 'connected'
  | 'preparing'
  | 'printing'
  | 'data_sent'
  | 'done'
  | 'error'
  | 'needs_attention'
  | 'failed'
  | 'cancelled'

// 纸张尺寸枚举
export const PaperSizes = [
  'ISO_A3', 'ISO_A4', 'ISO_A5', 'ISO_A6',
  'ISO_A0', 'ISO_A1', 'ISO_A2',
  'ISO_A7', 'ISO_A8', 'ISO_A9', 'ISO_A10',
  'EXECUTIVE', 'FOLIO', 'INVOICE',
] as const
```

- [ ] **Step 2: Commit**

```bash
git add src/api/types.ts
git commit -m "feat(api): add TypeScript types for Java Print Service API"
```

---

### Task 2: HTTP Client 封装

**Files:** Create `src/api/http-client.ts`

- [ ] **Step 1: 创建通用 fetch 封装**

```typescript
import type { CommonResult } from './types'

let baseURL = 'http://localhost:2024'

export function setBaseURL(url: string) {
  baseURL = url
}

export function getBaseURL(): string {
  return baseURL
}

export async function request<T>(
  path: string,
  options: RequestInit = {}
): Promise<CommonResult<T>> {
  const url = `${baseURL}${path}`
  const response = await fetch(url, {
    ...options,
    headers: {
      ...options.headers,
    },
  })

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }

  return response.json()
}

export async function get<T>(path: string): Promise<CommonResult<T>> {
  return request<T>(path, { method: 'GET' })
}

export async function postFormData<T>(
  path: string,
  data: Record<string, string | number | boolean | Blob | File | undefined>
): Promise<CommonResult<T>> {
  const formData = new FormData()
  for (const [key, value] of Object.entries(data)) {
    if (value === undefined) continue
    if (value instanceof Blob || value instanceof File) {
      formData.append(key, value)
    } else {
      formData.append(key, String(value))
    }
  }
  return request<T>(path, { method: 'POST', body: formData })
}
```

- [ ] **Step 2: Commit**

```bash
git add src/api/http-client.ts
git commit -m "feat(api): add HTTP client with fetch wrapper and FormData support"
```

---

### Task 3: 打印 API 接口

**Files:** Create `src/api/print-api.ts`

- [ ] **Step 1: 实现打印服务 API**

```typescript
import { get, postFormData } from './http-client'
import type { CommonResult, PrintRequest } from './types'

export async function getPrintServers(): Promise<string[]> {
  const result = await get<string[]>('/print/getPrintServers')
  if (result.code !== 0) throw new Error(result.msg)
  return result.data ?? []
}

export async function printSingle(req: PrintRequest): Promise<CommonResult> {
  const data: Record<string, any> = {
    type: req.type,
    source: req.source,
    sessionId: req.sessionId,
  }
  if (req.content !== undefined) data.content = req.content
  if (req.file !== undefined) data.file = req.file
  if (req.copies !== undefined) data.copies = req.copies
  if (req.color !== undefined) data.color = req.color
  if (req.paperSize !== undefined) data.paperSize = req.paperSize
  if (req.direction !== undefined) data.direction = req.direction
  if (req.printServer !== undefined) data.printServer = req.printServer

  return postFormData('/print/single', data)
}
```

- [ ] **Step 2: Commit**

```bash
git add src/api/print-api.ts
git commit -m "feat(api): add print API client (getPrintServers, printSingle)"
```

---

### Task 4: WebSocket 客户端

**Files:** Create `src/api/websocket-client.ts`, `src/composables/useWebSocket.ts`

- [ ] **Step 1: 创建底层 WebSocket 客户端**

`websocket-client.ts` 封装原生 WebSocket，提供：
- `connect(url: string): WebSocket`
- 事件回调注册
- 自动解析 JSON 消息

- [ ] **Step 2: 创建 useWebSocket composable**

`useWebSocket.ts` 提供响应式 WebSocket 管理：

```typescript
import { ref, onUnmounted } from 'vue'
import type { PrintStatus } from '../api/types'
import { PrintStatusCode } from '../api/types'

export function useWebSocket() {
  const ws = ref<WebSocket | null>(null)
  const sessionId = ref<string | null>(null)
  const status = ref<PrintStatus>('idle')
  const isConnected = ref(false)
  const lastMessage = ref<{ code: number; msg: string } | null>(null)

  let retryCount = 0
  let retryTimer: ReturnType<typeof setTimeout> | null = null

  function connect(url: string) {
    cleanup()
    status.value = 'connecting'

    const socket = new WebSocket(url)

    socket.onopen = () => {
      isConnected.value = true
      retryCount = 0
    }

    socket.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        // 首条消息是 sessionId
        if (data.code === 0 && data.data) {
          sessionId.value = data.data
          status.value = 'connected'
          return
        }
        // 后续消息是打印状态
        lastMessage.value = data
        status.value = mapStatusCode(data.code)
      } catch (e) {
        console.error('WS message parse error:', e)
      }
    }

    socket.onclose = () => {
      isConnected.value = false
      scheduleReconnect(url)
    }

    socket.onerror = () => {
      isConnected.value = false
    }

    ws.value = socket
  }

  // 指数退避重连：立即 → 1s → 2s → 4s → 8s → 每30s
  function scheduleReconnect(url: string) {
    if (retryTimer) return
    const delay = retryCount === 0 ? 0
      : retryCount <= 4 ? Math.pow(2, retryCount - 1) * 1000
      : 30000
    retryCount++
    retryTimer = setTimeout(() => {
      retryTimer = null
      connect(url)
    }, delay)
  }

  function mapStatusCode(code: number): PrintStatus {
    switch (code) {
      case PrintStatusCode.PREPARING: return 'preparing'
      case PrintStatusCode.PRINTING: return 'printing'
      case PrintStatusCode.COMPLETED: return 'done'
      case PrintStatusCode.ERROR: return 'error'
      case PrintStatusCode.DATA_TRANSFERRED: return 'data_sent'
      case PrintStatusCode.NEEDS_ATTENTION: return 'needs_attention'
      case PrintStatusCode.FAILED: return 'failed'
      case PrintStatusCode.CANCELLED: return 'cancelled'
      default: return 'idle'
    }
  }

  function cleanup() {
    if (retryTimer) { clearTimeout(retryTimer); retryTimer = null }
    if (ws.value) { ws.value.close(); ws.value = null }
  }

  function disconnect() {
    retryCount = Infinity // 阻止重连
    cleanup()
    sessionId.value = null
    status.value = 'idle'
    isConnected.value = false
  }

  onUnmounted(disconnect)

  return { connect, disconnect, sessionId, status, isConnected, lastMessage }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/api/websocket-client.ts src/composables/useWebSocket.ts
git commit -m "feat(ws): add WebSocket client with exponential backoff reconnection"
```

---

### Task 5: Pinia Stores — 打印机列表与打印任务

**Files:** Create `src/stores/printer-list.ts`, `src/stores/print-task.ts`

- [ ] **Step 1: 创建 printer-list store**

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getPrintServers } from '../api/print-api'

export const usePrinterList = defineStore('printer-list', () => {
  const printers = ref<string[]>([])
  const defaultPrinter = ref<string>('')
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function refresh() {
    loading.value = true
    error.value = null
    try {
      printers.value = await getPrintServers()
      if (!defaultPrinter.value && printers.value.length > 0) {
        defaultPrinter.value = printers.value[0]
      }
    } catch (e: any) {
      error.value = e.message || '获取打印机列表失败'
    } finally {
      loading.value = false
    }
  }

  return { printers, defaultPrinter, loading, error, refresh }
})
```

- [ ] **Step 2: 创建 print-task store**

管理当前打印任务的状态，由 WebSocket 消息驱动更新。

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { PrintStatus } from '../api/types'

export const usePrintTask = defineStore('print-task', () => {
  const currentStatus = ref<PrintStatus>('idle')
  const currentJobName = ref('')
  const statusMessage = ref('')
  const isActive = ref(false)

  function updateStatus(status: PrintStatus, msg?: string) {
    currentStatus.value = status
    if (msg) statusMessage.value = msg
    isActive.value = !['idle', 'done', 'failed', 'cancelled', 'error'].includes(status)
  }

  function reset() {
    currentStatus.value = 'idle'
    currentJobName.value = ''
    statusMessage.value = ''
    isActive.value = false
  }

  return { currentStatus, currentJobName, statusMessage, isActive, updateStatus, reset }
})
```

- [ ] **Step 3: 更新 stores/index.ts 导出**

- [ ] **Step 4: Commit**

```bash
git add src/stores/
git commit -m "feat(stores): add printer-list and print-task Pinia stores"
```

---

### Task 6: 打印流程编排 composable

**Files:** Create `src/composables/usePrintService.ts`

- [ ] **Step 1: 实现打印流程编排**

核心流程：
1. 确保 WebSocket 已连接
2. 获取 sessionId
3. 构建 PrintRequest
4. POST /print/single（对于 source=blob，从 Tauri file_read 读取并转为 Blob）
5. WebSocket 监听打印状态
6. 状态终结后 → 调用 Tauri command 写入打印历史

```typescript
import { useWebSocket } from './useWebSocket'
import { usePrintTask } from '../stores/print-task'
import { usePrinterList } from '../stores/printer-list'
import { useAppConfig } from '../stores/app-config'
import { printSingle } from '../api/print-api'
import { invoke } from '@tauri-apps/api/core'

export function usePrintService() {
  const wsClient = useWebSocket()
  const printTask = usePrintTask()
  const printerList = usePrinterList()
  const appConfig = useAppConfig()

  async function ensureConnected() {
    if (!wsClient.isConnected.value) {
      wsClient.connect(appConfig.wsUrl)
      // 等待 sessionId
      await new Promise<void>((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error('WebSocket 连接超时')), 10000)
        const unwatch = watch(() => wsClient.sessionId.value, (id) => {
          if (id) { clearTimeout(timeout); unwatch(); resolve() }
        })
      })
    }
  }

  async function print(params: {
    fileName: string
    filePath: string
    type: 'PDF' | 'IMG' | 'TEXT' | 'HTML'
    source: 'blob' | 'path' | 'url' | 'text'
    content?: string
    printer?: string
    copies?: number
    color?: boolean
    paperSize?: string
    direction?: string
  }) {
    printTask.updateStatus('connecting')
    printTask.currentJobName = params.fileName

    await ensureConnected()
    printTask.updateStatus('preparing')

    // 构建请求
    const req: any = {
      type: params.type,
      source: params.source,
      sessionId: wsClient.sessionId.value,
      copies: params.copies ?? 1,
      color: params.color ?? false,
      paperSize: params.paperSize,
      direction: params.direction,
      printServer: params.printer || printerList.defaultPrinter,
    }

    if (params.source === 'blob') {
      // 从 Tauri 读取文件 → base64 → Blob
      const base64: string = await invoke('file_read', { name: params.fileName })
      const binary = atob(base64)
      const bytes = new Uint8Array(binary.length)
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
      req.file = new Blob([bytes])
    } else if (params.source === 'path') {
      req.content = params.filePath
    } else {
      req.content = params.content
    }

    // 创建打印任务记录
    await invoke('print_jobs_create', {
      name: params.fileName,
      printer: req.printServer || '',
      printType: params.type,
      source: 'desktop',
      copies: req.copies,
      filePath: params.filePath || '',
      fileSize: 0,
    })

    // 发送打印请求
    const result = await printSingle(req)
    if (result.code !== 0) {
      printTask.updateStatus('error', result.msg)
      throw new Error(result.msg)
    }

    // 状态由 WebSocket 推送驱动更新
  }

  return { print, ensureConnected }
}
```

注意：`watch` 需要从 `vue` 导入。实际使用时需要监听 `wsClient.status` 变化并同步到 `printTask`。

- [ ] **Step 2: Commit**

```bash
git add src/composables/usePrintService.ts
git commit -m "feat(composables): add usePrintService for print flow orchestration"
```

---

### Task 7: 端到端验证

- [ ] **Step 1: pnpm build 验证无类型错误**

```bash
pnpm build
```

Expected: vue-tsc 通过。

- [ ] **Step 2: 在开发模式下验证 API 连通**

```bash
pnpm tauri dev
```

在浏览器 console 中手动测试：
1. `getPrintServers()` 返回打印机列表
2. WebSocket 连接并获取 sessionId

- [ ] **Step 3: Commit（如有修复）**

---

### 验收标准

1. `pnpm build` 编译通过，TypeScript 无错误
2. `getPrintServers()` 能从 Java 服务获取打印机列表
3. WebSocket 连接成功后获得 `sessionId`
4. WebSocket 断开后自动重连（指数退避）
5. `usePrintService().print()` 可发起打印请求并通过 WS 收到状态推送
6. 打印状态码（200000-200009）正确映射到 `PrintStatus` 枚举
7. 所有 API 类型与 Java 服务接口文档一致
