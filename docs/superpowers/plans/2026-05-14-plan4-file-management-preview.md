# Plan 4: 文件管理与预览

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现文件上传（拖拽 + 点击）、文件列表浏览、文件类型校验、多格式文件预览（PDF/图片/文本/HTML）。

**Architecture:** 文件存储在 Tauri `app_data_dir/files/` 目录。上传通过 Tauri command `file_save` 写入。预览根据文件扩展名选择对应预览组件。

**Tech Stack:** Vue 3, antd 6.x (Upload/List/Modal), pdfjs-dist, TypeScript

**Dependencies:** Plan 1（需要 Router、Pinia、Layout 已就绪）

**Files to create:**
- `src/utils/file-types.ts` — 文件类型判断与校验
- `src/composables/useFileSystem.ts` — 封装 Tauri file commands
- `src/composables/useFilePreview.ts` — 文件预览策略
- `src/stores/file-browser.ts` — 文件列表 store
- `src/views/file-manager/DesktopFileManager.vue` — 桌面文件管理
- `src/views/file-manager/MobileFileManager.vue` — 移动文件管理
- `src/views/file-preview/PdfPreview.vue` — PDF 预览
- `src/views/file-preview/ImagePreview.vue` — 图片预览
- `src/views/file-preview/TextPreview.vue` — 文本预览
- `src/views/file-preview/HtmlPreview.vue` — HTML 预览
- `src/components/FileUploadZone.vue` — 拖拽上传区
- `src/components/FileIcon.vue` — 文件图标
- `src/components/FileListItem.vue` — 文件列表项

**Files to modify:**
- `package.json` — 添加 pdfjs-dist
- `src/views/file-manager/FileManager.vue` — 从占位替换为实际实现
- `src/router/index.ts` — 添加预览路由

---

### Task 1: 安装 PDF 预览依赖

**Files:** `package.json`

- [ ] **Step 1: 安装 pdfjs-dist**

```bash
pnpm add pdfjs-dist
```

- [ ] **Step 2: Commit**

```bash
git add package.json pnpm-lock.yaml
git commit -m "chore(deps): add pdfjs-dist for PDF preview"
```

---

### Task 2: 文件类型工具

**Files:** Create `src/utils/file-types.ts`

- [ ] **Step 1: 实现文件类型映射**

```typescript
export type PrintableType = 'PDF' | 'IMG' | 'TEXT' | 'HTML'

const typeMap: Record<string, PrintableType> = {
  '.pdf': 'PDF',
  '.jpg': 'IMG', '.jpeg': 'IMG', '.png': 'IMG', '.bmp': 'IMG', '.gif': 'IMG',
  '.tiff': 'IMG', '.webp': 'IMG', '.svg': 'IMG',
  '.txt': 'TEXT', '.log': 'TEXT', '.csv': 'TEXT', '.json': 'TEXT', '.xml': 'TEXT',
  '.html': 'HTML', '.htm': 'HTML',
}

export function getFileType(fileName: string): PrintableType | null {
  const ext = fileName.substring(fileName.lastIndexOf('.')).toLowerCase()
  return typeMap[ext] ?? null
}

export function isSupported(fileName: string): boolean {
  return getFileType(fileName) !== null
}

export function getFileExtension(fileName: string): string {
  return fileName.substring(fileName.lastIndexOf('.')).toLowerCase()
}

export function getPreviewComponent(type: PrintableType): string {
  switch (type) {
    case 'PDF': return 'PdfPreview'
    case 'IMG': return 'ImagePreview'
    case 'TEXT': return 'TextPreview'
    case 'HTML': return 'HtmlPreview'
  }
}

// 所有支持的扩展名，按分类
export const fileCategories = {
  document: ['.htm', '.html', '.txt', '.doc', '.rtf', '.pdf', '.docx', '.xml', '.odt'],
  spreadsheet: ['.ods', '.csv', '.xlsx', '.xls'],
  presentation: ['.pptx', '.odp', '.ppt'],
  image: ['.tiff', '.png', '.webp', '.gif', '.jpeg', '.svg', '.jpg', '.bmp'],
} as const
```

