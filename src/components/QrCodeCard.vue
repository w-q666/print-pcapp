<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Card, Button, Image, message } from 'ant-design-vue'
import { CopyOutlined, QrcodeOutlined } from '@ant-design/icons-vue'
import { invoke } from '@tauri-apps/api/core'

const qrBase64 = ref('')
const lanUrl = ref('')
const loading = ref(true)

async function loadData() {
  loading.value = true
  try {
    const [qr, url] = await Promise.all([
      invoke<string>('lan_server_qrcode'),
      invoke<string>('lan_server_url'),
    ])
    qrBase64.value = qr
    lanUrl.value = url
  } catch (e) {
    console.warn('Failed to load QR code:', e)
  } finally {
    loading.value = false
  }
}

async function copyUrl() {
  try {
    await navigator.clipboard.writeText(lanUrl.value)
    message.success('链接已复制')
  } catch {
    message.error('复制失败')
  }
}

onMounted(loadData)
</script>

<template>
  <Card :loading="loading" size="small">
    <template #title>
      <QrcodeOutlined style="margin-right: 8px" />手机投印
    </template>
    <div v-if="!loading" class="qr-card-body">
      <Image
        v-if="qrBase64"
        :src="`data:image/png;base64,${qrBase64}`"
        alt="QR Code"
        :preview="false"
        class="qr-image"
      />
      <div v-if="lanUrl" class="qr-url-row">
        <span class="qr-url">{{ lanUrl }}</span>
        <Button type="link" size="small" @click="copyUrl">
          <template #icon><CopyOutlined /></template>
          复制
        </Button>
      </div>
    </div>
  </Card>
</template>

<style scoped>
.qr-card-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.qr-image {
  width: 160px;
  height: 160px;
  border-radius: 4px;
}

.qr-url-row {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
}

.qr-url {
  font-size: 12px;
  color: rgba(0, 0, 0, 0.45);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}
</style>
