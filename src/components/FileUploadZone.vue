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
