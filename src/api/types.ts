export interface CommonResult<T = unknown> {
  code: number
  msg: string
  data?: T
}

export interface PrintRequest {
  type: 'PDF' | 'IMG' | 'TEXT' | 'HTML'
  source: 'text' | 'path' | 'url' | 'blob'
  content?: string
  file?: File | Blob
  sessionId?: string
  copies?: number
  color?: boolean
  paperSize?: string
  direction?: 'PORTRAIT' | 'LANDSCAPE' | 'REVERSE_LANDSCAPE' | 'REVERSE_PORTRAIT'
  printServer?: string
}

export const PrintStatusCode = {
  PREPARING: 200000,
  PRINTING: 200001,
  COMPLETED: 200002,
  ERROR: 200003,
  DATA_TRANSFERRED: 200004,
  NEEDS_ATTENTION: 200005,
  FAILED: 200006,
  CANCELLED: 200007,
  FILE_NOT_FOUND: 200008,
  FILE_ERROR: 200009,
} as const

export type PrintStatus =
  | 'idle'
  | 'queued'
  | 'connecting'
  | 'connected'
  | 'preparing'
  | 'printing'
  | 'data_sent'
  | 'done'
  | 'error'
  | 'needs_attention'
  | 'failed'
  | 'cancelled'

export const PaperSizes = [
  'ISO_A3', 'ISO_A4', 'ISO_A5', 'ISO_A6',
  'ISO_A0', 'ISO_A1', 'ISO_A2',
  'ISO_A7', 'ISO_A8', 'ISO_A9', 'ISO_A10',
  'EXECUTIVE', 'FOLIO', 'INVOICE',
] as const
