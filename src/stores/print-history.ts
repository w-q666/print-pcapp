import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface PrintJobRecord {
  id: number
  name: string
  status: string
  printer: string
  print_type: string
  source: string
  copies: number
  created_at: string
  finished_at: string | null
  error_msg: string
}

export const usePrintHistory = defineStore('print-history', () => {
  const records = ref<PrintJobRecord[]>([])
  const loading = ref(false)

  const filterStatus = ref<string | null>(null)
  const filterPrinter = ref<string | null>(null)

  async function fetchRecords(limit?: number) {
    loading.value = true
    try {
      const args: Record<string, unknown> = {}
      if (limit !== undefined) args.limit = limit
      records.value = await invoke<PrintJobRecord[]>('print_jobs_list', args)
    } catch (e) {
      console.error('Failed to fetch print jobs:', e)
    } finally {
      loading.value = false
    }
  }

  const filteredRecords = computed(() => {
    return records.value.filter(r => {
      if (filterStatus.value && r.status !== filterStatus.value) return false
      if (filterPrinter.value && r.printer !== filterPrinter.value) return false
      return true
    })
  })

  const uniquePrinters = computed(() => {
    const set = new Set(records.value.map(r => r.printer).filter(Boolean))
    return Array.from(set)
  })

  return {
    records, loading,
    filterStatus, filterPrinter,
    filteredRecords, uniquePrinters,
    fetchRecords,
  }
})
