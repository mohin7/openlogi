/** Mirrors the Rust `hidpp` types so the bridge is a straight serialisation. */

export type ConnectionKind = 'bolt' | 'unifying' | 'bluetooth' | 'usb'
export type ChargeState =
  | 'discharging'
  | 'charging'
  | 'chargingSlow'
  | 'chargeComplete'
  | 'chargeError'
  | 'unknown'

export interface Battery {
  percentage: number | null
  state: ChargeState
  /** Which HID++ feature produced the reading — shown in diagnostics. */
  sourceFeature: number
}

export interface Firmware {
  kind: 'main' | 'bootloader' | 'hardware' | 'other'
  version: string
}

/**
 * A physical or virtual control on the device.
 *
 * `reprogrammable` and `divertable` come straight from feature 0x1B04 and
 * decide what the UI may offer. A control that is neither must be rendered as
 * locked — the firmware will refuse any change, and letting someone try and
 * fail is worse than saying so up front.
 */
export interface Control {
  cid: number
  taskId: number
  name: string
  reprogrammable: boolean
  divertable: boolean
  supportsGestures: boolean
  virtual: boolean
}

export interface DpiRange {
  min: number
  max: number
  step: number
}

export interface DeviceCapabilities {
  /** Feature ids the device reports, so screens gate on fact not device name. */
  features: number[]
  dpi: DpiRange | null
  hasAdjustableDpi: boolean
  hasPointerSpeed: boolean
  hasSmartShift: boolean
  hasHiResWheel: boolean
  hasThumbWheel: boolean
  /** No 0x1C00 / 0x8100 means mappings cannot live on the device. */
  canPersistOnboard: boolean
}

export interface Device {
  id: string
  unitId: string
  name: string
  kind: 'mouse' | 'keyboard' | 'trackball' | 'touchpad' | 'other'
  connection: ConnectionKind
  connected: boolean
  protocol: string
  modelId: string
  battery: Battery
  firmware: Firmware[]
  controls: Control[]
  capabilities: DeviceCapabilities
  currentDpi: number
  /** The sensor's factory default, used by Reset. */
  defaultDpi: number
  lastSeen: string | null
}

export type ActionKind =
  | 'default'
  | 'disabled'
  | 'keystroke'
  | 'media'
  | 'launch'
  | 'command'
  | 'url'
  | 'workspace'
  | 'screenshot'
  | 'macro'
  | 'gesture'

export interface ButtonAction {
  kind: ActionKind
  label: string
  detail?: string
}

export interface Mapping {
  cid: number
  action: ButtonAction
}

/**
 * The action payload sent to Rust. Mirrors `button_engine::Action`, which is a
 * serde-tagged enum: `kind` selects the variant and the remaining fields are
 * that variant's data.
 */
export type ActionPayload =
  | { kind: 'default' }
  | { kind: 'disabled' }
  | { kind: 'keystroke'; shortcut: string }
  | { kind: 'media'; key: string }
  | { kind: 'launch'; command: string }
  | { kind: 'command'; script: string }
  | { kind: 'url'; url: string }
  | { kind: 'mouseButton'; button: string }
  | { kind: 'workspace'; direction: WorkspaceDirection; moveWindow: boolean }
  | { kind: 'appSwitch'; mode: SwitchMode; backward: boolean }

export type WorkspaceDirection = 'left' | 'right' | 'up' | 'down'

export type SwitchMode =
  | 'applications'
  | 'windows'
  | 'windowsOfApp'
  | 'overview'
  | 'appGrid'

export interface EngineStatus {
  available: boolean
  reason: string | null
}

export interface Profile {
  id: string
  name: string
  /** Application that activates this profile, or null for the default one. */
  appMatch: string | null
  color: string
  mappings: Mapping[]
  isDefault: boolean
}
