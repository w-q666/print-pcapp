import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface LogEntry {
  id: number
  timestamp: string
  level: string
  category: string
  message: string
  logger: string
}

export const useSystemLog = defineStore('system-log', () => {
  const logs = ref<LogEntry[]>([])
  const loading = ref(false)

  const filterLevel = ref<string | null>(null)
  const filterCategory = ref<string | null>(null)
  const filterKeyword = ref('')
  const displayLimit = ref(100)

  async function fetchLogs() {
    loading.value = true
    try {
      logs.value = await invoke<LogEntry[]>('log_query', {
        level: filterLevel.value || null,
        category: filterCategory.value || null,
        keyword: filterKeyword.value || null,
        limit: displayLimit.value,
      })
    } catch (e) {
      console.error('Failed to fetch logs:', e)
    } finally {
      loading.value = false
    }
  }

  async function clearLogs() {
    try {
      await invoke('log_clear')
      await fetchLogs()
    } catch (e) {
      console.error('Failed to clear logs:', e)
    }
  }

  function setCategory(cat: string | null) {
    filterCategory.value = cat
    fetchLogs()
  }

  return {
    logs, loading,
    filterLevel, filterCategory, filterKeyword, displayLimit,
    fetchLogs, clearLogs, setCategory,
  }
})
