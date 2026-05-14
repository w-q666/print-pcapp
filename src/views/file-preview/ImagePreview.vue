<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { Image } from 'ant-design-vue'
import { useFileSystem } from '../../composables/useFileSystem'
import { getMimeType } from '../../utils/file-types'

const props = defineProps<{ fileName: string }>()

const { readFile, base64ToBlobUrl } = useFileSystem()

const imgSrc = ref('')
const scale = ref(1)
const loading = ref(false)
const error = ref('')

async function load() {
  loading.value = true
  error.value = ''
  try {
    const base64 = await readFile(props.fileName)
    cleanup()
    const mime = getMimeType(props.fileName)
    imgSrc.value = base64ToBlobUrl(base64, mime)
  } catch (e) {
    error.value = `图片加载失败: ${e}`
  } finally {
    loading.value = false
  }
}

function zoomIn() { scale.value = Math.min(scale.value + 0.25, 5) }
function zoomOut() { scale.value = Math.max(scale.value - 0.25, 0.25) }
function resetZoom() { scale.value = 1 }

function cleanup() {
  if (imgSrc.value) {
    URL.revokeObjectURL(imgSrc.value)
    imgSrc.value = ''
  }
}

watch(() => props.fileName, load)
onMounted(load)
onUnmounted(cleanup)
</script>

<template>
  <div class="image-preview">
    <div v-if="loading" class="preview-loading">
      <a-spin tip="加载中..." />
    </div>
    <div v-else-if="error" class="preview-error">{{ error }}</div>
    <template v-else>
      <div class="image-toolbar">
        <a-button size="small" @click="zoomOut">−</a-button>
        <span class="zoom-info">{{ Math.round(scale * 100) }}%</span>
        <a-button size="small" @click="zoomIn">+</a-button>
        <a-button size="small" @click="resetZoom">重置</a-button>
      </div>
      <div class="image-wrapper">
        <Image
          :src="imgSrc"
          :preview="false"
          :style="{ transform: `scale(${scale})`, transformOrigin: 'top center' }"
          class="preview-image"
        />
      </div>
    </template>
  </div>
</template>

<style scoped>
.image-preview {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.image-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  flex-shrink: 0;
}

.zoom-info {
  font-size: 13px;
  color: rgba(0, 0, 0, 0.65);
  min-width: 60px;
  text-align: center;
}

.image-wrapper {
  flex: 1;
  overflow: auto;
  text-align: center;
}

.image-wrapper :deep(.ant-image) {
  max-width: 100%;
  transition: transform 0.2s;
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
