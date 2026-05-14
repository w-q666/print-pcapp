import { createApp } from 'vue'
import { createPinia } from 'pinia'
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

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.use(Antd)
app.mount('#app')
