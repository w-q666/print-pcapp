import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useAppConfig = defineStore('app-config', () => {
  const serviceHost = ref('localhost')
  const servicePort = ref(2024)
  const lanPort = ref(5000)

  const serviceUrl = computed(() => `http://${serviceHost.value}:${servicePort.value}`)
  const wsUrl = computed(() => `ws://${serviceHost.value}:${servicePort.value}/print`)

  async function loadFromStore() {
    try {
      const { Store } = await import('@tauri-apps/plugin-store')
      const store = await Store.load('settings.json')
      const host = await store.get<string>('serviceHost')
      const port = await store.get<number>('servicePort')
      const lp = await store.get<number>('lanPort')
      if (host) serviceHost.value = host
      if (port) servicePort.value = port
      if (lp) lanPort.value = lp
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
      await store.save()
    } catch (e) {
      console.warn('Failed to save config to store:', e)
    }
  }

  return { serviceHost, servicePort, lanPort, serviceUrl, wsUrl, loadFromStore, saveToStore }
})
