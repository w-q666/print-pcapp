<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Card, Button, Empty, Spin } from 'ant-design-vue'
import { PrinterOutlined, ReloadOutlined } from '@ant-design/icons-vue'
import BasePage from '../../components/layout/BasePage.vue'
import FileUploadZone from '../../components/FileUploadZone.vue'
import FileListItem from '../../components/FileListItem.vue'
import QrCodeCard from '../../components/QrCodeCard.vue'
import SystemStatusCard from '../../components/SystemStatusCard.vue'
import PrinterStatusCard from '../../components/PrinterStatusCard.vue'
import PrintDialog from '../print/PrintDialog.vue'
import { useFileBrowser } from '../../stores/file-browser'
import { getFileType } from '../../utils/file-types'

const fileBrowser = useFileBrowser()

const printDialogOpen = ref(false)
const printFileName = ref('')
const printFilePath = ref('')

function handlePrint(fileName: string) {
  const fileType = getFileType(fileName)
  if (!fileType) return
  printFileName.value = fileName
  printFilePath.value = fileName
  printDialogOpen.value = true
}

function handlePreview(name: string) {
  console.log('Preview:', name)
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
        <Card title="文件列表" size="small">
          <Spin v-if="fileBrowser.loading" />
          <Empty v-else-if="fileBrowser.sortedFiles.length === 0" description="暂无文件，请上传" />
          <div v-else class="file-list">
            <div v-for="file in fileBrowser.sortedFiles" :key="file.name" class="file-row">
              <FileListItem
                :file-name="file.name"
                @preview="handlePreview"
                @delete="handleDelete"
              />
              <Button
                type="primary"
                size="small"
                ghost
                @click="handlePrint(file.name)"
              >
                <template #icon><PrinterOutlined /></template>
                打印
              </Button>
            </div>
          </div>
        </Card>
      </div>

      <div class="home-aside">
        <QrCodeCard />
        <SystemStatusCard />
        <PrinterStatusCard />
      </div>
    </div>

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
  grid-template-columns: 1fr 280px;
  gap: 16px;
}

.home-main {
  min-width: 0;
}

.home-aside {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.file-list {
  display: flex;
  flex-direction: column;
}

.file-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-row :deep(.file-list-item) {
  flex: 1;
}

@media (max-width: 680px) {
  .home-grid {
    grid-template-columns: 1fr;
  }
}
</style>
