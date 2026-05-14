import { ref, onUnmounted } from 'vue'
import type { PrintStatus } from '../api/types'
import { PrintStatusCode } from '../api/types'
import { createWebSocket } from '../api/websocket-client'
import { useLogger } from './useLogger'

const logger = useLogger('frontend:useWebSocket')

export function useWebSocket() {
  const ws = ref<WebSocket | null>(null)
  const sessionId = ref<string | null>(null)
  const status = ref<PrintStatus>('idle')
  const isConnected = ref(false)
  const lastMessage = ref<{ code: number; msg: string } | null>(null)

  let retryCount = 0
  let retryTimer: ReturnType<typeof setTimeout> | null = null
  let currentUrl = ''

  function connect(url: string) {
    cleanup()
    currentUrl = url
    status.value = 'connecting'

    const socket = createWebSocket(url, {
      onOpen() {
        isConnected.value = true
        retryCount = 0
        logger.info('print', `WebSocket connected: ${url}`)
      },
      onMessage(data) {
        const msg = data as { code: number; msg: string; data?: string }
        if (msg.code === 0 && msg.data) {
          sessionId.value = msg.data
          status.value = 'connected'
          logger.debug('print', `WebSocket session established: ${msg.data}`)
          return
        }
        lastMessage.value = { code: msg.code, msg: msg.msg }
        status.value = mapStatusCode(msg.code)
        logger.debug('print', `WebSocket message: code=${msg.code}, msg=${msg.msg}`)
      },
      onClose() {
        isConnected.value = false
        logger.warn('print', `WebSocket disconnected: ${url}`)
        if (retryCount !== Infinity) {
          scheduleReconnect()
        }
      },
      onError() {
        isConnected.value = false
        logger.error('print', `WebSocket error: ${url}`)
      },
    })

    ws.value = socket
  }

  function scheduleReconnect() {
    if (retryTimer) return
    const delay = retryCount === 0 ? 0
      : retryCount <= 4 ? Math.pow(2, retryCount - 1) * 1000
      : 30000
    retryCount++
    logger.debug('print', `WebSocket reconnect #${retryCount} in ${delay}ms`)
    retryTimer = setTimeout(() => {
      retryTimer = null
      connect(currentUrl)
    }, delay)
  }

  function mapStatusCode(code: number): PrintStatus {
    switch (code) {
      case PrintStatusCode.PREPARING: return 'preparing'
      case PrintStatusCode.PRINTING: return 'printing'
      case PrintStatusCode.COMPLETED: return 'done'
      case PrintStatusCode.ERROR: return 'error'
      case PrintStatusCode.DATA_TRANSFERRED: return 'data_sent'
      case PrintStatusCode.NEEDS_ATTENTION: return 'needs_attention'
      case PrintStatusCode.FAILED: return 'failed'
      case PrintStatusCode.CANCELLED: return 'cancelled'
      case PrintStatusCode.FILE_NOT_FOUND: return 'error'
      case PrintStatusCode.FILE_ERROR: return 'error'
      default: return 'idle'
    }
  }

  function cleanup() {
    if (retryTimer) { clearTimeout(retryTimer); retryTimer = null }
    if (ws.value) {
      ws.value.onopen = null
      ws.value.onmessage = null
      ws.value.onclose = null
      ws.value.onerror = null
      ws.value.close()
      ws.value = null
    }
  }

  function disconnect() {
    retryCount = Infinity
    cleanup()
    sessionId.value = null
    status.value = 'idle'
    isConnected.value = false
    logger.info('print', 'WebSocket manually disconnected')
  }

  onUnmounted(disconnect)

  return { connect, disconnect, sessionId, status, isConnected, lastMessage }
}
