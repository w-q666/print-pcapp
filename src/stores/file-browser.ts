import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useFileSystem, type FileInfo } from '../composables/useFileSystem'

export interface FileItem {
  name: string
  extension: string
  size: number
  modified_at: number
  dateLabel: string
}

function toFileItem(info: FileInfo): FileItem {
  return {
    name: info.name,
    extension: info.name.substring(info.name.lastIndexOf('.')).toLowerCase(),
    size: info.size,
    modified_at: info.modified_at,
    dateLabel: info.modified_at > 0
      ? new Date(info.modified_at * 1000).toLocaleString('zh-CN', {
          year: 'numeric', month: '2-digit', day: '2-digit',
          hour: '2-digit', minute: '2-digit',
        })
      : '',
  }
}

export const useFileBrowser = defineStore('file-browser', () => {
  const files = ref<FileItem[]>([])
  const loading = ref(false)
  const sortBy = ref<'name' | 'extension'>('name')
  const selected = ref<Set<string>>(new Set())

  const page = ref(1)
  const pageSize = ref(50)
  const total = ref(0)

  const { listFiles, deleteFile } = useFileSystem()

  async function refresh() {
    loading.value = true
    try {
      const result = await listFiles(page.value, pageSize.value)
      files.value = result.files.map(toFileItem)
      total.value = result.total
      selected.value = new Set()
    } finally {
      loading.value = false
    }
  }

  function changePage(newPage: number, newPageSize?: number) {
    page.value = newPage
    if (newPageSize && newPageSize !== pageSize.value) {
      pageSize.value = newPageSize
      page.value = 1
    }
    refresh()
  }

  async function remove(name: string) {
    await deleteFile(name)
    await refresh()
  }

  async function removeSelected() {
    const names = [...selected.value]
    for (const name of names) {
      await deleteFile(name)
    }
    await refresh()
  }

  async function removeAll() {
    for (const file of files.value) {
      await deleteFile(file.name)
    }
    await refresh()
  }

  function toggleSelect(name: string) {
    const s = new Set(selected.value)
    if (s.has(name)) s.delete(name)
    else s.add(name)
    selected.value = s
  }

  function selectAll() {
    selected.value = new Set(files.value.map(f => f.name))
  }

  function clearSelection() {
    selected.value = new Set()
  }

  const isAllSelected = computed(() =>
    files.value.length > 0 && selected.value.size === files.value.length
  )

  const hasSelection = computed(() => selected.value.size > 0)

  const sortedFiles = computed(() => {
    return [...files.value].sort((a, b) => {
      if (sortBy.value === 'extension') return a.extension.localeCompare(b.extension)
      return a.name.localeCompare(b.name, 'zh')
    })
  })

  return {
    files, loading, sortBy, selected, sortedFiles,
    page, pageSize, total,
    isAllSelected, hasSelection,
    refresh, changePage, remove, removeSelected, removeAll,
    toggleSelect, selectAll, clearSelection,
  }
})
