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

/** 同时进行的保存任务数，避免大量文件时 IPC 与主线程过载 */
const MAX_CONCURRENT = 2

const acceptExtensions = [
  ...fileCategories.pdf,
  ...fileCategories.image,
  ...fileCategories.text,
].join(',')

const queue: File[] = []
let active = 0
let batchOk = 0
let batchFail = 0
const batchFailNames: string[] = []

function resetBatchStats() {
  batchOk = 0
  batchFail = 0
  batchFailNames.length = 0
}

function enqueue(file: File) {
  if (active === 0 && queue.length === 0) {
    resetBatchStats()
  }
  queue.push(file)
  uploading.value = true
  pump()
}

async function finishBatch() {
  uploading.value = false
  await fileBrowser.refresh()
  if (batchOk > 0 && batchFail === 0) {
    message.success(batchOk === 1 ? `上传成功` : `已成功上传 ${batchOk} 个文件`)
  } else if (batchOk > 0 && batchFail > 0) {
    message.warning(`上传完成：成功 ${batchOk} 个，失败 ${batchFail} 个`)
  } else if (batchFail > 0) {
    const sample = batchFailNames.slice(0, 5).join('、')
    const more = batchFailNames.length > 5 ? ` 等共 ${batchFail} 个` : ''
    message.error(`上传失败${more}: ${sample}`)
  }
}

function pump() {
  while (active < MAX_CONCURRENT && queue.length > 0) {
    const file = queue.shift()!
    active++
    void uploadOne(file).finally(() => {
      active--
      if (active === 0 && queue.length === 0) {
        void finishBatch()
      }
      pump()
    })
  }
}

async function uploadOne(file: File) {
  try {
    const buffer = await file.arrayBuffer()
    const bytes = new Uint8Array(buffer)
    await saveFile(file.name, bytes)
    batchOk++
  } catch (e) {
    batchFail++
    batchFailNames.push(file.name)
    console.error('[FileUploadZone] save failed', file.name, e)
  }
}

function beforeUpload(file: File) {
  if (!isSupported(file.name)) {
    message.error(`不支持的文件格式: ${file.name}`)
    return false
  }
  enqueue(file)
  return false
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