- [ ] **Step 2: Commit**

```bash
git add src/utils/file-types.ts
git commit -m "feat(utils): add file type detection and category mapping"
```

---

### Task 3: 文件系统 composable

**Files:** Create `src/composables/useFileSystem.ts`

- [ ] **Step 1: 封装 Tauri file commands**

```typescript
import { invoke } from '@tauri-apps/api/core'

export function useFileSystem() {
  async function saveFile(name: string, bytes: Uint8Array): Promise<string> {
    return invoke<string>('file_save', { name, bytes: Array.from(bytes) })
  }

  async function readFile(name: string): Promise<string> {
    return invoke<string>('file_read', { name })
  }

  async function deleteFile(name: string): Promise<void> {
    return invoke<void>('file_delete', { name })
  }

  async function listFiles(): Promise<string[]> {
    return invoke<string[]>('file_list')
  }

  // 将 base64 转为 Blob URL（用于预览）
  function base64ToBlobUrl(base64: string, mimeType: string): string {
    const binary = atob(base64)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
    const blob = new Blob([bytes], { type: mimeType })
    return URL.createObjectURL(blob)
  }

  return { saveFile, readFile, deleteFile, listFiles, base64ToBlobUrl }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/composables/useFileSystem.ts
git commit -m "feat(composables): add useFileSystem wrapping Tauri file commands"
```

---

### Task 4: 文件浏览 Store

**Files:** Create `src/stores/file-browser.ts`

- [ ] **Step 1: 创建 file-browser store**

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useFileSystem } from '../composables/useFileSystem'

export interface FileItem {
  name: string
  extension: string
  selected: boolean
}

