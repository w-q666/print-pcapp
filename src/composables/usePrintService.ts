import { invoke } from '@tauri-apps/api/core'
import { useLogger } from './useLogger'
import { usePrintTask } from '../stores/print-task'
import { usePrinterList } from '../stores/printer-list'
import { useAppConfig } from '../stores/app-config'
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
  const printTask = usePrintTask()
  const printerList = usePrinterList()
  const appConfig = useAppConfig()
  const logger = useLogger('frontend:usePrintService')

  async function print(params: PrintParams) {
    logger.info('print', `提交打印队列: ${params.fileName} (type: ${params.type})`)
    printTask.updateStatus('preparing')
    printTask.currentJobName = params.fileName

    const printer = params.printer || printerList.defaultPrinter

    try {
      const jobId = await invoke<number>('print_queue_submit', {
        req: {
          fileName: params.fileName,
          printType: params.type,
          printer: printer || '',
          copies: params.copies ?? 1,
          color: params.color ?? false,
          paperSize: params.paperSize ?? 'ISO_A4',
          direction: params.direction ?? 'PORTRAIT',
          serviceUrl: appConfig.serviceUrl,
        },
      })

      printTask.currentJobId = jobId
      logger.info('print', `任务已入队, job_id: ${jobId}`)
      printTask.updateStatus('queued')
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      logger.error('print', `入队失败: ${msg}`)
      printTask.updateStatus('error', msg)
      throw e
    }
  }

  return { print }
}
