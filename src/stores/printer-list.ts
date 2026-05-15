import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { getPrintServers } from '../api/print-api'
import { useSettings } from './settings'

export const usePrinterList = defineStore('printer-list', () => {
  const settings = useSettings()

  /** Java 返回的全量打印机名（未应用黑名单） */
  const allPrinters = ref<string[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const visiblePrinters = computed(() => {
    const bl = new Set(settings.printerBlacklist)
    return allPrinters.value.filter((p) => !bl.has(p))
  })

  /** 与 `visiblePrinters` 相同，供现有组件使用 */
  const printers = visiblePrinters

  /**
   * 用于 UI 高亮「默认」：优先 settings.defaultPrinter（若在可见列表中），否则取第一个可见项。
   */
  const effectiveDefaultPrinter = computed(() => {
    const def = settings.defaultPrinter.trim()
    const vis = visiblePrinters.value
    if (def && vis.includes(def)) return def
    return vis[0] ?? ''
  })

  async function refresh() {
    loading.value = true
    error.value = null
    try {
      allPrinters.value = await getPrintServers()
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '获取打印机列表失败'
    } finally {
      loading.value = false
    }
  }

  return {
    allPrinters,
    visiblePrinters,
    printers,
    effectiveDefaultPrinter,
    loading,
    error,
    refresh,
  }
})
