import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useDeviceStore } from '@/stores/devices'
import { backend } from '@/services/backend'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
  vi.restoreAllMocks()
})

describe('device store', () => {
  it('loads devices from the backend', async () => {
    const s = useDeviceStore()
    await s.load()
    expect(s.devices.length).toBe(1)
    expect(s.selected?.name).toBe('Signature M650')
    expect(s.connectedCount).toBe(1)
  })

  it('still shows devices when the profiles call fails', async () => {
    // Regression: Promise.all rejected as a unit, so a profiles failure
    // discarded a devices result that had already succeeded and the UI
    // claimed no hardware was connected.
    vi.spyOn(backend, 'listProfiles').mockRejectedValue(new Error('no such command'))
    const s = useDeviceStore()
    await s.load()
    expect(s.devices.length).toBe(1)
    expect(s.error).toContain('Profiles')
    // And a usable fallback profile exists.
    expect(s.profiles.length).toBeGreaterThan(0)
  })

  it('surfaces a device failure instead of failing silently', async () => {
    vi.spyOn(backend, 'listDevices').mockRejectedValue(new Error('hidraw permission denied'))
    const s = useDeviceStore()
    await s.load()
    expect(s.devices.length).toBe(0)
    expect(s.error).toContain('hidraw permission denied')
  })

  it('rolls a mapping back when the device refuses it', async () => {
    const s = useDeviceStore()
    await s.load()
    vi.spyOn(backend, 'setMapping').mockRejectedValue(new Error('InvalidArgument'))
    await expect(s.setMapping(0x53, { kind: 'keystroke', shortcut: 'ctrl+z' })).rejects.toThrow()
    // The UI must never claim a mapping the hardware did not accept.
    expect(s.mappings[0x53]).toBeUndefined()
  })

  it('keeps the previous mapping when a change is refused', async () => {
    const s = useDeviceStore()
    await s.load()
    await s.setMapping(0x53, { kind: 'keystroke', shortcut: 'ctrl+z' })
    vi.spyOn(backend, 'setMapping').mockRejectedValue(new Error('Busy'))
    await expect(s.setMapping(0x53, { kind: 'disabled' })).rejects.toThrow()
    expect(s.mappings[0x53]).toEqual({ kind: 'keystroke', shortcut: 'ctrl+z' })
  })

  it('flags low battery using the real reading', async () => {
    const s = useDeviceStore()
    await s.load()
    expect(s.lowBattery.map((d) => d.name)).toEqual(['Signature M650'])
  })
})
