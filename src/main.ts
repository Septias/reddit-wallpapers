import generatedRoutes from 'virtual:generated-pages'
import { createRouter, createWebHistory } from 'vue-router'
import { invoke } from '@tauri-apps/api'
import App from './App.vue'

import '@unocss/reset/tailwind.css'
import './styles/main.css'
import 'uno.css'

const routes = generatedRoutes

const app = createApp(App)
const router = createRouter({
  history: createWebHistory(),
  routes,
})
router.beforeEach(async (to) => {
  if (to.path === '/' && !await invoke('is_configured'))
    return '/config'
})
app.use(router)
app.mount('#app')
