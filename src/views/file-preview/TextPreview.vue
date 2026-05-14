<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { TypographyParagraph } from 'ant-design-vue'
import { useFileSystem } from '../../composables/useFileSystem'

const props = defineProps<{ fileName: string }>()

const { readFile, base64ToText } = useFileSystem()

const content = ref('')
const loading = ref(false)
const error = ref('')

async function load() {
  loading.value = true
  error.value = ''
  try {
    const base64 = await readFile(props.fileName)
    content.value = base64ToText(base64)
  } catch (e) {
    error.value = `文本加载失败: ${e}`
  } finally {
    loading.value = false
  }
}

watch(() => props.fileName, load)
onMounted(load)
</script>

<template>
  <div class="text-preview">
    <div v-if="loading" class="preview-loading">
      <a-spin tip="加载中..." />
    </div>
    <div v-else-if="error" class="preview-error">{{ error }}</div>
    <TypographyParagraph v-else class="text-content" code>{{ content }}</TypographyParagraph>
  </div>
</template>

<style scoped>
.text-preview {
  height: 100%;
  overflow: auto;
}

.text-content {
  margin: 0;
  padding: 16px;
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
  border-radius: 6px;
}

.text-content :deep(code) {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  background: transparent;
  border: none;
  white-space: pre-wrap;
  word-break: break-all;
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
