# Plan 6: 配置/日志/历史页面

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现三个完整功能页面：系统配置（三个 Tab）、系统日志查看器、打印历史（含 CSV 导出），对齐截图设计。

**Architecture:** 页面通过 Pinia stores 管理数据，配置持久化到 plugin-store，日志和历史通过 Tauri commands 读写 SQLite。

**Tech Stack:** Vue 3, antd 6.x, Pinia, TypeScript

**Dependencies:** Plan 1（Router + Layout）、Plan 2（system_logs 表 + 日志 commands）

**Files to create:**
- `src/views/settings/FileFormatTab.vue` — 文件格式配置
- `src/views/settings/PrintSettingsTab.vue` — 打印设置配置
- `src/views/settings/SystemSettingsTab.vue` — 系统设置配置
- `src/views/log/SystemLog.vue` — 系统日志页面
- `src/stores/print-history.ts` — 打印历史 store
- `src/stores/system-log.ts` — 系统日志 store
- `src/stores/settings.ts` — 配置 store
- `src/utils/export-csv.ts` — CSV 导出工具

**Files to modify:**
- `src/views/settings/Settings.vue` — 从占位替换为 Tab 布局
- `src/views/history/PrintHistory.vue` — 从占位替换为实际实现
- `src/router/index.ts` — 添加日志页路由

---

### Task 1: 配置 Store

**Files:** Create `src/stores/settings.ts`

- [ ] **Step 1: 实现 settings store**

管理所有可配置项，持久化到 plugin-store。

```typescript
import { defineStore } from 'pinia'
import { ref, reactive } from 'vue'

export const useSettings = defineStore('settings', () => {
  // 文件格式配置：按分类的扩展名开关
  const allowedExtensions = reactive({
    document: {
      '.htm': true, '.html': true, '.txt': true, '.doc': true,
      '.rtf': true, '.pdf': true, '.docx': true, '.xml': true, '.odt': true,
    },
    spreadsheet: {
      '.ods': true, '.csv': true, '.xlsx': true, '.xls': true,
    },
    presentation: {
      '.pptx': true, '.odp': true, '.ppt': true,
    },
    image: {
      '.tiff': true, '.png': true, '.webp': true, '.gif': true,
      '.jpeg': true, '.svg': true, '.jpg': true, '.bmp': true,
    },
  })

  // 打印设置
  const defaultPrinter = ref('')
  const defaultPaperSize = ref('ISO_A4')
  const defaultCopies = ref(1)
  const defaultColor = ref(false)
  const defaultDirection = ref('PORTRAIT')

  // 系统设置
  const lanPort = ref(5000)
  const logLevel = ref('INFO')
  const autoStart = ref(false)

  // 获取所有允许的扩展名列表
  function getAllowedExtList(): string[] {
    const result: string[] = []
    for (const category of Object.values(allowedExtensions)) {
      for (const [ext, enabled] of Object.entries(category)) {
        if (enabled) result.push(ext)
      }
    }
    return result
  }

  async function loadFromStore() { /* plugin-store 读取 */ }
  async function saveToStore() { /* plugin-store 写入 */ }

  return {
    allowedExtensions, defaultPrinter, defaultPaperSize,
    defaultCopies, defaultColor, defaultDirection,
    lanPort, logLevel, autoStart,
    getAllowedExtList, loadFromStore, saveToStore,
  }
})
```

- [ ] **Step 2: Commit**

```bash
git add src/stores/settings.ts
git commit -m "feat(stores): add settings store with file format, print, and system config"
```

---

### Task 2: 系统配置页面 — 文件格式 Tab

**Files:** Create `src/views/settings/FileFormatTab.vue`, Modify `src/views/settings/Settings.vue`

- [ ] **Step 1: 实现 Settings.vue 为 Tab 容器**

使用 antd Tabs 组件，三个 Tab：
- 文件格式（FileFormatTab）
- 打印设置（PrintSettingsTab）
- 系统设置（SystemSettingsTab）

