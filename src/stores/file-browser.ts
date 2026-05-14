import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useFileSystem, type FileInfo } from '../composables/useFileSystem'

export interface FileItem {
  name: string
  extension: string
  size: number
  modified_at: number  // Unix timestamp seconds
  dateLabel: string     // Formatted for display
}

export const useFileBrowser = defineStore('file-browser', () => {
  const files = ref<FileItem[]>([])
  const loading = ref(false)
  const sortBy = ref<'name' | 'extension'>('name')

  const { listFiles, deleteFile } = useFileSystem()

  async function refresh() {
    loading.value = true
    try {
      const infos: FileInfo[] = await listFiles()
      files.value = infos.map(info => ({
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
      }))
    } finally {
      loading.value = false
    }
  }

  async function remove(name: string) {
    await deleteFile(name)
    await refresh()
  }

  const sortedFiles = computed(() => {
    return [...files.value].sort((a, b) => {
      if (sortBy.value === 'extension') return a.extension.localeCompare(b.extension)
      return a.name.localeCompare(b.name, 'zh')
    })
  })

  return { files, loading, sortBy, sortedFiles, refresh, remove }
})
