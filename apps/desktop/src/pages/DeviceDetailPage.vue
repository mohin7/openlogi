<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import BaseCard from '@/components/BaseCard.vue'
import AppIcon from '@/components/AppIcon.vue'
import MouseVisual from '@/components/MouseVisual.vue'
import SliderControl from '@/components/SliderControl.vue'
import BatteryIndicator from '@/components/BatteryIndicator.vue'
import ActionPicker from '@/components/ActionPicker.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { useDeviceStore } from '@/stores/devices'
import { useUiStore } from '@/stores/ui'
import { useSettingsStore } from '@/stores/settings'
import type { ActionPayload, Control } from '@/types/device'

const route = useRoute()
const store = useDeviceStore()
const ui = useUiStore()
const prefs = useSettingsStore()

watch(
  () => route.params.id,
  (id) => id && store.select(String(id)),
  { immediate: true },
)

const device = computed(() => store.selected)
const selectedCid = ref<number | null>(0x0053)

const selectedControl = computed(
  () => device.value?.controls.find((c) => c.cid === selectedCid.value) ?? null,
)

const assigned = computed(() =>
  Object.entries(store.mappings)
    .filter(([, a]) => a.kind !== 'default')
    .map(([cid]) => Number(cid)),
)

const confirmReset = ref(false)
const resetting = ref(false)

/** Anything OpenLogi has changed on this device, for the reset affordance. */
const deviceModified = computed(() => {
  const d = device.value
  if (!d) return false
  const dpiChanged = d.defaultDpi > 0 && d.currentDpi !== d.defaultDpi
  const hasMappings = Object.values(store.mappings).some((a) => a.kind !== 'default')
  return dpiChanged || hasMappings
})

async function doResetDevice() {
  resetting.value = true
  try {
    await store.resetDevice()
    ui.notify('Device reset', 'DPI and button mappings restored to defaults.', 'success')
  } catch (e) {
    ui.notify('Reset failed', e instanceof Error ? e.message : String(e), 'danger')
  } finally {
    resetting.value = false
    confirmReset.value = false
  }
}

const picker = ref(false)
const pickerControl = ref<Control | null>(null)

const DEFAULT_ACTION: ActionPayload = { kind: 'default' }

function currentAction(cid: number): ActionPayload {
  return store.mappings[cid] ?? DEFAULT_ACTION
}

/** Computed so the picker receives a stable reference, not a fresh object
 *  on every render. */
const pickerCurrent = computed<ActionPayload>(() =>
  pickerControl.value ? currentAction(pickerControl.value.cid) : DEFAULT_ACTION,
)

/** Human label for a mapping, used in the button list. */
function actionLabel(cid: number): string {
  const a = currentAction(cid)
  switch (a.kind) {
    case 'default': return 'Default'
    case 'disabled': return 'Disabled'
    case 'keystroke': return a.shortcut
    case 'media': return a.key
    case 'launch': return a.command
    case 'command': return a.script
    case 'url': return a.url
    case 'mouseButton': return `${a.button} click`
    case 'workspace':
      return a.moveWindow ? `Move window ${a.direction}` : `Workspace ${a.direction}`
    case 'appSwitch': {
      const names: Record<string, string> = {
        applications: 'Switch apps',
        windows: 'Switch windows',
        windowsOfApp: 'Windows of app',
        overview: 'Overview',
        appGrid: 'App grid',
      }
      return names[a.mode] + (a.backward ? ' (back)' : '')
    }
  }
}

function onVisualSelect(cid: number) {
  const c = device.value?.controls.find((x) => x.cid === cid)
  if (c) openPicker(c)
}

function openPicker(c: Control) {
  if (!c.divertable && !c.reprogrammable) return
  selectedCid.value = c.cid
  pickerControl.value = c
  picker.value = true
}

async function applyAction(action: ActionPayload) {
  const c = pickerControl.value
  if (!c) return
  picker.value = false
  try {
    await store.setMapping(c.cid, action)
    ui.notify('Button assigned', `${c.name} → ${actionLabel(c.cid)}`, 'success')
  } catch (e) {
    ui.notify('Could not assign', e instanceof Error ? e.message : String(e), 'danger')
  }
}

const dpi = computed({
  get: () => device.value?.currentDpi ?? 1000,
  set: (v: number) => store.setDpi(v),
})

const dpiRange = computed(() => device.value?.capabilities.dpi)
const dpiPresets = computed(() => {
  const r = dpiRange.value
  if (!r) return []
  return [400, 800, 1200, 1600, 2400, 4000].filter((v) => v >= r.min && v <= r.max)
})

</script>

