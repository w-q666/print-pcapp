import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import Antd from 'ant-design-vue'
import 'ant-design-vue/dist/reset.css'
import './assets/styles/variables.css'
import './assets/styles/global.css'
import './assets/styles/layout.css'
import './assets/styles/titlebar.css'
import './assets/styles/nav-sidebar.css'
import './assets/styles/base-page.css'
import router from './router'
import App from './App.vue'

function logGlobalError(message: string) {
  invoke('log_insert', {
    level: 'ERROR',
    category: 'error',
    message,
    logger: 'frontend:global-error-handler',
  }).catch(() => {})
}

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.use(Antd)

app.config.errorHandler = (err, instance, info) => {
  const name = instance?.$options?.name || 'unknown'
  logGlobalError(`[Vue] ${err} | component: ${name} | ${info}`)
  console.error('[Vue Error]', err, instance, info)
}

window.onerror = (message, source, lineno, colno) => {
  logGlobalError(`[Runtime] ${message} | ${source}:${lineno}:${colno}`)
}

window.addEventListener('unhandledrejection', (event) => {
  logGlobalError(`[Promise] ${event.reason}`)
})

if (!import.meta.env.DEV) {
  document.addEventListener('contextmenu', (e) => e.preventDefault())
}

app.mount('#app')
