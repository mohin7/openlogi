/**
 * The single boundary between the UI and the Rust side.
 *
 * In a Tauri window this calls into the daemon. In a plain browser — which is
 * how the UI is developed and reviewed — it serves the capability snapshot
 * captured from real hardware by `logi-probe`. Every field below is measured,
 * not invented, so the UI is built against what the device actually reports.
 *
 * See docs/devices/signature-m650.md for the capture.
 */
import type { ActionPayload, Device, EngineStatus, Profile } from '@/types/device'

export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

/** Feature ids the Signature M650 reports (0x0001 enumeration). */
const M650_FEATURES = [
  0x0000, 0x0001, 0x0003, 0x0005, 0x1d4b, 0x0020, 0x0007, 0x1004, 0x1b04,
  0x1815, 0x2250, 0x2130, 0x2201, 0x00c3, 0x1602,
]

const m650: Device = {
  id: '481871d4',
  unitId: '481871D4',
  name: 'Signature M650',
  kind: 'mouse',
  connection: 'bolt',
  connected: true,
  protocol: 'HID++ 4.5',
  modelId: 'B02A',
  battery: { percentage: 30, state: 'discharging', sourceFeature: 0x1004 },
  firmware: [
    { kind: 'main', version: 'RBM16.02.B0013' },
    { kind: 'bootloader', version: 'BL138.02.B0013' },
  ],
  // Exactly the six controls feature 0x1B04 reports, with the real flags.
  controls: [
    { cid: 0x0050, taskId: 0x0038, name: 'Left Click', reprogrammable: false, divertable: false, supportsGestures: false, virtual: false },
    { cid: 0x0051, taskId: 0x0039, name: 'Right Click', reprogrammable: false, divertable: false, supportsGestures: false, virtual: false },
    { cid: 0x0052, taskId: 0x003a, name: 'Middle Click', reprogrammable: true, divertable: true, supportsGestures: true, virtual: false },
    { cid: 0x0053, taskId: 0x003c, name: 'Back', reprogrammable: true, divertable: true, supportsGestures: true, virtual: false },
    { cid: 0x0056, taskId: 0x003e, name: 'Forward', reprogrammable: true, divertable: true, supportsGestures: true, virtual: false },
    { cid: 0x00d7, taskId: 0x00b4, name: 'Gesture', reprogrammable: false, divertable: true, supportsGestures: true, virtual: true },
  ],
  capabilities: {
    features: M650_FEATURES,
    dpi: { min: 400, max: 4000, step: 100 },
    hasAdjustableDpi: true,
    // No 0x2205 — the pointer-speed slider is backed by DPI instead.
    hasPointerSpeed: false,
    hasSmartShift: false,
    hasHiResWheel: false,
    hasThumbWheel: false,
    // No 0x1C00 and no 0x8100: mappings cannot be stored on the device.
    canPersistOnboard: false,
  },
  currentDpi: 1000,
  defaultDpi: 1000,
  lastSeen: null,
}

const defaultProfiles: Profile[] = [
  {
    id: 'default',
    name: 'Default',
    appMatch: null,
    color: '#00a651',
    isDefault: true,
    mappings: [
      { cid: 0x0052, action: { kind: 'default', label: 'Middle Click' } },
      { cid: 0x0053, action: { kind: 'default', label: 'Back' } },
      { cid: 0x0056, action: { kind: 'default', label: 'Forward' } },
      { cid: 0x00d7, action: { kind: 'default', label: 'Off' } },
    ],
  },
  {
    id: 'figma',
    name: 'Figma',
    appMatch: 'figma',
    color: '#a259ff',
    isDefault: false,
    mappings: [
      { cid: 0x0052, action: { kind: 'keystroke', label: 'Zoom to fit', detail: 'Shift+1' } },
      { cid: 0x0053, action: { kind: 'keystroke', label: 'Undo', detail: 'Ctrl+Z' } },
      { cid: 0x0056, action: { kind: 'keystroke', label: 'Redo', detail: 'Ctrl+Shift+Z' } },
      { cid: 0x00d7, action: { kind: 'gesture', label: 'Gestures' } },
    ],
  },
  {
    id: 'code',
    name: 'VS Code',
    appMatch: 'code',
    color: '#0098ff',
    isDefault: false,
    mappings: [
      { cid: 0x0052, action: { kind: 'keystroke', label: 'Command Palette', detail: 'Ctrl+Shift+P' } },
      { cid: 0x0053, action: { kind: 'keystroke', label: 'Go Back', detail: 'Ctrl+Alt+-' } },
      { cid: 0x0056, action: { kind: 'keystroke', label: 'Go Forward', detail: 'Ctrl+Shift+-' } },
      { cid: 0x00d7, action: { kind: 'default', label: 'Off' } },
    ],
  },
]

function delay<T>(value: T, ms = 320): Promise<T> {
  return new Promise((resolve) => setTimeout(() => resolve(value), ms))
}

export const backend = {
  async listDevices(): Promise<Device[]> {
    if (isTauri) {
      const { invoke } = await import('@tauri-apps/api/core')
      return invoke<Device[]>('list_devices')
    }
    return delay([m650])
  },

  async listProfiles(): Promise<Profile[]> {
    if (isTauri) {
      const { invoke } = await import('@tauri-apps/api/core')
      return invoke<Profile[]>('list_profiles')
    }
    return delay(defaultProfiles, 120)
  },

  async engineStatus(): Promise<EngineStatus> {
    if (isTauri) {
      const { invoke } = await import('@tauri-apps/api/core')
      return invoke<EngineStatus>('engine_status')
    }
    return delay(
      { available: false, reason: 'Browser preview — input synthesis needs the desktop app.' },
      60,
    )
  },

  async setMapping(deviceId: string, cid: number, action: ActionPayload): Promise<void> {
    if (isTauri) {
      const { invoke } = await import('@tauri-apps/api/core')
      return invoke<void>('set_mapping', { deviceId, cid, action })
    }
    return delay(undefined, 80)
  },

  async resetDevice(deviceId: string): Promise<void> {
    if (isTauri) {
      const { invoke } = await import('@tauri-apps/api/core')
      return invoke<void>('reset_device', { deviceId })
    }
    return delay(undefined, 200)
  },

  async setDpi(deviceId: string, dpi: number): Promise<number> {
    if (isTauri) {
      const { invoke } = await import('@tauri-apps/api/core')
      return invoke<number>('set_dpi', { deviceId, dpi })
    }
    m650.currentDpi = dpi
    return delay(dpi, 60)
  },
}
