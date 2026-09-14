import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
import { useSettingsStore, type Theme } from './settings'

interface Toast {
  id: number
  title: string
  detail?: string
  tone: 'default' | 'success' | 'warning' | 'danger'
}

/**
 * Transient interface state.
 *
 * Anything the user would expect to survive a restart lives in the settings
 * store instead; this store holds the things that should not (search text,
 * toasts) and provides the theme/sidebar accessors that apply persisted
 * settings to the DOM.
 */
export const useUiStore = defineStore('ui', () => {
  const prefs = useSettingsStore()
  const search = ref('')
  const toasts = ref<Toast[]>([])
  let nextId = 1

  const theme = computed<Theme>({
    get: () => prefs.settings.theme,
    set: (v) => (prefs.settings.theme = v),
  })
  const sidebarCollapsed = computed<boolean>({
    get: () => prefs.settings.sidebarCollapsed,
    set: (v) => (prefs.settings.sidebarCollapsed = v),
  })

  function resolved(): 'light' | 'dark' {
    if (theme.value !== 'system') return theme.value
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }

  function apply() {
    document.documentElement.classList.toggle('dark', resolved() === 'dark')
  }

  function setTheme(next: Theme) {
    const root = document.documentElement
    // Cross-fade colour only; the class is removed afterwards so the
    // transition never applies during ordinary interaction.
    root.classList.add('theme-transition')
    theme.value = next
    window.setTimeout(() => root.classList.remove('theme-transition'), 300)
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  function notify(title: string, detail?: string, tone: Toast['tone'] = 'default') {
    const id = nextId++
    toasts.value.push({ id, title, detail, tone })
    window.setTimeout(() => {
      toasts.value = toasts.value.filter((t) => t.id !== id)
    }, 4000)
  }

  function dismiss(id: number) {
    toasts.value = toasts.value.filter((t) => t.id !== id)
  }

  // Re-apply whenever the persisted theme changes, including via a reset.
  watch(theme, apply)
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (theme.value === 'system') apply()
  })
  apply()

  return { theme, sidebarCollapsed, search, toasts, setTheme, toggleSidebar, notify, dismiss, resolved }
})