<template>
  <!-- Single root: required for <Transition mode="out-in"> in AppShell.
       A fragment root leaves the leave-transition unable to complete, which
       blanks the content area on the *next* navigation. -->
  <div class="h-full">
    <div v-if="device" class="mx-auto max-w-5xl px-8 py-8">
      <header class="mb-7 animate-rise">
      <div class="flex flex-wrap items-start justify-between gap-5">
        <div class="flex min-w-0 items-start gap-4">
          <div
            class="grid size-12 shrink-0 place-items-center rounded-[12px]"
            :style="{ background: 'var(--surface-raised)', border: '1px solid var(--border)', boxShadow: 'var(--elev-1)' }"
          >
            <AppIcon name="mouse" :size="22" class="text-[var(--text-muted)]" />
          </div>
          <div class="min-w-0">
            <h1 class="text-display truncate">{{ device.name }}</h1>
            <div class="mt-1.5 flex flex-wrap items-center gap-x-2 gap-y-1 text-caption text-[var(--text-muted)]">
              <span
                class="rounded-full border px-2 py-0.5"
                :style="{ borderColor: 'var(--border)' }"
              >{{ device.protocol }}</span>
              <span class="flex items-center gap-1">
                <AppIcon name="zap" :size="11.5" />Logi Bolt
              </span>
              <span class="text-[var(--text-subtle)]">·</span>
              <span>Unit {{ device.unitId }}</span>
            </div>
          </div>
        </div>
        <BatteryIndicator :battery="device.battery" variant="ring" class="shrink-0" />
      </div>
    </header>

    <div class="grid gap-5 lg:grid-cols-[280px_1fr]">
        <!-- Visual -->
        <BaseCard class="self-start">
          <MouseVisual
            :controls="device.controls"
            :selected-cid="selectedCid"
            :assigned="assigned"
            @select="onVisualSelect"
          />
          <p class="mt-4 text-center text-caption text-[var(--text-muted)]">
            Select a button to assign an action
          </p>
        </BaseCard>

        <div class="space-y-5">
          <!-- Remapping needs uinput. Say so loudly rather than letting a
               button be assigned and then silently do nothing. -->
          <BaseCard v-if="!store.engine.available" class="border-amber-400/40">
            <div class="flex items-start gap-3">
              <AppIcon name="alert" :size="17" class="mt-0.5 shrink-0 text-amber-500" />
              <div class="min-w-0">
                <p class="text-label font-semibold">Button actions can't be delivered yet</p>
                <p class="mt-1 text-caption leading-relaxed text-[var(--text-muted)]">
                  {{ store.engine.reason ?? 'Input synthesis is unavailable.' }}
                </p>
              </div>
            </div>
          </BaseCard>

          <!-- Buttons -->
          <BaseCard>
            <h2 class="text-title">Buttons</h2>
            <p class="mt-0.5 text-caption text-[var(--text-muted)]">
              {{ device.controls.filter((c) => c.divertable).length }} of
              {{ device.controls.length }} can be reassigned.
            </p>

            <ul class="mt-3.5 space-y-1.5">
              <li v-for="c in device.controls" :key="c.cid">
                <button
                  class="pressable flex w-full items-center gap-3 rounded-[10px] border px-3 py-2.5 text-left"
                  :class="[
                    selectedCid === c.cid ? 'border-brand-500/60' : 'hover:border-[var(--border-strong)]',
                    !c.divertable && !c.reprogrammable ? 'cursor-not-allowed opacity-60' : '',
                  ]"
                  :style="{
                    background: selectedCid === c.cid ? 'var(--surface-sunken)' : 'transparent',
                    borderColor: selectedCid === c.cid ? undefined : 'var(--border)',
                  }"
                  :disabled="!c.divertable && !c.reprogrammable"
                  @click="openPicker(c)"
                >
                  <span class="min-w-0 flex-1">
                    <span class="flex items-center gap-2 text-label font-medium">
                      {{ c.name }}
                      <AppIcon
                        v-if="!c.divertable && !c.reprogrammable"
                        name="lock"
                        :size="12"
                        class="text-[var(--text-subtle)]"
                      />
                      <span
                        v-if="c.virtual"
                        class="rounded px-1.5 py-px text-[10px] font-semibold tracking-wide text-[var(--text-subtle)] uppercase"
                        :style="{ background: 'var(--surface-sunken)' }"
                      >
                        Virtual
                      </span>
                    </span>
                    <span class="mt-0.5 block text-caption text-[var(--text-muted)]">
                      {{
                        !c.divertable && !c.reprogrammable
                          ? 'Locked by firmware — cannot be reassigned'
                          : actionLabel(c.cid)
                      }}
                    </span>
                  </span>
                  <code class="shrink-0 text-[10.5px] text-[var(--text-subtle)]">
                    0x{{ c.cid.toString(16).padStart(4, '0') }}
                  </code>
                </button>
              </li>
            </ul>

            <p
              v-if="selectedControl && !selectedControl.divertable && !selectedControl.reprogrammable"
              class="mt-4 flex items-start gap-2 rounded-[10px] p-3 text-caption text-[var(--text-muted)]"
              :style="{ background: 'var(--surface-sunken)' }"
            >
              <AppIcon name="lock" :size="14" class="mt-px shrink-0" />
              <span>
                The firmware refuses to divert this control. That is deliberate — remapping the
                primary click could leave you unable to undo it.
              </span>
            </p>
          </BaseCard>

          <!-- Pointer -->
          <BaseCard v-if="dpiRange">
            <div class="flex items-baseline justify-between">
              <h2 class="text-title">Pointer speed</h2>
              <span class="text-caption text-[var(--text-subtle)]">via DPI · 0x2201</span>
            </div>
            <p class="mt-1 mb-5 text-caption text-[var(--text-muted)]">
              This device has no separate pointer-speed feature, so the slider sets sensor DPI
              directly. Saved on the device — it persists with OpenLogi closed.
            </p>
            <SliderControl
              v-model="dpi"
              :min="dpiRange.min"
              :max="dpiRange.max"
              :step="dpiRange.step"
              label="Sensor DPI"
              unit=" dpi"
              :ticks="dpiPresets"
            />
          </BaseCard>

          <!-- Persistence notice -->
          <BaseCard v-if="!device.capabilities.canPersistOnboard">
            <div class="flex items-start gap-3">
              <AppIcon name="info" :size="17" class="mt-0.5 shrink-0 text-[var(--text-muted)]" />
              <div>
                <p class="text-label font-semibold">Button mappings need OpenLogi running</p>
                <p class="mt-1 text-caption leading-relaxed text-[var(--text-muted)]">
                  This device has no onboard mapping storage, so button actions are applied by the
                  background service and stop when it does. DPI is different — it is written to the
                  device and stays.
                </p>
              </div>
            </div>
          </BaseCard>

          <!-- Reset -->
          <BaseCard>
            <div class="flex items-start justify-between gap-6">
              <div class="min-w-0">
                <p class="text-label font-semibold">Reset this device</p>
                <p class="mt-1 text-caption leading-relaxed text-[var(--text-muted)]">
                  Clears every button mapping and puts the sensor back to its factory DPI<span
                    v-if="device.defaultDpi"
                  >
                    ({{ device.defaultDpi }} dpi)</span
                  >. App settings are unaffected.
                </p>
              </div>
              <button
                class="pressable shrink-0 rounded-[9px] border px-3 py-1.5 text-label font-medium disabled:opacity-40"
                :class="deviceModified ? 'hover:border-red-400 hover:text-red-500' : ''"
                :style="{ borderColor: 'var(--border)' }"
                :disabled="!deviceModified || resetting"
                @click="prefs.settings.confirmDestructive ? (confirmReset = true) : doResetDevice()"
              >
                {{ deviceModified ? 'Reset' : 'At defaults' }}
              </button>
            </div>
          </BaseCard>

          <!-- Diagnostics -->
          <BaseCard>
            <h2 class="text-title mb-3.5">Diagnostics</h2>
            <dl class="grid grid-cols-2 gap-x-6 gap-y-2.5 text-label">
              <div v-for="row in [
                ['Unit ID', device.unitId],
                ['Model ID', device.modelId],
                ['Protocol', device.protocol],
                ['Firmware', device.firmware.find((f) => f.kind === 'main')?.version ?? '—'],
                ['Bootloader', device.firmware.find((f) => f.kind === 'bootloader')?.version ?? '—'],
                ['Features', `${device.capabilities.features.length} reported`],
              ]" :key="row[0]" class="flex justify-between gap-4 border-b pb-2.5" :style="{ borderColor: 'var(--border)' }">
                <dt class="text-[var(--text-muted)]">{{ row[0] }}</dt>
                <dd class="truncate font-medium">{{ row[1] }}</dd>
              </div>
            </dl>
            <p class="mt-3.5 text-caption text-[var(--text-subtle)]">
              Polling rate is not exposed over HID++ on this device.
            </p>
          </BaseCard>
        </div>
      </div>
    </div>

    <div v-else class="grid h-full place-items-center text-[14px] text-[var(--text-muted)]">
      Loading device…
    </div>

    <ConfirmDialog
      :open="confirmReset"
      title="Reset this device?"
      message="Every button goes back to its firmware action and the sensor returns to its factory DPI. This cannot be undone."
      confirm-label="Reset device"
      tone="danger"
      :busy="resetting"
      @close="confirmReset = false"
      @confirm="doResetDevice"
    />

    <ActionPicker
      :open="picker"
      :control="pickerControl"
      :current="pickerCurrent"
      @close="picker = false"
      @apply="applyAction"
    />
  </div>
</template>
