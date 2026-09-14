import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export type Theme = 'light' | 'dark' | 'system'
export type UpdateChannel = 'stable' | 'beta'

export interface Settings {
  theme: Theme
  startWithSystem: boolean
  minimiseToTray: boolean
  notifications: boolean
  lowBatteryWarning: boolean
  /** Percentage at which the low-battery warning fires. */
  lowBatteryThreshold: number
  confirmDestructive: boolean
  updateChannel: UpdateChannel
  developerMode: boolean
  sidebarCollapsed: boolean
}

/**
 * The single source of truth for what "default" means.
 *
 * Chosen so a first run is useful and quiet: the app follows the system theme,
 * starts with the session (button mappings only work while it runs), and warns
 * about battery once — at 15%, where a Logitech AA cell has days rather than
 * weeks left. Developer mode is off because raw feature ids help nobody who
 * has not asked for them.
 */
export const DEFAULT_SETTINGS: Readonly<Settings> = Object.freeze({
  theme: 'system',
  startWithSystem: true,
  minimiseToTray: true,
  notifications: true,
  lowBatteryWarning: true,
  lowBatteryThreshold: 15,
  confirmDestructive: true,
  updateChannel: 'stable',
  developerMode: false,
  sidebarCollapsed: false,
})

const STORAGE_KEY = 'openlogi.settings'

function load(): Settings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return { ...DEFAULT_SETTINGS }
    // Merge over defaults rather than trusting the stored shape: a settings
    // file written by an older version is missing keys a newer one needs, and
    // an undefined toggle renders as "off" rather than as its default.
    return { ...DEFAULT_SETTINGS, ...(JSON.parse(raw) as Partial<Settings>) }
  } catch {
    return { ...DEFAULT_SETTINGS }
  }
}

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Settings>(load())

  watch(
    settings,
    (value) => {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(value))
      } catch {
        // Storage can be unavailable (private mode, quota). Losing persistence
        // is not worth breaking the app over.
      }
    },
    { deep: true },
  )

  /** True when anything differs from the shipped defaults. */
  function isModified(): boolean {
    return (Object.keys(DEFAULT_SETTINGS) as (keyof Settings)[]).some(
      (k) => settings.value[k] !== DEFAULT_SETTINGS[k],
    )
  }

  function changedKeys(): (keyof Settings)[] {
    return (Object.keys(DEFAULT_SETTINGS) as (keyof Settings)[]).filter(
      (k) => settings.value[k] !== DEFAULT_SETTINGS[k],
    )
  }

  function reset() {
    settings.value = { ...DEFAULT_SETTINGS }
  }

  function resetKey<K extends keyof Settings>(key: K) {
    settings.value[key] = DEFAULT_SETTINGS[key]
  }

  return { settings, DEFAULT_SETTINGS, isModified, changedKeys, reset, resetKey }
})
