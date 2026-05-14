<script setup lang="ts">
import { Checkbox, Row, Col, Typography } from 'ant-design-vue'
import { useSettings, type AllowedExtensions } from '../../stores/settings'

const settings = useSettings()

interface ExtInfo {
  ext: string
  desc: string
}

const groups: { key: keyof AllowedExtensions; title: string; items: ExtInfo[] }[] = [
  {
    key: 'document',
    title: '文档格式',
    items: [
      { ext: '.htm', desc: 'HTML 网页' },
      { ext: '.html', desc: 'HTML 网页' },
      { ext: '.txt', desc: '纯文本' },
      { ext: '.doc', desc: 'Word 文档' },
      { ext: '.rtf', desc: '富文本' },
      { ext: '.pdf', desc: 'PDF 文档' },
      { ext: '.docx', desc: 'Word 文档' },
      { ext: '.xml', desc: 'XML 文档' },
      { ext: '.odt', desc: 'OpenDocument' },
    ],
  },
  {
    key: 'spreadsheet',
    title: '表格格式',
    items: [
      { ext: '.ods', desc: 'OpenDocument 表格' },
      { ext: '.csv', desc: 'CSV 表格' },
      { ext: '.xlsx', desc: 'Excel 表格' },
      { ext: '.xls', desc: 'Excel 表格' },
    ],
  },
  {
    key: 'presentation',
    title: '演示文稿格式',
    items: [
      { ext: '.pptx', desc: 'PowerPoint' },
      { ext: '.odp', desc: 'OpenDocument 演示' },
      { ext: '.ppt', desc: 'PowerPoint' },
    ],
  },
  {
    key: 'image',
    title: '图片格式',
    items: [
      { ext: '.tiff', desc: 'TIFF 图像' },
      { ext: '.png', desc: 'PNG 图像' },
      { ext: '.webp', desc: 'WebP 图像' },
      { ext: '.gif', desc: 'GIF 图像' },
      { ext: '.jpeg', desc: 'JPEG 图像' },
      { ext: '.svg', desc: 'SVG 矢量图' },
      { ext: '.jpg', desc: 'JPEG 图像' },
      { ext: '.bmp', desc: 'BMP 图像' },
    ],
  },
]

function toggle(groupKey: keyof AllowedExtensions, ext: string) {
  settings.allowedExtensions[groupKey][ext] = !settings.allowedExtensions[groupKey][ext]
}
</script>

<template>
  <div class="file-format-tab">
    <div v-for="group in groups" :key="group.key" class="format-group">
      <Typography.Title :level="5" style="margin-bottom: 12px">{{ group.title }}</Typography.Title>
      <Row :gutter="[16, 8]">
        <Col v-for="item in group.items" :key="item.ext" :xs="12" :sm="8" :md="6">
          <Checkbox
            :checked="settings.allowedExtensions[group.key][item.ext]"
            @change="toggle(group.key, item.ext)"
          >
            <span class="ext-name">{{ item.ext }}</span>
            <span class="ext-desc">{{ item.desc }}</span>
          </Checkbox>
        </Col>
      </Row>
    </div>
  </div>
</template>

<style scoped>
.file-format-tab {
  padding: 4px 0;
}
.format-group {
  margin-bottom: 24px;
}
.ext-name {
  color: var(--ant-color-pink, #eb2f96);
  font-weight: 600;
  margin-right: 6px;
}
.ext-desc {
  color: var(--ant-color-text-secondary, #666);
  font-size: 13px;
}
</style>
