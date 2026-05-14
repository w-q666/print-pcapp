export type PrintableType = 'PDF' | 'IMG' | 'TEXT' | 'HTML'

const typeMap: Record<string, PrintableType> = {
  '.pdf': 'PDF',
  '.jpg': 'IMG', '.jpeg': 'IMG', '.png': 'IMG', '.bmp': 'IMG', '.gif': 'IMG',
  '.tiff': 'IMG', '.webp': 'IMG',
  '.txt': 'TEXT',
  '.html': 'HTML', '.htm': 'HTML',
}

export function getFileType(fileName: string): PrintableType | null {
  const ext = fileName.substring(fileName.lastIndexOf('.')).toLowerCase()
  return typeMap[ext] ?? null
}

export function isSupported(fileName: string): boolean {
  return getFileType(fileName) !== null
}

export function getFileExtension(fileName: string): string {
  return fileName.substring(fileName.lastIndexOf('.')).toLowerCase()
}

export function getPreviewComponent(type: PrintableType): string {
  switch (type) {
    case 'PDF': return 'PdfPreview'
    case 'IMG': return 'ImagePreview'
    case 'TEXT': return 'TextPreview'
    case 'HTML': return 'HtmlPreview'
  }
}

export const fileCategories = {
  pdf: ['.pdf'],
  image: ['.jpg', '.jpeg', '.png', '.gif', '.bmp', '.tiff', '.webp'],
  text: ['.txt', '.htm', '.html'],
} as const

export function getMimeType(fileName: string): string {
  const ext = getFileExtension(fileName)
  const mimeMap: Record<string, string> = {
    '.pdf': 'application/pdf',
    '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.png': 'image/png',
    '.bmp': 'image/bmp', '.gif': 'image/gif', '.tiff': 'image/tiff',
    '.webp': 'image/webp', '.svg': 'image/svg+xml',
    '.txt': 'text/plain', '.log': 'text/plain', '.csv': 'text/csv',
    '.json': 'application/json', '.xml': 'application/xml',
    '.html': 'text/html', '.htm': 'text/html',
  }
  return mimeMap[ext] ?? 'application/octet-stream'
}
