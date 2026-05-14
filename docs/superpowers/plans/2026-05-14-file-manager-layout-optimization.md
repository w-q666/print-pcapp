# File Management Layout Optimization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Merge FileManager features into HomePage, compact the upload zone, inline print into file rows, add file metadata, and rearrange right-sidebar cards into a mixed layout.

**Architecture:** Backend-first: add a `FileInfo` struct to Rust, update `file_list` to return metadata. Then modify the frontend data layer (composable + store), upgrade leaf components (UploadZone, FileListItem, cards), and finally assemble everything in HomePage.vue. Delete the now-redundant FileManager.vue.

**Tech Stack:** Rust (Tauri 2 commands, rusqlite, serde), Vue 3 + TypeScript (Composition API, Pinia), Ant Design Vue 4.x

---

### Task 1: Add FileInfo entity and update file_list command

**Files:**
- Modify: `src-tauri/src/entities.rs`
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 1: Add FileInfo struct to entities.rs**

Add after the existing `LogQuery` struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub modified_at: u64,
}
```

`modified_at` is a Unix timestamp in seconds — the frontend will format it.

- [ ] **Step 2: Update file_list command in commands.rs**

Replace the existing `file_list` function (lines 169-188) with:

```rust
#[tauri::command]
pub fn file_list(app_handle: AppHandle) -> Result<Vec<crate::entities::FileInfo>, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let files_dir = data_dir.join("files");
    if !files_dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    let entries = fs::read_dir(&files_dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            let meta = entry.metadata().map_err(|e| e.to_string())?;
            let size = meta.len();
            let modified_at = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            files.push(crate::entities::FileInfo {
                name,
                size,
                modified_at,
            });
        }
    }
    Ok(files)
}
```

Remove the `use std::path::Path;` import (line 2) if it's no longer needed — actually, keep it, `safe_filename` still uses `Path`.

- [ ] **Step 3: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compilation succeeds with no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/entities.rs src-tauri/src/commands.rs
git commit -m "feat: add FileInfo struct and return file metadata from file_list"
```

---

### Task 2: Update frontend data layer for file metadata

**Files:**
- Modify: `src/composables/useFileSystem.ts`
- Modify: `src/stores/file-browser.ts`

- [ ] **Step 1: Add FileInfo type and update useFileSystem**

In `src/composables/useFileSystem.ts`, add the interface and update `listFiles` return type:

```typescript
import { invoke } from '@tauri-apps/api/core'

export interface FileInfo {
  name: string
  size: number
  modified_at: number  // Unix timestamp in seconds
}

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

  async function listFiles(): Promise<FileInfo[]> {
    return invoke<FileInfo[]>('file_list')
  }

  function base64ToBlobUrl(base64: string, mimeType: string): string {
    const binary = atob(base64)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
    const blob = new Blob([bytes], { type: mimeType })
    return URL.createObjectURL(blob)
  }

  function base64ToArrayBuffer(base64: string): ArrayBuffer {
    const binary = atob(base64)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
    return bytes.buffer
  }

  function base64ToText(base64: string): string {
    const binary = atob(base64)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
    return new TextDecoder('utf-8').decode(bytes)
  }

  return { saveFile, readFile, deleteFile, listFiles, base64ToBlobUrl, base64ToArrayBuffer, base64ToText }
}
```

- [ ] **Step 2: Update file-browser store**

Replace `src/stores/file-browser.ts` entirely:

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useFileSystem, type FileInfo } from '../composables/useFileSystem'

export interface FileItem {
  name: string
  extension: string
  size: number
  modified_at: number  // Unix timestamp seconds
  dateLabel: string     // Formatted for display
}

