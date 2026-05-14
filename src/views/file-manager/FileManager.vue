<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { message, RadioGroup, RadioButton, Spin, Empty, Modal } from 'ant-design-vue'
import FileUploadZone from '../../components/FileUploadZone.vue'
import FileListItem from '../../components/FileListItem.vue'
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
  const type = getFileType(name)
  if (!type) {
    message.warning('该文件类型不支持预览')
    return
  }
  previewFileName.value = name
  previewVisible.value = true
}

async function handleDelete(name: string) {
  try {
    await fileBrowser.remove(name)
    message.success(`${name} 已删除`)
  } catch (e) {
    message.error(`删除失败: ${e}`)
  }
}

onMounted(() => {
  fileBrowser.refresh()
})
</script>

<template>
  <div class="file-manager">
    <div class="file-manager-header">
      <h2 class="page-title">文件管理</h2>
    </div>

    <FileUploadZone />

    <div class="file-list-section">
      <div class="file-list-header">
        <span class="file-count">共 {{ fileBrowser.sortedFiles.length }} 个文件</span>
        <RadioGroup v-model:value="fileBrowser.sortBy" size="small">
          <RadioButton value="name">按名称</RadioButton>
          <RadioButton value="extension">按类型</RadioButton>
        </RadioGroup>
      </div>

      <Spin :spinning="fileBrowser.loading">
        <div v-if="fileBrowser.sortedFiles.length === 0 && !fileBrowser.loading" class="empty-state">
          <Empty description="暂无文件，请上传" />
        </div>
        <div v-else class="file-list">
          <FileListItem
            v-for="file in fileBrowser.sortedFiles"
            :key="file.name"
            :file-name="file.name"
            @preview="openPreview"
            @delete="handleDelete"
          />
        </div>
      </Spin>
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
  </div>
</template>

<style scoped>
.file-manager {
  padding: 24px;
  max-width: 960px;
  margin: 0 auto;
}

.file-manager-header {
  margin-bottom: 20px;
}

.page-title {
  font-size: 22px;
  font-weight: 600;
  margin: 0;
}

.file-list-section {
  margin-top: 16px;
}

.file-list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.file-count {
  font-size: 14px;
  color: rgba(0, 0, 0, 0.45);
}

.file-list {
  border: 1px solid var(--ant-color-border-secondary, #f0f0f0);
  border-radius: 8px;
  overflow: hidden;
}

.empty-state {
  padding: 48px 0;
}
</style>
