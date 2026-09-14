<script setup lang="ts">
import { computed, ref } from 'vue'
import BaseCard from '@/components/BaseCard.vue'
import SettingsToggle from '@/components/SettingsToggle.vue'
import SliderControl from '@/components/SliderControl.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import AppIcon from '@/components/AppIcon.vue'
import { useUiStore } from '@/stores/ui'
import { useSettingsStore } from '@/stores/settings'
import { isTauri } from '@/services/backend'

const ui = useUiStore()
const prefs = useSettingsStore()
const s = prefs.settings

const confirmReset = ref(false)
const modified = computed(() => prefs.isModified())
const changed = computed(() => prefs.changedKeys())

const themes = [
  { id: 'light', label: 'Light', icon: 'sun' },
  { id: 'dark', label: 'Dark', icon: 'moon' },
  { id: 'system', label: 'System', icon: 'monitor' },
] as const

function doReset() {
  prefs.reset()
  confirmReset.value = false
  ui.notify('Settings reset', 'Everything is back to its default.', 'success')
}
</script>

<template>
  <div class="mx-auto max-w-3xl px-7 py-7">
    <header class="mb-8 flex items-start justify-between gap-4">
      <div>
        <h1 class="text-display">Settings</h1>
        <p class="mt-1 text-[14px] text-[var(--text-muted)]">
          Appearance, startup and diagnostics.
        </p>
      </div>
      <span
        v-if="modified"
        class="rounded-full px-2.5 py-1 text-caption font-medium text-[var(--text-muted)]"
        :style="{ background: 'var(--surface-raised)' }"
      >
        {{ changed.length }} changed from default
      </span>
    </header>

    <section class="mb-6">
      <h2 class="mb-3 text-overline text-[var(--text-subtle)] uppercase">
        Appearance
      </h2>
      <BaseCard>
        <p class="mb-3 text-label font-semibold">Theme</p>
        <div class="grid grid-cols-3 gap-2">
          <button
            v-for="t in themes"
            :key="t.id"
            class="pressable flex flex-col items-center gap-2 rounded-[10px] border py-4 text-label font-medium transition-colors"
            :class="ui.theme === t.id ? 'border-brand-500' : 'hover:border-[var(--border-strong)]'"
            :style="{
              borderColor: ui.theme === t.id ? undefined : 'var(--border)',
              background: ui.theme === t.id ? 'var(--surface-sunken)' : 'transparent',
            }"
            @click="ui.setTheme(t.id)"
          >
            <AppIcon
              :name="t.icon"
              :size="18"
              :class="ui.theme === t.id ? 'text-brand-500' : 'text-[var(--text-muted)]'"
            />
            {{ t.label }}
          </button>
        </div>
      </BaseCard>
    </section>

    <section class="mb-6">
      <h2 class="mb-3 text-overline text-[var(--text-subtle)] uppercase">
        Startup &amp; notifications
      </h2>
      <BaseCard>
        <div class="divide-y" :style="{ borderColor: 'var(--border)' }">
          <SettingsToggle
            v-model="s.startWithSystem"
            label="Start with Linux"
            description="Runs the background service as a systemd user unit. Button mappings only apply while it is running."
          />
          <SettingsToggle
            v-model="s.minimiseToTray"
            label="Close to tray"
            description="Closing the window keeps the service running instead of quitting."
          />
          <SettingsToggle
            v-model="s.notifications"
            label="Desktop notifications"
            description="Device connected and disconnected events."
          />
          <SettingsToggle
            v-model="s.lowBatteryWarning"
            label="Low battery warning"
            description="Warn once when a device drops below the threshold."
          />
        </div>

        <div v-if="s.lowBatteryWarning" class="mt-4 border-t pt-4" :style="{ borderColor: 'var(--border)' }">
          <SliderControl
            v-model="s.lowBatteryThreshold"
            :min="5"
            :max="50"
            :step="5"
            label="Warn below"
            unit="%"
          />
        </div>
      </BaseCard>
    </section>

    <section class="mb-6">
      <h2 class="mb-3 text-overline text-[var(--text-subtle)] uppercase">
        System
      </h2>
      <BaseCard>
        <dl class="space-y-2.5 text-label">
          <div class="flex justify-between gap-4">
            <dt class="text-[var(--text-muted)]">Display server</dt>
            <dd class="font-medium">{{ isTauri ? 'X11' : '—' }}</dd>
          </div>
          <div class="flex justify-between gap-4">
            <dt class="text-[var(--text-muted)]">Input synthesis</dt>
            <dd class="font-medium">uinput</dd>
          </div>
          <div class="flex justify-between gap-4">
            <dt class="text-[var(--text-muted)]">Backend</dt>
            <dd class="font-medium">{{ isTauri ? 'Connected' : 'Browser preview' }}</dd>
          </div>
        </dl>
      </BaseCard>
    </section>

    <section class="mb-6">
      <h2 class="mb-3 text-overline text-[var(--text-subtle)] uppercase">
        Advanced
      </h2>
      <BaseCard>
        <div class="divide-y" :style="{ borderColor: 'var(--border)' }">
          <SettingsToggle
            v-model="s.confirmDestructive"
            label="Confirm before resetting"
            description="Ask before anything that discards configuration."
          />
          <SettingsToggle
            v-model="s.developerMode"
            label="Developer mode"
            description="Show raw HID++ feature ids, control ids and packet logs."
          />
        </div>
      </BaseCard>
    </section>

    <!-- Reset -->
    <section>
      <h2 class="mb-3 text-overline text-[var(--text-subtle)] uppercase">
        Reset
      </h2>
      <BaseCard>
        <div class="flex items-start justify-between gap-6">
          <div class="min-w-0">
            <p class="text-label font-semibold">Restore default settings</p>
            <p class="mt-1 text-caption leading-relaxed text-[var(--text-muted)]">
              Puts every option on this page back to how OpenLogi ships. Your button mappings and
              device DPI are not touched — reset those from the device page.
            </p>
            <p v-if="modified" class="mt-2 text-caption text-[var(--text-subtle)]">
              Currently changed: {{ changed.join(', ') }}
            </p>
          </div>
          <button
            class="shrink-0 rounded-[10px] border px-3.5 py-2 text-label font-medium transition-colors disabled:opacity-40"
            :class="modified ? 'hover:border-red-400 hover:text-red-500' : ''"
            :style="{ borderColor: 'var(--border)' }"
            :disabled="!modified"
            @click="s.confirmDestructive ? (confirmReset = true) : doReset()"
          >
            {{ modified ? 'Reset' : 'At defaults' }}
          </button>
        </div>
      </BaseCard>
    </section>

    <ConfirmDialog
      :open="confirmReset"
      title="Restore default settings?"
      :message="`This resets ${changed.length} changed setting${changed.length === 1 ? '' : 's'}. Button mappings and device DPI are unaffected.`"
      confirm-label="Reset settings"
      tone="danger"
      @close="confirmReset = false"
      @confirm="doReset"
    />
  </div>
</template>
