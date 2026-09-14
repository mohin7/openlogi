import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { backend } from '@/services/backend'
import type { ActionPayload, Device, EngineStatus, Profile } from '@/types/device'

export const useDeviceStore = defineStore('devices', () => {
  const devices = ref<Device[]>([])
  const profiles = ref<Profile[]>([])
  const activeProfileId = ref('default')
  const selectedId = ref<string | null>(null)
  const loading = ref(true)
  const error = ref<string | null>(null)
  /** cid -> action, for the selected device. */
  const mappings = ref<Record<number, ActionPayload>>({})
  const engine = ref<EngineStatus>({ available: false, reason: null })

  const selected = computed(
    () => devices.value.find((d) => d.id === selectedId.value) ?? devices.value[0] ?? null,
  )
  const activeProfile = computed(
    () => profiles.value.find((p) => p.id === activeProfileId.value) ?? profiles.value[0] ?? null,
  )
  const connectedCount = computed(() => devices.value.filter((d) => d.connected).length)
  const lowBattery = computed(() =>
    devices.value.filter((d) => (d.battery.percentage ?? 100) <= 30),
  )

  async function load() {
    loading.value = true
    error.value = null

    // Settled, not `all`. These two calls are independent, and `Promise.all`
    // rejects as a unit — a profiles failure would discard a devices result
    // that had already succeeded, leaving the UI claiming no hardware is
    // connected when it is.
    const [deviceResult, profileResult] = await Promise.allSettled([
      backend.listDevices(),
      backend.listProfiles(),
    ])

    const problems: string[] = []

    if (deviceResult.status === 'fulfilled') {
      devices.value = deviceResult.value
      if (!selectedId.value && deviceResult.value.length) {
        selectedId.value = deviceResult.value[0].id
      }
    } else {
      problems.push(`Devices: ${reasonOf(deviceResult.reason)}`)
    }

    if (profileResult.status === 'fulfilled') {
      profiles.value = profileResult.value
    } else {
      // Non-fatal: fall back to a single default so the device UI still works.
      profiles.value = [
        { id: 'default', name: 'Default', appMatch: null, color: '#00a651', mappings: [], isDefault: true },
      ]
      problems.push(`Profiles: ${reasonOf(profileResult.reason)}`)
    }

    error.value = problems.length ? problems.join(' · ') : null
    loading.value = false

    // Non-blocking: the device list must render even if uinput is unavailable.
    void loadEngineStatus()
  }

  function reasonOf(reason: unknown): string {
    if (reason instanceof Error) return reason.message
    return String(reason)
  }

  async function loadEngineStatus() {
    try {
      engine.value = await backend.engineStatus()
    } catch (e) {
      engine.value = { available: false, reason: reasonOf(e) }
    }
  }

  async function setMapping(cid: number, action: ActionPayload) {
    const device = selected.value
    if (!device) return
    const previous = mappings.value[cid]
    mappings.value = { ...mappings.value, [cid]: action }
    try {
      await backend.setMapping(device.id, cid, action)
    } catch (e) {
      // The device refused it — put the UI back so it never claims a mapping
      // the hardware does not actually have.
      if (previous) mappings.value = { ...mappings.value, [cid]: previous }
      else {
        const next = { ...mappings.value }
        delete next[cid]
        mappings.value = next
      }
      throw new Error(reasonOf(e))
    }
  }

  async function resetDevice() {
    const device = selected.value
    if (!device) return
    await backend.resetDevice(device.id)
    mappings.value = {}
    // Re-read rather than assuming: the device decides what its defaults are.
    await load()
  }

  async function setDpi(dpi: number) {
    const device = selected.value
    if (!device) return
    // Optimistic: the slider must track the thumb, and the device snaps the
    // value to its own step anyway — we reconcile with what it returns.
    const previous = device.currentDpi
    device.currentDpi = dpi
    try {
      device.currentDpi = await backend.setDpi(device.id, dpi)
    } catch {
      device.currentDpi = previous
    }
  }

  function select(id: string) {
    selectedId.value = id
  }

  function setActiveProfile(id: string) {
    activeProfileId.value = id
  }

  return {
    devices, profiles, loading, error, selected, selectedId, activeProfile,
    activeProfileId, connectedCount, lowBattery, mappings, engine,
    load, setDpi, select, setActiveProfile, setMapping, loadEngineStatus, resetDevice,
  }
})
