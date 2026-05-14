import type { CommonResult } from './types'
import { useLogger } from '../composables/useLogger'

const logger = useLogger('frontend:http-client::request')

let baseURL = 'http://localhost:2024'

export function setBaseURL(url: string) {
  baseURL = url
}

export function getBaseURL(): string {
  return baseURL
}

export async function request<T>(
  path: string,
  options: RequestInit = {}
): Promise<CommonResult<T>> {
  const method = options.method || 'GET'
  const start = Date.now()
  const url = `${baseURL}${path}`

  try {
    const response = await fetch(url, {
      ...options,
      headers: {
        ...options.headers,
      },
    })

    const elapsed = Date.now() - start

    if (!response.ok) {
      const errMsg = `${method} ${path} → ${response.status} ${response.statusText} (${elapsed}ms)`
      logger.error('http', errMsg)
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }

    logger.debug('http', `${method} ${path} → ${response.status} (${elapsed}ms)`)
    return response.json()
  } catch (err) {
    const elapsed = Date.now() - start
    if (err instanceof Error && !err.message.startsWith('HTTP ')) {
      logger.error('http', `${method} ${path} → ${err.message} (${elapsed}ms)`)
    }
    throw err
  }
}

export async function get<T>(path: string): Promise<CommonResult<T>> {
  return request<T>(path, { method: 'GET' })
}

export async function postFormData<T>(
  path: string,
  data: Record<string, string | number | boolean | Blob | File | undefined>
): Promise<CommonResult<T>> {
  const formData = new FormData()
  for (const [key, value] of Object.entries(data)) {
    if (value === undefined) continue
    if (typeof value === 'object') {
      formData.append(key, value)
    } else {
      formData.append(key, String(value))
    }
  }
  return request<T>(path, { method: 'POST', body: formData })
}
