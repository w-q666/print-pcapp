import { defineStore } from 'pinia'
import { ref, reactive } from 'vue'

export interface ExtensionMap {
  [ext: string]: boolean
}

export interface AllowedExtensions {
  document: ExtensionMap
  spreadsheet: ExtensionMap
  presentation: ExtensionMap
  image: ExtensionMap
}

export const useSettings = defineStore('settings', () => {
  const allowedExtensions: AllowedExtensions = reactive({
    document: {
      '.htm': true, '.html': true, '.txt': true, '.doc': true,
      '.rtf': true, '.pdf': true, '.docx': true, '.xml': true, '.odt': true,
    },
    spreadsheet: {
      '.ods': true, '.csv': true, '.xlsx': true, '.xls': true,
    },
    presentation: {
      '.pptx': true, '.odp': true, '.ppt': true,
    },
    image: {
      '.tiff': true, '.png': true, '.webp': true, '.gif': true,
      '.jpeg': true, '.svg': true, '.jpg': true, '.bmp': true,
    },
  })

  const defaultPrinter = ref('')
  const defaultPaperSize = ref('ISO_A4')
  const defaultCopies = ref(1)
  const defaultColor = ref(false)
  const defaultDirection = ref<'PORTRAIT' | 'LANDSCAPE'>('PORTRAIT')

  const lanPort = ref(5000)
  const logLevel = ref<'DEBUG' | 'INFO' | 'WARN' | 'ERROR'>('INFO')
  const autoStart = ref(false)

  function getAllowedExtList(): string[] {
    const result: string[] = []
    for (const category of Object.values(allowedExtensions)) {
      for (const [ext, enabled] of Object.entries(category)) {
        if (enabled) result.push(ext)
      }
    }
    return result
  }

  async function loadFromStore() {
    try {
      const { Store } = await import('@tauri-apps/plugin-store')
      const store = await Store.load('settings.json')

      const ext = await store.get<AllowedExtensions>('allowedExtensions')
      if (ext) {
        for (const cat of Object.keys(allowedExtensions) as (keyof AllowedExtensions)[]) {
          if (ext[cat]) {
            for (const [k, v] of Object.entries(ext[cat])) {
              allowedExtensions[cat][k] = v
            }
          }
        }
      }

      const p = await store.get<string>('defaultPrinter')
      if (p !== null && p !== undefined) defaultPrinter.value = p
      const ps = await store.get<string>('defaultPaperSize')
      if (ps) defaultPaperSize.value = ps
      const c = await store.get<number>('defaultCopies')
      if (c !== null && c !== undefined) defaultCopies.value = c
      const dc = await store.get<boolean>('defaultColor')
      if (dc !== null && dc !== undefined) defaultColor.value = dc
      const dd = await store.get<'PORTRAIT' | 'LANDSCAPE'>('defaultDirection')
      if (dd) defaultDirection.value = dd

      const lp = await store.get<number>('lanPort')
      if (lp !== null && lp !== undefined) lanPort.value = lp
      const ll = await store.get<'DEBUG' | 'INFO' | 'WARN' | 'ERROR'>('logLevel')
      if (ll) logLevel.value = ll
      const as_ = await store.get<boolean>('autoStart')
      if (as_ !== null && as_ !== undefined) autoStart.value = as_
    } catch (e) {
      console.warn('Failed to load settings from store:', e)
    }
  }

  async function saveToStore() {
    try {
      const { Store } = await import('@tauri-apps/plugin-store')
      const store = await Store.load('settings.json')

      const extSnapshot: Record<string, Record<string, boolean>> = {}
      for (const cat of Object.keys(allowedExtensions) as (keyof AllowedExtensions)[]) {
        extSnapshot[cat] = { ...allowedExtensions[cat] }
      }
      await store.set('allowedExtensions', extSnapshot)

      await store.set('defaultPrinter', defaultPrinter.value)
      await store.set('defaultPaperSize', defaultPaperSize.value)
      await store.set('defaultCopies', defaultCopies.value)
      await store.set('defaultColor', defaultColor.value)
      await store.set('defaultDirection', defaultDirection.value)

      await store.set('lanPort', lanPort.value)
      await store.set('logLevel', logLevel.value)
      await store.set('autoStart', autoStart.value)

      await store.save()
    } catch (e) {
      console.warn('Failed to save settings to store:', e)
    }
  }

  return {
    allowedExtensions,
    defaultPrinter, defaultPaperSize, defaultCopies, defaultColor, defaultDirection,
    lanPort, logLevel, autoStart,
    getAllowedExtList, loadFromStore, saveToStore,
  }
})
