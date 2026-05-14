<script setup lang="ts">
import { onMounted, computed, h } from 'vue'
import { Table, Tag, Select, SelectOption, Button, Space } from 'ant-design-vue'
import { ReloadOutlined, DownloadOutlined } from '@ant-design/icons-vue'
import BasePage from '../../components/layout/BasePage.vue'
import { usePrintHistory } from '../../stores/print-history'
import { exportCSV } from '../../utils/export-csv'
import type { TableColumnType as ColumnType } from 'ant-design-vue'

const store = usePrintHistory()

const statusColorMap: Record<string, string> = {
  done: 'green',
  failed: 'red',
  printing: 'blue',
  queued: 'orange',
  cancelled: 'default',
}

const statusLabelMap: Record<string, string> = {
  done: '已完成',
  failed: '失败',
  printing: '打印中',
  queued: '排队中',
  cancelled: '已取消',
}

const sourceLabelMap: Record<string, string> = {
  desktop: '桌面端',
  mobile: '移动端',
  api: 'API',
}

const columns = computed<ColumnType[]>(() => [
  { title: '文件名', dataIndex: 'name', key: 'name', width: 200, ellipsis: true },
  {
    title: '状态', dataIndex: 'status', key: 'status', width: 100,
    customRender: ({ text }: { text: string }) =>
      h(Tag, { color: statusColorMap[text] || 'default' }, () => statusLabelMap[text] || text),
  },
  { title: '打印机', dataIndex: 'printer', key: 'printer', width: 150, ellipsis: true },
  { title: '类型', dataIndex: 'print_type', key: 'print_type', width: 80 },
  {
    title: '来源', dataIndex: 'source', key: 'source', width: 80,
    customRender: ({ text }: { text: string }) =>
      h(Tag, { color: text === 'mobile' ? 'cyan' : 'geekblue' }, () => sourceLabelMap[text] || text),
  },
  { title: '份数', dataIndex: 'copies', key: 'copies', width: 60, align: 'center' as const },
  { title: '创建时间', dataIndex: 'created_at', key: 'created_at', width: 160 },
  { title: '完成时间', dataIndex: 'finished_at', key: 'finished_at', width: 160,
    customRender: ({ text }: { text: string | null }) => text || '-',
  },
])

function handleStatusChange(val: unknown) {
  store.filterStatus = (val as string) || null
}

function handlePrinterChange(val: unknown) {
  store.filterPrinter = (val as string) || null
}

function handleExportCSV() {
  const headers = ['文件名', '状态', '打印机', '类型', '来源', '份数', '创建时间', '完成时间']
  const rows = store.filteredRecords.map(r => [
    r.name,
    statusLabelMap[r.status] || r.status,
    r.printer,
    r.print_type,
    sourceLabelMap[r.source] || r.source,
    String(r.copies),
    r.created_at,
    r.finished_at || '',
  ])
  const now = new Date().toISOString().slice(0, 10)
  exportCSV(headers, rows, `print-history-${now}.csv`)
}

onMounted(() => {
  store.fetchRecords()
})
</script>

<template>
  <BasePage title="打印历史">
    <template #actions>
      <Space>
        <Select
          :value="store.filterStatus || ''"
          style="width: 130px"
          placeholder="状态筛选"
          @change="handleStatusChange"
        >
          <SelectOption value="">全部状态</SelectOption>
          <SelectOption value="done">已完成</SelectOption>
          <SelectOption value="printing">打印中</SelectOption>
          <SelectOption value="queued">排队中</SelectOption>
          <SelectOption value="failed">失败</SelectOption>
          <SelectOption value="cancelled">已取消</SelectOption>
        </Select>

        <Select
          :value="store.filterPrinter || ''"
          style="width: 160px"
          placeholder="打印机筛选"
          @change="handlePrinterChange"
        >
          <SelectOption value="">全部打印机</SelectOption>
          <SelectOption
            v-for="p in store.uniquePrinters"
            :key="p"
            :value="p"
          >
            {{ p }}
          </SelectOption>
        </Select>

        <Button @click="handleExportCSV">
          <template #icon><DownloadOutlined /></template>
          导出
        </Button>

        <Button @click="store.fetchRecords()" :loading="store.loading">
          <template #icon><ReloadOutlined /></template>
          刷新
        </Button>
      </Space>
    </template>

    <div class="history-content">
      <Table
        :columns="columns"
        :data-source="store.filteredRecords"
        :loading="store.loading"
        row-key="id"
        size="small"
        :scroll="{ x: 900 }"
        :pagination="{ pageSize: 20, showSizeChanger: true, showTotal: (total: number) => `共 ${total} 条` }"
      />
    </div>
  </BasePage>
</template>

