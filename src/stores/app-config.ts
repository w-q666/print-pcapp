import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useAppConfig = defineStore('app-config', () => {
  const serviceHost = ref('localhost')
  const servicePort = ref(2024)
  const lanPort = ref(5000)
  const scanStartIp = ref('')
  const scanEndIp = ref('')
  /** 已自动推算过网段，或用户保存过「双空」范围后，为 true 则不再自动填网段 */
  const scanRangeInferredOnce = ref(false)
  /** store 中是否已有非空的 serviceHost（用于首次启动向导） */
  const isServiceHostPersisted = ref(false)

  const serviceUrl = computed(() => `http://${serviceHost.value}:${servicePort.value}`)
  const wsUrl = computed(() => `ws://${serviceHost.value}:${servicePort.value}/print`)

  async function loadFromStore() {
    try {
      const { Store } = await import('@tauri-apps/plugin-store')
      const store = await Store.load('settings.json')
      const host = await store.get<string>('serviceHost')
      isServiceHostPersisted.value =
        host !== null && host !== undefined && String(host).trim() !== ''
      const port = await store.get<number>('servicePort')
      const lp = await store.get<number>('lanPort')
      const ss = await store.get<string>('scanStartIp')
      const se = await store.get<string>('scanEndIp')
      const sri = await store.get<boolean>('scanRangeInferredOnce')
      if (isServiceHostPersisted.value) serviceHost.value = String(host).trim()
      if (port !== null && port !== undefined) {
        const n = Number(port)
        if (Number.isInteger(n) && n >= 1 && n <= 65535) {
          servicePort.value = n
        }
      }
      if (lp) lanPort.value = lp
      if (ss !== null && ss !== undefined) scanStartIp.value = ss
      if (se !== null && se !== undefined) scanEndIp.value = se
      if (sri !== null && sri !== undefined) scanRangeInferredOnce.value = sri
    } catch (e) {
      console.warn('Failed to load config from store:', e)
    }
  }

  async function saveToStore() {
    try {
      const { Store } = await import('@tauri-apps/plugin-store')
      const store = await Store.load('settings.json')
      await store.set('serviceHost', serviceHost.value)
      await store.set('servicePort', servicePort.value)
      await store.set('lanPort', lanPort.value)
      await store.set('scanStartIp', scanStartIp.value)
      await store.set('scanEndIp', scanEndIp.value)
      await store.set('scanRangeInferredOnce', scanRangeInferredOnce.value)
      await store.save()
      isServiceHostPersisted.value = serviceHost.value.trim() !== ''
    } catch (e) {
      console.warn('Failed to save config to store:', e)
    }
  }

  return {
    serviceHost,
    servicePort,
    lanPort,
    scanStartIp,
    scanEndIp,
    scanRangeInferredOnce,
    isServiceHostPersisted,
    serviceUrl,
    wsUrl,
    loadFromStore,
    saveToStore,
  }
})
