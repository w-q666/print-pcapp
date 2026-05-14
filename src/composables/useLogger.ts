import { invoke } from '@tauri-apps/api/core'

let debugEnabled: boolean | null = null

async function loadDebugFlag(): Promise<boolean> {
  if (debugEnabled !== null) return debugEnabled
  try {
    const { Store } = await import('@tauri-apps/plugin-store')
    const store = await Store.load('settings.json')
    const level = await store.get<string>('logLevel')
    debugEnabled = level === 'DEBUG'
  } catch {
    debugEnabled = false
  }
  return debugEnabled
}

export function setDebugEnabled(enabled: boolean) {
  debugEnabled = enabled
}

function writeLog(level: string, category: string, message: string, logger: string) {
  invoke('log_insert', { level, category, message, logger }).catch(() => {})
}

export function useLogger(source: string) {
  return {
    debug(category: string, message: string) {
      loadDebugFlag().then((enabled) => {
        if (enabled) writeLog('DEBUG', category, message, source)
      })
    },
    info(category: string, message: string) {
      writeLog('INFO', category, message, source)
    },
    warn(category: string, message: string) {
      writeLog('WARN', category, message, source)
    },
    error(category: string, message: string) {
      writeLog('ERROR', category, message, source)
    },
  }
}
