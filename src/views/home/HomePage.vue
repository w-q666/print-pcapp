<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { Button, Empty, Spin, Modal, RadioGroup, RadioButton } from 'ant-design-vue'
import { ReloadOutlined } from '@ant-design/icons-vue'
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
