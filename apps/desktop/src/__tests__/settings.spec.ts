import { describe, it, expect, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useSettingsStore, DEFAULT_SETTINGS } from '@/stores/settings'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})

describe('settings store', () => {
  it('starts at the shipped defaults', () => {
    const s = useSettingsStore()
    expect(s.settings).toEqual(DEFAULT_SETTINGS)
    expect(s.isModified()).toBe(false)
    expect(s.changedKeys()).toEqual([])
  })

  it('reports exactly what differs from default', () => {
    const s = useSettingsStore()
    s.settings.theme = 'dark'
    s.settings.developerMode = true
    expect(s.isModified()).toBe(true)
    expect(s.changedKeys().sort()).toEqual(['developerMode', 'theme'])
  })

  it('reset restores every key', () => {
    const s = useSettingsStore()
    s.settings.theme = 'dark'
    s.settings.lowBatteryThreshold = 40
    s.reset()
    expect(s.settings).toEqual(DEFAULT_SETTINGS)
  })

  it('a partial stored config is merged over defaults, not trusted wholesale', () => {
    // Simulates an upgrade: a config written before a key existed.
    localStorage.setItem('openlogi.settings', JSON.stringify({ theme: 'dark' }))
    setActivePinia(createPinia())
    const s = useSettingsStore()
    expect(s.settings.theme).toBe('dark')
    // Missing keys must take their default, not become undefined/false.
    expect(s.settings.notifications).toBe(DEFAULT_SETTINGS.notifications)
    expect(s.settings.lowBatteryThreshold).toBe(DEFAULT_SETTINGS.lowBatteryThreshold)
  })

  it('corrupt storage falls back to defaults instead of throwing', () => {
    localStorage.setItem('openlogi.settings', '{not json')
    setActivePinia(createPinia())
    const s = useSettingsStore()
    expect(s.settings).toEqual(DEFAULT_SETTINGS)
  })

  it('DEFAULT_SETTINGS cannot be mutated by accident', () => {
    const s = useSettingsStore()
    s.settings.theme = 'dark'
    // The frozen constant must be untouched by store mutation.
    expect(DEFAULT_SETTINGS.theme).toBe('system')
  })
})
