import { ref, computed, onMounted, onUnmounted, watch } from 'vue'

const BREAKPOINT_AUTO_COLLAPSE = 1024

const windowWidth = ref(window.innerWidth)
const manualCollapsed = ref(false)
let loaded = false
let storeInstance: Awaited<ReturnType<typeof loadStore>> | null = null

async function loadStore() {
  const { Store } = await import('@tauri-apps/plugin-store')
  return Store.load('ui-state.json')
}

export function useNavCollapse() {
  const isAutoCollapsed = computed(
    () => windowWidth.value < BREAKPOINT_AUTO_COLLAPSE && windowWidth.value >= 768
  )

  const collapsed = computed(() => isAutoCollapsed.value || manualCollapsed.value)
  const canToggle = computed(() => !isAutoCollapsed.value)

  function toggle() {
    if (!canToggle.value) return
    manualCollapsed.value = !manualCollapsed.value
  }

  watch(manualCollapsed, async (val) => {
    if (loaded && storeInstance) {
      await storeInstance.set('nav-collapsed', val)
      await storeInstance.save()
    }
  })

  function onResize() {
    windowWidth.value = window.innerWidth
  }

  onMounted(async () => {
    window.addEventListener('resize', onResize)
    if (!loaded) {
      try {
        storeInstance = await loadStore()
        const saved = await storeInstance.get<boolean>('nav-collapsed')
        if (saved !== null && saved !== undefined) {
          manualCollapsed.value = saved
        }
      } catch {
        // Store not available (e.g. in browser dev mode)
      }
      loaded = true
    }
  })

  onUnmounted(() => {
    window.removeEventListener('resize', onResize)
  })

  return { collapsed, canToggle, toggle }
}
