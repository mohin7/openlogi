import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import { router } from './router'
import './style.css'
import { installDiagnostics, logNavigation } from './services/diagnostics'

installDiagnostics()

// Log every navigation. When someone reports "it froze when I clicked X", the
// last line in the log names X.
router.afterEach((to) => logNavigation(to.fullPath))

const app = createApp(App)

app.config.errorHandler = (err, _instance, info) => {
  console.error('[openlogi] Vue error', err, info)
  void import('./services/diagnostics').then(() => {
    window.dispatchEvent(
      new ErrorEvent('error', {
        message: err instanceof Error ? err.message : String(err),
        error: err instanceof Error ? err : undefined,
      }),
    )
  })
}

app.use(createPinia()).use(router).mount('#app')
