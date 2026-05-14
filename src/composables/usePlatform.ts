import { ref, onMounted, onUnmounted } from 'vue'

export type Platform = 'desktop' | 'mobile'

const MOBILE_BREAKPOINT = 768

export function usePlatform() {
  const platform = ref<Platform>(
    window.innerWidth < MOBILE_BREAKPOINT ? 'mobile' : 'desktop'
  )

  function onResize() {
    platform.value = window.innerWidth < MOBILE_BREAKPOINT ? 'mobile' : 'desktop'
  }

  async function detect() {
    onResize()
  }

  onMounted(() => {
    window.addEventListener('resize', onResize)
  })

  onUnmounted(() => {
    window.removeEventListener('resize', onResize)
  })

  return { platform, detect }
}