export const useFileBrowser = defineStore('file-browser', () => {
  const files = ref<FileItem[]>([])
  const loading = ref(false)
  const sortBy = ref<'name' | 'extension'>('name')

  const { listFiles, deleteFile } = useFileSystem()

  async function refresh() {
    loading.value = true
    try {
      const infos: FileInfo[] = await listFiles()
      files.value = infos.map(info => ({
        name: info.name,
        extension: info.name.substring(info.name.lastIndexOf('.')).toLowerCase(),
        size: info.size,
        modified_at: info.modified_at,
        dateLabel: info.modified_at > 0
          ? new Date(info.modified_at * 1000).toLocaleString('zh-CN', {
              year: 'numeric', month: '2-digit', day: '2-digit',
              hour: '2-digit', minute: '2-digit',
            })
          : '',
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
      return a.name.localeCompare(b.name, 'zh')
    })
  })

  return { files, loading, sortBy, sortedFiles, refresh, remove }
})
```

- [ ] **Step 3: Verify TypeScript compiles**

Run: `pnpm build 2>&1`
Expected: Build succeeds with no TS errors.

- [ ] **Step 4: Commit**

```bash
git add src/composables/useFileSystem.ts src/stores/file-browser.ts
git commit -m "feat: update frontend data layer for FileInfo with size and modified_at"
```

---

### Task 3: Compact FileUploadZone

**File:** `src/components/FileUploadZone.vue`

- [ ] **Step 1: Replace FileUploadZone with horizontal compact layout**

Replace the entire file:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { message, UploadDragger } from 'ant-design-vue'
import { CloudUploadOutlined } from '@ant-design/icons-vue'
import { isSupported, fileCategories } from '../utils/file-types'
import { useFileSystem } from '../composables/useFileSystem'
import { useFileBrowser } from '../stores/file-browser'

const { saveFile } = useFileSystem()
const fileBrowser = useFileBrowser()
const uploading = ref(false)

const acceptExtensions = [
  ...fileCategories.pdf,
  ...fileCategories.image,
  ...fileCategories.text,
].join(',')

function beforeUpload(file: File) {
  if (!isSupported(file.name)) {
    message.error(`不支持的文件格式: ${file.name}`)
    return false
  }
  handleUpload(file)
  return false
}

async function handleUpload(file: File) {
  uploading.value = true
  try {
    const buffer = await file.arrayBuffer()
    const bytes = new Uint8Array(buffer)
    await saveFile(file.name, bytes)
    message.success(`${file.name} 上传成功`)
    await fileBrowser.refresh()
  } catch (e) {
    message.error(`上传失败: ${e}`)
  } finally {
    uploading.value = false
  }
}
</script>

<template>
  <UploadDragger
    :accept="acceptExtensions"
    :multiple="true"
    :show-upload-list="false"
    :before-upload="beforeUpload"
    :disabled="uploading"
    class="upload-zone"
  >
    <div class="upload-content">
      <CloudUploadOutlined class="upload-icon" />
      <span class="upload-title">拖拽文件到此处或点击上传</span>
      <span class="upload-separator">|</span>
      <span class="upload-hint">支持 PDF、图片、纯文本、HTML</span>
    </div>
  </UploadDragger>
</template>

<style scoped>
.upload-zone {
  margin-bottom: 0;
}

.upload-zone :deep(.ant-upload-drag) {
  padding: 0 !important;
}

.upload-content {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 12px 18px;
}

.upload-icon {
  font-size: 24px;
  color: var(--primary-color, #1677ff);
  flex-shrink: 0;
}

.upload-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary, rgba(0, 0, 0, 0.85));
}

.upload-separator {
  color: #d9d9d9;
  font-size: 14px;
}

.upload-hint {
  font-size: 12px;
  color: var(--text-secondary, rgba(0, 0, 0, 0.45));
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/FileUploadZone.vue
git commit -m "refactor: compact horizontal FileUploadZone layout"
```

---

### Task 4: Upgrade FileListItem with metadata and print action

**File:** `src/components/FileListItem.vue`

- [ ] **Step 1: Replace FileListItem with enhanced two-row layout**

Replace the entire file:

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { Button, Popconfirm } from 'ant-design-vue'
import { EyeOutlined, DeleteOutlined, PrinterOutlined } from '@ant-design/icons-vue'
import FileIcon from './FileIcon.vue'
import { getFileType } from '../utils/file-types'

const props = defineProps<{
  fileName: string
  fileSize?: number
  fileDate?: string
}>()

const emit = defineEmits<{
  preview: [name: string]
  delete: [name: string]
  print: [name: string]
}>()

const canPreview = computed(() => getFileType(props.fileName) !== null)

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<template>
  <div class="file-list-item">
    <div class="file-info">
      <FileIcon :file-name="fileName" />
      <div class="file-text">
        <span class="file-name">{{ fileName }}</span>
        <span class="file-meta">
          <template v-if="fileSize !== undefined">{{ formatSize(fileSize) }}</template>
          <template v-if="fileDate"> · {{ fileDate }}</template>
        </span>
      </div>
    </div>
    <div class="file-actions">
      <Button v-if="canPreview" type="text" size="small" @click="emit('preview', fileName)">
        <template #icon><EyeOutlined /></template>
        预览
      </Button>
      <Popconfirm
        title="确定删除此文件？"
        ok-text="删除"
        cancel-text="取消"
        @confirm="emit('delete', fileName)"
      >
        <Button type="text" size="small" danger>
          <template #icon><DeleteOutlined /></template>
          删除
        </Button>
      </Popconfirm>
      <Button
        type="primary"
        size="small"
        ghost
        @click="emit('print', fileName)"
      >
        <template #icon><PrinterOutlined /></template>
        打印
      </Button>
    </div>
  </div>
</template>

<style scoped>
.file-list-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid #f5f5f5;
  transition: background-color 0.2s;
}