export const useFileBrowser = defineStore('file-browser', () => {
  const files = ref<FileItem[]>([])
  const loading = ref(false)
  const selectedFile = ref<string | null>(null)
  const sortBy = ref<'name' | 'extension'>('name')

  const { listFiles, deleteFile } = useFileSystem()

  async function refresh() {
    loading.value = true
    try {
      const names = await listFiles()
      files.value = names.map(name => ({
        name,
        extension: name.substring(name.lastIndexOf('.')).toLowerCase(),
        selected: false,
      }))
    } finally {
      loading.value = false
    }
  }

  async function remove(name: string) {
    await deleteFile(name)
    await refresh()
  }

  const sortedFiles = computed(() => {
    return [...files.value].sort((a, b) => {
      if (sortBy.value === 'extension') return a.extension.localeCompare(b.extension)
      return a.name.localeCompare(b.name)
    })
  })

  return { files, loading, selectedFile, sortBy, sortedFiles, refresh, remove }
})
```

- [ ] **Step 2: Commit**

```bash
git add src/stores/file-browser.ts
git commit -m "feat(stores): add file-browser store with list, sort, delete"
```

---

### Task 5: 上传组件

**Files:** Create `src/components/FileUploadZone.vue`

- [ ] **Step 1: 实现拖拽 + 点击上传区**

使用 antd Upload.Dragger 组件：
- 支持拖拽和点击选择文件
- 上传前校验文件扩展名（`isSupported()`）
- 自定义上传逻辑：读取 File → `file_save` command
- 上传成功后刷新文件列表

关键 props:
- `accept`：根据 `fileCategories` 生成
- `beforeUpload`：校验并手动处理（不走默认 HTTP 上传）
- 显示上传进度（对于大文件）

界面参考截图：大区域居中，云上传图标，「拖拽文件到此处或点击选择文件」，「支持 PDF, Word, Excel, 图片, CAD, 设计软件等多种格式」，蓝色「选择文件」按钮。

- [ ] **Step 2: Commit**

```bash
git add src/components/FileUploadZone.vue
git commit -m "feat(components): add FileUploadZone with drag-drop and validation"
```

---

### Task 6: 文件列表项与图标组件

**Files:** Create `src/components/FileIcon.vue`, `src/components/FileListItem.vue`

- [ ] **Step 1: 创建 FileIcon.vue**

根据扩展名返回对应 antd 图标或颜色标记：
- PDF → 红色文件图标
- IMG → 绿色图片图标
- TEXT → 灰色文本图标
- HTML → 蓝色代码图标
- 其他 → 默认文件图标

- [ ] **Step 2: 创建 FileListItem.vue**

文件列表中单行项：图标 + 文件名 + 操作按钮（预览、打印、删除）。

- [ ] **Step 3: Commit**

```bash
git add src/components/FileIcon.vue src/components/FileListItem.vue
git commit -m "feat(components): add FileIcon and FileListItem"
```

---

### Task 7: 文件预览组件

**Files:** Create preview components

- [ ] **Step 1: PdfPreview.vue**

使用 `pdfjs-dist`：
1. 接收文件名 prop
2. 通过 `file_read` 获取 base64
3. 转为 ArrayBuffer → `pdfjsLib.getDocument()`
4. 渲染到 Canvas（支持翻页、缩放）

配置 PDF.js worker：
```typescript
import * as pdfjsLib from 'pdfjs-dist'
pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  'pdfjs-dist/build/pdf.worker.mjs',
  import.meta.url
).href
```

- [ ] **Step 2: ImagePreview.vue**

1. `file_read` → base64 → `data:image/xxx;base64,...`（或 Blob URL）
2. `<img>` 展示，支持缩放（CSS transform）和拖动

- [ ] **Step 3: TextPreview.vue**

1. `file_read` → base64 → decode UTF-8 text
2. `<pre>` 等宽字体展示
3. 行号显示（可选）

- [ ] **Step 4: HtmlPreview.vue**

1. `file_read` → base64 → decode HTML string
2. `<iframe sandbox="allow-same-origin">` 隔离渲染
3. srcdoc 属性注入 HTML 内容

- [ ] **Step 5: Commit**

```bash
git add src/views/file-preview/
git commit -m "feat(preview): add PDF, Image, Text, HTML preview components"
```

---

### Task 8: 文件管理主页面

**Files:** Modify `src/views/file-manager/FileManager.vue`, create Desktop/Mobile variants

- [ ] **Step 1: 实现 FileManager.vue**

根据平台选择布局：
- 桌面：上方上传区 + 下方文件列表（antd Table），点击文件名弹出预览 Modal 或右侧面板
- 移动：全屏文件列表，点击进入全屏预览

组件组合：FileUploadZone + 文件列表（使用 FileListItem）+ 预览弹窗

- [ ] **Step 2: 添加预览路由**

在 `router/index.ts` 中添加：
```typescript
{
  path: '/files/preview/:name',
  name: 'file-preview',
  component: () => import('../views/file-manager/FileManager.vue'),
  props: true,
}
```

或使用 Modal 方式（桌面端推荐），不需要额外路由。

- [ ] **Step 3: pnpm build 验证**

```bash
pnpm build
```

- [ ] **Step 4: Commit**

```bash
git add src/views/file-manager/ src/router/index.ts
git commit -m "feat(file-manager): implement file upload, list, and preview page"
```

---

### 验收标准

1. `pnpm build` 编译通过
2. 拖拽文件到上传区可成功保存到 `app_data_dir/files/`
3. 点击「选择文件」按钮可通过系统文件对话框选择文件
4. 不支持的文件扩展名被拒绝并提示
5. 文件列表展示所有已上传文件，显示文件名和类型图标
6. 点击文件名可预览：PDF 渲染正常、图片显示正常、文本显示正常、HTML 在 iframe 中渲染
7. 可删除已上传文件
