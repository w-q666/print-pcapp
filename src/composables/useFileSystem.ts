import { invoke } from '@tauri-apps/api/core'

export interface FileInfo {
  name: string
  size: number
  modified_at: number  // Unix timestamp in seconds
}

export function useFileSystem() {
  async function saveFile(name: string, bytes: Uint8Array): Promise<string> {
    return invoke<string>('file_save', { name, bytes: Array.from(bytes) })
  }

  async function readFile(name: string): Promise<string> {
    return invoke<string>('file_read', { name })
  }

  async function deleteFile(name: string): Promise<void> {
    return invoke<void>('file_delete', { name })
  }

  async function listFiles(): Promise<FileInfo[]> {
    return invoke<FileInfo[]>('file_list')
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
