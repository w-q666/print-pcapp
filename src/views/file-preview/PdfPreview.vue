<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted, watch } from 'vue'
import * as pdfjsLib from 'pdfjs-dist'
import { useFileSystem } from '../../composables/useFileSystem'

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  'pdfjs-dist/build/pdf.worker.mjs',
  import.meta.url
).href

const props = defineProps<{ fileName: string }>()

const { readFile, base64ToArrayBuffer } = useFileSystem()

const canvasRef = ref<HTMLCanvasElement>()
const currentPage = ref(1)
const totalPages = ref(0)
const scale = ref(1.5)
const loading = ref(false)
const error = ref('')

let pdfDoc: pdfjsLib.PDFDocumentProxy | null = null

async function loadPdf() {
  loading.value = true
  error.value = ''
  try {
    const base64 = await readFile(props.fileName)
    const data = base64ToArrayBuffer(base64)
    pdfDoc = await pdfjsLib.getDocument({ data }).promise
    totalPages.value = pdfDoc.numPages
    currentPage.value = 1
    loading.value = false
    await nextTick()
    await renderPage(1)
  } catch (e) {
    error.value = `PDF 加载失败: ${e}`
    loading.value = false
  }
}

async function renderPage(pageNum: number) {
  if (!pdfDoc || !canvasRef.value) return
  const page = await pdfDoc.getPage(pageNum)
  const viewport = page.getViewport({ scale: scale.value })
  const canvas = canvasRef.value
  canvas.width = viewport.width
  canvas.height = viewport.height
  const ctx = canvas.getContext('2d')!
  await page.render({ canvasContext: ctx, viewport, canvas } as never).promise
}

function prevPage() {
  if (currentPage.value > 1) {
    currentPage.value--
    renderPage(currentPage.value)
  }
}

function nextPage() {
  if (currentPage.value < totalPages.value) {
    currentPage.value++
    renderPage(currentPage.value)
  }
}

function zoomIn() {
  scale.value = Math.min(scale.value + 0.25, 3)
  renderPage(currentPage.value)
}

function zoomOut() {
  scale.value = Math.max(scale.value - 0.25, 0.5)
  renderPage(currentPage.value)
}

watch(() => props.fileName, loadPdf)
onMounted(loadPdf)

onUnmounted(() => {
  pdfDoc?.destroy()
  pdfDoc = null
})
</script>

<template>
  <div class="pdf-preview">
    <div v-if="loading" class="preview-loading">
      <a-spin tip="加载中..." />
    </div>
    <div v-else-if="error" class="preview-error">{{ error }}</div>
    <template v-else>
      <div class="pdf-toolbar">
        <a-button size="small" :disabled="currentPage <= 1" @click="prevPage">上一页</a-button>
        <span class="page-info">{{ currentPage }} / {{ totalPages }}</span>
        <a-button size="small" :disabled="currentPage >= totalPages" @click="nextPage">下一页</a-button>
        <a-button size="small" @click="zoomOut">−</a-button>
        <span class="zoom-info">{{ Math.round(scale * 100) }}%</span>
        <a-button size="small" @click="zoomIn">+</a-button>
      </div>
      <div class="pdf-canvas-wrapper">
        <canvas ref="canvasRef" />
      </div>
    </template>
  </div>
</template>

<style scoped>
.pdf-preview {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.pdf-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  flex-shrink: 0;
}

.page-info, .zoom-info {
  font-size: 13px;
  color: rgba(0, 0, 0, 0.65);
  min-width: 60px;
  text-align: center;
}

.pdf-canvas-wrapper {
  flex: 1;
  overflow: auto;
  text-align: center;
}

.pdf-canvas-wrapper canvas {
  max-width: 100%;
}

.preview-loading, .preview-error {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
}

.preview-error {
  color: #ff4d4f;
}
</style>
