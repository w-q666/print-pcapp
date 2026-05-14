import { watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useWebSocket } from './useWebSocket'
import { useLogger } from './useLogger'
import { usePrintTask } from '../stores/print-task'
import { usePrinterList } from '../stores/printer-list'
import { useAppConfig } from '../stores/app-config'
import { printSingle } from '../api/print-api'
import type { PrintRequest } from '../api/types'

export interface PrintParams {
  fileName: string
  filePath: string
  type: PrintRequest['type']
  source: PrintRequest['source']
  content?: string
  printer?: string
  copies?: number
  color?: boolean
  paperSize?: string
  direction?: PrintRequest['direction']
}

export function usePrintService() {
  const wsClient = useWebSocket()
  const printTask = usePrintTask()
  const printerList = usePrinterList()
  const appConfig = useAppConfig()
  const logger = useLogger('frontend:usePrintService')

  watch(() => wsClient.status.value, (newStatus) => {
    if (newStatus !== 'idle' && newStatus !== 'connecting' && newStatus !== 'connected') {
      const msg = wsClient.lastMessage.value?.msg
      printTask.updateStatus(newStatus, msg)
    }
  })

  async function ensureConnected(): Promise<string> {
    if (wsClient.isConnected.value && wsClient.sessionId.value) {
      return wsClient.sessionId.value
    }

    logger.debug('print', `WebSocket connecting: ${appConfig.wsUrl}`)
    wsClient.connect(appConfig.wsUrl)

    return new Promise<string>((resolve, reject) => {
      const timeout = setTimeout(() => {
        unwatch()
        logger.error('print', `WebSocket connect timeout (10s): ${appConfig.wsUrl}`)
        reject(new Error('WebSocket connect timeout'))
      }, 10000)

      const unwatch = watch(() => wsClient.sessionId.value, (id) => {
        if (id) {
          clearTimeout(timeout)
          unwatch()
          logger.debug('print', `WebSocket connected, sessionId: ${id}`)
          resolve(id)
        }
      }, { immediate: true })
    })
  }

  async function print(params: PrintParams) {
    logger.info('print', `print start: ${params.fileName} (type: ${params.type}, source: ${params.source})`)
    printTask.updateStatus('connecting')
    printTask.currentJobName = params.fileName

    const sessionId = await ensureConnected()
    printTask.updateStatus('preparing')

    const req: PrintRequest = {
      type: params.type,
      source: params.source,
      sessionId,
      copies: params.copies ?? 1,
      color: params.color ?? false,
      paperSize: params.paperSize,
      direction: params.direction,
      printServer: params.printer || printerList.defaultPrinter,
    }

    if (params.source === 'blob') {
      const base64: string = await invoke('file_read', { name: params.fileName })
      const binary = atob(base64)
      const bytes = new Uint8Array(binary.length)
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
      req.file = new Blob([bytes])
    } else if (params.source === 'path') {
      req.content = params.filePath
    } else {
      req.content = params.content
    }

    await invoke('print_jobs_create', {
      name: params.fileName,
      printer: req.printServer || '',
      printType: params.type,
      source: 'desktop',
      copies: req.copies,
      filePath: params.filePath || '',
      fileSize: 0,
    })
    logger.debug('print', `print job record created: ${params.fileName}`)

    const result = await printSingle(req)
    if (result.code !== 0) {
      logger.error('print', `print failed: ${params.fileName} - ${result.msg}`)
      printTask.updateStatus('error', result.msg)
      throw new Error(result.msg)
    }
    logger.info('print', `print submitted: ${params.fileName}`)
  }

  return { print, ensureConnected }
}
