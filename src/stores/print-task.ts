import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { PrintStatus } from '../api/types'

interface PrintJobEvent {
  job_id: number
  file_name: string
  status: string
  error_msg: string | null
}

function mapBackendStatus(status: string): PrintStatus {
  const map: Record<string, PrintStatus> = {
    queued: 'queued',
    printing: 'printing',
    done: 'done',
    failed: 'failed',
    cancelled: 'cancelled',
  }
  return map[status] ?? 'idle'
}

export const usePrintTask = defineStore('print-task', () => {
  const currentStatus = ref<PrintStatus>('idle')
  const currentJobName = ref('')
  const currentJobId = ref<number | null>(null)
  const statusMessage = ref('')
  const isActive = ref(false)

  let unlisten: UnlistenFn | null = null

  async function startListening() {
    if (unlisten) return
    unlisten = await listen<PrintJobEvent>('print-job-update', (event) => {
      const payload = event.payload
      if (currentJobId.value !== null && payload.job_id !== currentJobId.value) return

      const mapped = mapBackendStatus(payload.status)
      updateStatus(mapped, payload.error_msg ?? undefined)
      if (payload.file_name) currentJobName.value = payload.file_name
    })
  }

  function stopListening() {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  }

  function updateStatus(newStatus: PrintStatus, msg?: string) {
    currentStatus.value = newStatus
    if (msg) statusMessage.value = msg
    isActive.value = !['idle', 'done', 'failed', 'cancelled', 'error'].includes(newStatus)
  }

  function reset() {
    currentStatus.value = 'idle'
    currentJobName.value = ''
    currentJobId.value = null
    statusMessage.value = ''
    isActive.value = false
  }

  startListening()

  return {
    currentStatus, currentJobName, currentJobId, statusMessage, isActive,
    updateStatus, reset, startListening, stopListening,
  }
})
