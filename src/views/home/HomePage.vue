<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Row, Col, Card, Button, Empty, Spin } from 'ant-design-vue'
import { PrinterOutlined } from '@ant-design/icons-vue'
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
  <div class="home-page">
    <Row :gutter="16">
      <Col :xs="24" :lg="16">
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
      </Col>

      <Col :xs="24" :lg="8">
        <div class="sidebar-stack">
          <QrCodeCard />
          <SystemStatusCard />
          <PrinterStatusCard />
        </div>
      </Col>
    </Row>

    <PrintDialog
      v-model:open="printDialogOpen"
      :file-name="printFileName"
      :file-path="printFilePath"
      @submitted="fileBrowser.refresh()"
    />
  </div>
</template>

<style scoped>
.home-page {
  padding: 0;
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

.sidebar-stack {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
</style>