.file-list-item:last-child {
  border-bottom: none;
}

.file-list-item:hover {
  background-color: var(--ant-control-item-bg-hover, #f5f5f5);
}

.file-info {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  flex: 1;
}

.file-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.file-name {
  font-size: 13px;
  font-weight: 500;
  color: rgba(0, 0, 0, 0.85);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-meta {
  font-size: 11px;
  color: rgba(0, 0, 0, 0.45);
  margin-top: 1px;
}

.file-actions {
  display: flex;
  gap: 4px;
  align-items: center;
  flex-shrink: 0;
  margin-left: 12px;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/FileListItem.vue
git commit -m "feat: upgrade FileListItem with metadata, preview gate, and inline print"
```

---

### Task 5: Compact SystemStatusCard and PrinterStatusCard

**Files:**
- Modify: `src/components/SystemStatusCard.vue`
- Modify: `src/components/PrinterStatusCard.vue`

- [ ] **Step 1: Replace SystemStatusCard with compact version**

Replace `src/components/SystemStatusCard.vue` entirely:

```vue
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const queueCount = ref(0)
const todayCount = ref(0)
const loading = ref(true)
let timer: ReturnType<typeof setInterval> | null = null

async function fetchCounts() {
  try {
    const [queue, today] = await Promise.all([
      invoke<number>('print_jobs_count_queue'),
      invoke<number>('print_jobs_count_today'),
    ])
    queueCount.value = queue
    todayCount.value = today
  } catch (e) {
    console.warn('Failed to fetch system status:', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchCounts()
  timer = setInterval(fetchCounts, 10000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<template>
  <div class="status-card">
    <div class="status-row">
      <div class="status-item">
        <span class="status-value">{{ loading ? '-' : queueCount }}</span>
        <span class="status-label">排队</span>
      </div>
      <div class="status-divider" />
      <div class="status-item">
        <span class="status-value">{{ loading ? '-' : todayCount }}</span>
        <span class="status-label">今日完成</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.status-card {
  background: linear-gradient(135deg, #722ed1, #b37feb);
  border-radius: 8px;
  padding: 14px 10px;
  text-align: center;
}

.status-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0;
}

.status-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.status-value {
  font-size: 22px;
  font-weight: 700;
  color: #fff;
  line-height: 1.2;
}

.status-label {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.7);
  margin-top: 2px;
}

.status-divider {
  width: 1px;
  height: 32px;
  background: rgba(255, 255, 255, 0.25);
}
</style>
```

- [ ] **Step 2: Replace PrinterStatusCard with compact version**

Replace `src/components/PrinterStatusCard.vue` entirely:

```vue
<script setup lang="ts">
import { onMounted } from 'vue'
import { Tag, Button, Empty } from 'ant-design-vue'
import { ReloadOutlined } from '@ant-design/icons-vue'
import { usePrinterList } from '../stores/printer-list'

const printerList = usePrinterList()

onMounted(() => {
  if (printerList.printers.length === 0) {
    printerList.refresh()
  }
})
</script>

<template>
  <div class="printer-card">
    <div class="printer-header">
      <span class="printer-title">打印机</span>
      <Button type="text" size="small" :loading="printerList.loading" @click="printerList.refresh()">
        <template #icon><ReloadOutlined /></template>
      </Button>
    </div>

    <Empty v-if="printerList.printers.length === 0 && !printerList.loading" description="未发现打印机" :image-style="{ height: '40px' }" />

    <div v-else class="printer-list">
      <div v-for="name in printerList.printers" :key="name" class="printer-item">
        <span class="printer-dot" :class="{ active: name === printerList.defaultPrinter }" />
        <span class="printer-name">{{ name }}</span>
        <Tag :color="name === printerList.defaultPrinter ? 'blue' : 'default'" class="printer-tag">
          {{ name === printerList.defaultPrinter ? '默认' : '就绪' }}
        </Tag>
      </div>
    </div>
  </div>
</template>

<style scoped>
.printer-card {
  border: 1px solid #f0f0f0;
  border-radius: 8px;
  padding: 10px 12px;
  background: #fff;
}

.printer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.printer-title {
  font-size: 13px;
  font-weight: 600;
  color: rgba(0, 0, 0, 0.85);
}

.printer-list {
  display: flex;
  flex-direction: column;
}

.printer-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 0;
  border-bottom: 1px solid #f5f5f5;
}

.printer-item:last-child {
  border-bottom: none;
}

.printer-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #d9d9d9;
  flex-shrink: 0;
}

.printer-dot.active {
  background: #52c41a;
}

.printer-name {
  font-size: 12px;
  color: rgba(0, 0, 0, 0.85);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.printer-tag {
  font-size: 10px;
  line-height: 1;
  margin: 0;
}
</style>
```

- [ ] **Step 3: Commit**

```bash
git add src/components/SystemStatusCard.vue src/components/PrinterStatusCard.vue
git commit -m "refactor: compact SystemStatusCard and PrinterStatusCard for sidebar"
```

---

### Task 6: Rewrite HomePage with full integration

**File:** `src/views/home/HomePage.vue`

- [ ] **Step 1: Replace HomePage.vue with integrated version**

Replace the entire file:

```vue
<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { Button, Empty, Spin, Modal, RadioGroup, RadioButton } from 'ant-design-vue'
import { ReloadOutlined, PrinterOutlined } from '@ant-design/icons-vue'
import BasePage from '../../components/layout/BasePage.vue'
import FileUploadZone from '../../components/FileUploadZone.vue'
import FileListItem from '../../components/FileListItem.vue'
import QrCodeCard from '../../components/QrCodeCard.vue'
import SystemStatusCard from '../../components/SystemStatusCard.vue'
import PrinterStatusCard from '../../components/PrinterStatusCard.vue'
import PrintDialog from '../print/PrintDialog.vue'
import PdfPreview from '../file-preview/PdfPreview.vue'
import ImagePreview from '../file-preview/ImagePreview.vue'
import TextPreview from '../file-preview/TextPreview.vue'
import HtmlPreview from '../file-preview/HtmlPreview.vue'
import { useFileBrowser } from '../../stores/file-browser'
import { getFileType } from '../../utils/file-types'

const fileBrowser = useFileBrowser()

const previewVisible = ref(false)
const previewFileName = ref('')

const previewType = computed(() => {
  if (!previewFileName.value) return null
  return getFileType(previewFileName.value)
})

function openPreview(name: string) {
  previewFileName.value = name
  previewVisible.value = true
}

const printDialogOpen = ref(false)
const printFileName = ref('')
const printFilePath = ref('')

function openPrint(name: string) {
  printFileName.value = name
  printFilePath.value = name
  printDialogOpen.value = true
}

async function handleDelete(name: string) {
  await fileBrowser.remove(name)
}

onMounted(() => {
  fileBrowser.refresh()
})
</script>

<template>
  <BasePage title="文件管理">
    <template #actions>
      <Button size="small" @click="fileBrowser.refresh()" :loading="fileBrowser.loading">
        <template #icon><ReloadOutlined /></template>
        刷新
      </Button>
    </template>

    <div class="home-grid">
      <div class="home-main">
        <FileUploadZone />

        <div class="file-toolbar">
          <span class="file-count">共 <b>{{ fileBrowser.sortedFiles.length }}</b> 个文件</span>
          <RadioGroup v-model:value="fileBrowser.sortBy" size="small">
            <RadioButton value="name">按名称</RadioButton>
            <RadioButton value="extension">按类型</RadioButton>
          </RadioGroup>
        </div>

        <Spin :spinning="fileBrowser.loading">
          <Empty v-if="fileBrowser.sortedFiles.length === 0 && !fileBrowser.loading" description="暂无文件，请上传" />
          <div v-else class="file-list">
            <FileListItem
              v-for="file in fileBrowser.sortedFiles"
              :key="file.name"
              :file-name="file.name"
              :file-size="file.size"
              :file-date="file.dateLabel"
              @preview="openPreview"
              @delete="handleDelete"
              @print="openPrint"
            />
          </div>
        </Spin>
      </div>

      <div class="home-aside">
        <QrCodeCard />
        <div class="aside-cards-row">
          <SystemStatusCard />
          <PrinterStatusCard />
        </div>
      </div>
    </div>

    <Modal
      v-model:open="previewVisible"
      :title="previewFileName"
      :footer="null"
      width="80%"
      :styles="{ body: { height: '70vh', overflow: 'auto', padding: '16px' } }"
      destroy-on-close
    >
      <PdfPreview v-if="previewType === 'PDF'" :file-name="previewFileName" />
      <ImagePreview v-else-if="previewType === 'IMG'" :file-name="previewFileName" />
      <TextPreview v-else-if="previewType === 'TEXT'" :file-name="previewFileName" />
      <HtmlPreview v-else-if="previewType === 'HTML'" :file-name="previewFileName" />
    </Modal>

    <PrintDialog
      v-model:open="printDialogOpen"
      :file-name="printFileName"
      :file-path="printFilePath"
      @submitted="fileBrowser.refresh()"
    />
  </BasePage>
</template>

<style scoped>
.home-grid {
  display: grid;
  grid-template-columns: 1fr 260px;
  gap: 14px;
}

.home-main {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.home-aside {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.aside-cards-row {
  display: flex;
  gap: 8px;
}

.aside-cards-row > * {
  flex: 1;
  min-width: 0;
}

.file-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.file-count {
  font-size: 13px;
  color: rgba(0, 0, 0, 0.45);
}

.file-list {
  border: 1px solid #f0f0f0;
  border-radius: 8px;
  overflow: hidden;
}

@media (max-width: 900px) {
  .home-grid {
    grid-template-columns: 1fr 220px;
  }
}

@media (max-width: 680px) {
  .home-grid {
    grid-template-columns: 1fr;
  }

  .home-aside {
    display: flex;
    flex-direction: row;
    gap: 8px;
  }

  .home-aside > * {
    flex: 1;
    min-width: 0;
  }

  .aside-cards-row {
    flex: 1;
  }
}
</style>
```

- [ ] **Step 2: Build and verify**

Run: `pnpm build 2>&1`
Expected: Build succeeds with no errors.

- [ ] **Step 3: Commit**

```bash
git add src/views/home/HomePage.vue
git commit -m "feat: integrate sort, preview, compact layout into HomePage"
```

---

### Task 7: Delete FileManager.vue and final cleanup

**Files:**
- Delete: `src/views/file-manager/FileManager.vue`

- [ ] **Step 1: Delete the now-redundant FileManager.vue**

```bash
rm "src/views/file-manager/FileManager.vue"
```

Check if the `src/views/file-manager/` directory is now empty:

```bash
ls "src/views/file-manager/" 2>&1
```

If empty, remove it:
```bash
rmdir "src/views/file-manager/" 2>/dev/null || true
```

- [ ] **Step 2: Verify build still passes after deletion**

Run: `pnpm build 2>&1`
Expected: Build succeeds with no errors (FileManager was not imported by any route).

- [ ] **Step 3: Final commit**

```bash
git add src/views/file-manager/
git commit -m "chore: remove unused FileManager.vue (merged into HomePage)"
```
