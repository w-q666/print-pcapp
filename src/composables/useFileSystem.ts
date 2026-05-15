import { invoke } from '@tauri-apps/api/core'
import { useLogger } from './useLogger'

const logger = useLogger('frontend:useFileSystem')

export interface FileInfo {
  name: string
  size: number
  modified_at: number  // Unix timestamp in seconds
}

export function useFileSystem() {
  async function saveFile(name: string, bytes: Uint8Array): Promise<string> {
    try {
      const result = await invoke<string>('file_save', { name, bytes: Array.from(bytes) })
      logger.info('file', `file saved: ${name} (${(bytes.length / 1024).toFixed(2)} KB)`)
      return result
    } catch (err) {
      logger.error('file', `file save failed: ${name} - ${err}`)
      throw err
    }
  }

  async function readFile(name: string): Promise<string> {
    try {
      const result = await invoke<string>('file_read', { name })
      logger.debug('file', `file read: ${name}`)
      return result
    } catch (err) {
      logger.error('file', `file read failed: ${name} - ${err}`)
      throw err
    }
  }

  async function deleteFile(name: string): Promise<void> {
    try {
      await invoke<void>('file_delete', { name })
      logger.info('file', `file deleted: ${name}`)
    } catch (err) {
      logger.error('file', `file delete failed: ${name} - ${err}`)
      throw err
    }
  }

  interface FileListPage {
    files: FileInfo[]
    total: number
  }

  async function listFiles(page?: number, pageSize?: number): Promise<FileListPage> {
    try {
      const result = await invoke<FileListPage>('file_list', { page, pageSize })
      logger.debug('file', `file list: ${result.files.length}/${result.total} files (page ${page ?? 1})`)
      return result
    } catch (err) {
      logger.error('file', `file list failed: ${err}`)
      throw err
    }
  }

  function base64ToBlobUrl(base64: string, mimeType: string): string {
    const binary = atob(base64)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
    const blob = new Blob([bytes], { type: mimeType })
    return URL.createObjectURL(blob)
  }

  function base64ToArrayBuffer(base64: string): ArrayBuffer {
    const binary = atob(base64)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
    return bytes.buffer
  }

  function base64ToText(base64: string): string {
    const binary = atob(base64)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
    return new TextDecoder('utf-8').decode(bytes)
  }

  return { saveFile, readFile, deleteFile, listFiles, base64ToBlobUrl, base64ToArrayBuffer, base64ToText }
}
