<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { Button, Checkbox, Empty, Spin, Modal, Popconfirm, RadioGroup, RadioButton, Pagination, message } from 'ant-design-vue'
import { ReloadOutlined, DeleteOutlined, ClearOutlined, PrinterOutlined } from '@ant-design/icons-vue'
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
const printFileNames = ref<string[]>([])

function openPrint(name: string) {
  printFileName.value = name
  printFilePath.value = name
  printFileNames.value = [name]
  printDialogOpen.value = true
}

function openBatchPrint() {
  const names = [...fileBrowser.selected]
  if (names.length === 0) return
  printFileNames.value = names
  printFileName.value = ''
  printFilePath.value = ''
  printDialogOpen.value = true
}

const selectionMode = ref(false)

function toggleSelectionMode() {
  selectionMode.value = !selectionMode.value
  if (!selectionMode.value) fileBrowser.clearSelection()
}

async function handleDelete(name: string) {
  try {
    await fileBrowser.remove(name)
    message.success(`${name} 已删除`)
  } catch (e) {
    message.error(`删除失败: ${e}`)
  }
}

async function handleBatchDelete() {
  const count = fileBrowser.selected.size
  try {
    await fileBrowser.removeSelected()
    selectionMode.value = false
    message.success(`已删除 ${count} 个文件`)
  } catch (e) {
    message.error(`批量删除失败: ${e}`)
  }
}

async function handleClearAll() {
  const count = fileBrowser.files.length
  try {
    await fileBrowser.removeAll()
    selectionMode.value = false
    message.success(`已清空 ${count} 个文件`)
  } catch (e) {
    message.error(`清空失败: ${e}`)
  }
}

function handleSelectAll() {
  if (fileBrowser.isAllSelected) fileBrowser.clearSelection()
  else fileBrowser.selectAll()
}

let unlistenFileChanged: (() => void) | null = null

onMounted(async () => {
  fileBrowser.refresh()
  const { listen } = await import('@tauri-apps/api/event')
  unlistenFileChanged = await listen('file-changed', () => {
    fileBrowser.refresh()
  })
})

onUnmounted(() => {
  if (unlistenFileChanged) unlistenFileChanged()
})
</script>

<template>
  <BasePage title="文件管理">
    <template #actions>
      <Button size="small" :type="selectionMode ? 'primary' : 'default'" ghost @click="toggleSelectionMode">
        {{ selectionMode ? '取消选择' : '选择' }}
      </Button>
      <Popconfirm title="确定清空所有文件？" ok-text="清空" cancel-text="取消" @confirm="handleClearAll">
        <Button size="small" danger :disabled="fileBrowser.files.length === 0">
          <template #icon><ClearOutlined /></template>
          清空
        </Button>
      </Popconfirm>
      <Button size="small" @click="fileBrowser.refresh()" :loading="fileBrowser.loading">
        <template #icon><ReloadOutlined /></template>
        刷新
      </Button>
    </template>

    <div class="home-grid">
      <div class="home-main">
        <FileUploadZone />

        <div class="file-toolbar">
          <div class="file-toolbar-left">
            <Checkbox
              v-if="selectionMode"
              :checked="fileBrowser.isAllSelected"
              :indeterminate="fileBrowser.hasSelection && !fileBrowser.isAllSelected"
              @change="handleSelectAll"
            >全选</Checkbox>
            <span class="file-count">共 <b>{{ fileBrowser.sortedFiles.length }}</b> 个文件</span>
            <template v-if="selectionMode && fileBrowser.hasSelection">
              <span class="file-count">· 已选 <b>{{ fileBrowser.selected.size }}</b> 个</span>
              <Button size="small" type="primary" @click="openBatchPrint">
                <template #icon><PrinterOutlined /></template>
                批量打印
              </Button>
              <Popconfirm title="确定删除选中的文件？" ok-text="删除" cancel-text="取消" @confirm="handleBatchDelete">
                <Button size="small" type="primary" danger>
                  <template #icon><DeleteOutlined /></template>
                  删除选中
                </Button>
              </Popconfirm>
            </template>
          </div>
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
              :selectable="selectionMode"
              :selected="fileBrowser.selected.has(file.name)"
              @preview="openPreview"
              @delete="handleDelete"
              @print="openPrint"
              @select="fileBrowser.toggleSelect"
            />
          </div>
        </Spin>
        <div v-if="fileBrowser.total > fileBrowser.pageSize" class="file-pagination">
          <Pagination
            :current="fileBrowser.page"
            :page-size="fileBrowser.pageSize"
            :total="fileBrowser.total"
            size="small"
            show-size-changer
            :page-size-options="['20', '50', '100']"
            @change="fileBrowser.changePage"
          />
        </div>
      </div>

      <div class="home-aside">
        <QrCodeCard />
        <SystemStatusCard />
        <PrinterStatusCard />
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
      :file-names="printFileNames"
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


.file-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.file-toolbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
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

.file-pagination {
  display: flex;
  justify-content: flex-end;
  padding-top: 8px;
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

}
</style>