顶部标题「系统配置」，右上角「保存配置」按钮。

参考截图第二张的布局。

- [ ] **Step 2: 实现 FileFormatTab.vue**

参考截图：按「文档格式」「表格格式」「演示文稿格式」「图片格式」分组。每个扩展名一行：蓝色 Checkbox + 扩展名（粉色高亮）+ 描述文字。

使用 antd Checkbox 组件。数据绑定到 `settings.allowedExtensions`。

布局：4 列网格（桌面端），2 列（移动端）。

- [ ] **Step 3: Commit**

```bash
git add src/views/settings/
git commit -m "feat(settings): implement Settings page with file format tab"
```

---

### Task 3: 打印设置与系统设置 Tab

**Files:** Create `src/views/settings/PrintSettingsTab.vue`, `src/views/settings/SystemSettingsTab.vue`

- [ ] **Step 1: PrintSettingsTab.vue**

表单项：
- 默认打印机：antd Select（选项从 printer-list store 获取）
- 纸张大小：antd Select（ISO_A4 等）
- 打印份数：antd InputNumber
- 默认彩色：antd Switch
- 打印方向：antd Radio.Group（纵向/横向）

- [ ] **Step 2: SystemSettingsTab.vue**

表单项：
- LAN 服务端口：antd InputNumber（默认 5000）
- 日志级别：antd Select（DEBUG/INFO/WARN/ERROR）
- 开机自启：antd Switch

使用 antd Form 组件统一表单验证。

- [ ] **Step 3: Commit**

```bash
git add src/views/settings/
git commit -m "feat(settings): add print settings and system settings tabs"
```

---

### Task 4: 系统日志 Store 与页面

**Files:** Create `src/stores/system-log.ts`, `src/views/log/SystemLog.vue`

- [ ] **Step 1: 创建 system-log store**

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface LogEntry {
  id: number
  timestamp: string
  level: string
  category: string
  message: string
  logger: string
}

export const useSystemLog = defineStore('system-log', () => {
  const logs = ref<LogEntry[]>([])
  const loading = ref(false)
  const totalCount = ref(0)

  // 过滤条件
  const filterLevel = ref<string | null>(null)
  const filterCategory = ref<string | null>(null)
  const filterKeyword = ref('')
  const displayLimit = ref(100)

  async function fetchLogs() {
    loading.value = true
    try {
      logs.value = await invoke<LogEntry[]>('log_query', {
        level: filterLevel.value,
        category: filterCategory.value,
        keyword: filterKeyword.value || null,
        limit: displayLimit.value,
      })
    } finally {
      loading.value = false
    }
  }

  async function clearLogs() {
    await invoke('log_clear')
    await fetchLogs()
  }

  return {
    logs, loading, totalCount,
    filterLevel, filterCategory, filterKeyword, displayLimit,
    fetchLogs, clearLogs,
  }
})
```

- [ ] **Step 2: 实现 SystemLog.vue**

参考截图第三张的布局：

**顶栏：**
- 标题「系统日志」
- 分类快捷按钮组：全部 / 服务 / 打印 / 上传 / 系统（参考截图的彩色标签按钮）
- 刷新按钮

**过滤区：**
- 日志级别：antd Select（全部/INFO/WARN/ERROR/DEBUG）
- 分类：antd Select（全部/service/print/upload/system）
- 搜索关键词：antd Input
- 显示行数：antd Select（100行/500行/1000行）

**日志内容区：**
- 深色背景（`background: #1a1a2e` 或类似）
- 等宽字体（`font-family: 'Consolas', monospace`）
- 每行一条日志，JSON 格式化显示
- 右上角显示总行数 + 「自动滚动」开关

使用 `ref` + `scrollIntoView` 实现自动滚动到最新日志。

- [ ] **Step 3: 添加日志路由**

在 `router/index.ts` 中添加 `/log` 路由。

在 DesktopLayout 侧边栏和 MobileLayout 底部 Tabs 中添加「系统日志」入口。

- [ ] **Step 4: Commit**

```bash
git add src/stores/system-log.ts src/views/log/ src/router/index.ts src/layouts/
git commit -m "feat(log): implement system log viewer with filtering and auto-scroll"
```

---

### Task 5: 打印历史 Store 与 CSV 导出

**Files:** Create `src/stores/print-history.ts`, `src/utils/export-csv.ts`

- [ ] **Step 1: 创建 print-history store**

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface PrintJobRecord {
  id: number
  name: string
  status: string
  printer: string
  print_type: string
  source: string
  copies: number
  created_at: string
  finished_at: string | null
  error_msg: string
}

export const usePrintHistory = defineStore('print-history', () => {
  const records = ref<PrintJobRecord[]>([])
  const loading = ref(false)

  // 筛选条件
  const filterStatus = ref<string | null>(null)
  const filterPrinter = ref<string | null>(null)
  const filterDateRange = ref<[string, string] | null>(null)

  async function fetchRecords() {
    loading.value = true
    try {
      // 调用 Tauri command 获取所有记录
      const all = await invoke<PrintJobRecord[]>('print_jobs_list')
      records.value = all
    } finally {
      loading.value = false
    }
  }

  const filteredRecords = computed(() => {
    return records.value.filter(r => {
      if (filterStatus.value && r.status !== filterStatus.value) return false
      if (filterPrinter.value && r.printer !== filterPrinter.value) return false
      return true
    })
  })

  return { records, loading, filterStatus, filterPrinter, filterDateRange, filteredRecords, fetchRecords }
})
```

- [ ] **Step 2: 创建 export-csv.ts**

```typescript
export function exportCSV(headers: string[], rows: string[][], filename: string) {
  const bom = '\uFEFF' // UTF-8 BOM for Excel compatibility
  const csvContent = bom + [
    headers.join(','),
    ...rows.map(row => row.map(cell => `"${cell.replace(/"/g, '""')}"`).join(',')),
  ].join('\n')

  const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.click()
  URL.revokeObjectURL(url)
}
```

- [ ] **Step 3: Commit**

```bash
git add src/stores/print-history.ts src/utils/export-csv.ts
git commit -m "feat(history): add print history store and CSV export utility"
```

---

### Task 6: 打印历史页面

**Files:** Modify `src/views/history/PrintHistory.vue`

- [ ] **Step 1: 实现 PrintHistory.vue**

使用 antd Table 组件：

列定义：
| 列 | 字段 | 宽度 |
|----|------|------|
| 文件名 | name | 200px |
| 状态 | status | 100px（带 Tag 颜色：done=green, failed=red, printing=blue） |
| 打印机 | printer | 150px |
| 类型 | print_type | 80px |
| 来源 | source | 80px（desktop/mobile Tag） |
| 份数 | copies | 60px |
| 创建时间 | created_at | 160px |
| 完成时间 | finished_at | 160px |
| 操作 | — | 100px（重新打印 / 删除） |

页面顶部：
- 标题「打印历史」
- 筛选器：状态 Select + 打印机 Select + 日期范围 DatePicker
- 「导出 CSV」按钮
- 「刷新」按钮

- [ ] **Step 2: pnpm build 验证**

```bash
pnpm build
```

- [ ] **Step 3: Commit**

```bash
git add src/views/history/PrintHistory.vue
git commit -m "feat(history): implement print history page with table, filter, and CSV export"
```

---

### 验收标准

1. `pnpm build` 编译通过
2. **系统配置页**：三个 Tab 可切换，文件格式按分组展示 Checkbox 开关
3. **系统配置页**：点击「保存配置」后配置持久化（刷新后保留）
4. **系统日志页**：能展示日志列表，支持按级别/分类过滤
5. **系统日志页**：深色背景等宽字体，「自动滚动」开关可用
6. **打印历史页**：Table 展示所有打印记录，状态用彩色 Tag 区分
7. **打印历史页**：「导出 CSV」按钮可下载 CSV 文件，Excel 打开不乱码
8. 所有页面在桌面布局和移动布局下均可正常显示
