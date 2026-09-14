<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import BaseCard from '@/components/BaseCard.vue'
import DeviceCard from '@/components/DeviceCard.vue'
import AppIcon from '@/components/AppIcon.vue'
import BatteryIndicator from '@/components/BatteryIndicator.vue'
import { useDeviceStore } from '@/stores/devices'
import { useUiStore } from '@/stores/ui'
import { useSettingsStore } from '@/stores/settings'

const store = useDeviceStore()
const ui = useUiStore()
const prefs = useSettingsStore()
const router = useRouter()

const filtered = computed(() => {
  const q = ui.search.trim().toLowerCase()
  if (!q) return store.devices
  return store.devices.filter(
    (d) => d.name.toLowerCase().includes(q) || d.connection.includes(q),
  )
})

/** The single device that deserves the hero slot. */
const primary = computed(() => store.selected ?? store.devices[0] ?? null)

const lowBattery = computed(() =>
  store.devices.filter(
    (d) => (d.battery.percentage ?? 100) <= prefs.settings.lowBatteryThreshold,
  ),
)

const quickActions = [
  { icon: 'layers', label: 'Profiles', detail: 'Per-app button behaviour', to: '/profiles' },
  { icon: 'gesture', label: 'Gestures', detail: 'Hold and swipe actions', to: '/gestures' },
  { icon: 'command', label: 'Macros', detail: 'Recorded key sequences', to: '/macros' },
]

function open(id: string) {
  store.select(id)
  router.push(`/device/${id}`)
}
</script>

<template>
  <div class="mx-auto max-w-4xl px-7 py-7">
    <header class="mb-7 flex items-end justify-between gap-6">
      <div>
        <h1 class="text-display">Devices</h1>
        <p class="mt-1 text-body text-[var(--text-muted)]">
          Your Logitech hardware, configured natively.
        </p>
      </div>
      <BatteryIndicator
        v-if="primary && !store.loading"
        :battery="primary.battery"
        variant="ring"
        class="shrink-0"
      />
    </header>

    <!-- Backend problems must be visible. A silent empty list is
         indistinguishable from "no hardware", which sends people hunting
         their receiver when the fault is ours. -->
    <BaseCard v-if="store.error" class="mb-4 border-red-400/50">
      <div class="flex items-start gap-3">
        <AppIcon name="alert" :size="16" class="mt-0.5 shrink-0 text-red-500" />
        <div class="min-w-0 flex-1">
          <p class="text-label font-semibold">Couldn't read everything</p>
          <p class="mt-0.5 font-mono text-caption break-words text-[var(--text-muted)]">
            {{ store.error }}
          </p>
        </div>
        <button
          class="pressable shrink-0 rounded-[8px] border px-2.5 py-1 text-caption font-medium hover:bg-[var(--surface-hover)]"
          :style="{ borderColor: 'var(--border)' }"
          @click="store.load()"
        >
          Retry
        </button>
      </div>
    </BaseCard>

    <!-- Skeletons reserve the exact card height, so nothing jumps on load. -->
    <div v-if="store.loading" class="grid gap-3 sm:grid-cols-2">
      <div
        v-for="i in 2"
        :key="i"
        class="h-[132px] animate-pulse rounded-[var(--radius-card)] border"
        :style="{ background: 'var(--surface-raised)', borderColor: 'var(--border)' }"
      />
    </div>

    <BaseCard v-else-if="!filtered.length" class="py-14 text-center">
      <div
        class="mx-auto grid size-11 place-items-center rounded-[12px]"
        :style="{ background: 'var(--surface-sunken)' }"
      >
        <AppIcon name="mouse" :size="20" class="text-[var(--text-subtle)]" />
      </div>
      <h3 class="mt-3.5 text-title">
        {{ ui.search ? 'No matches' : 'No devices found' }}
      </h3>
      <p class="mx-auto mt-1 max-w-xs text-label text-[var(--text-muted)]">
        {{
          ui.search
            ? 'Try a different search term.'
            : 'Switch your mouse on and make sure it is paired, then rescan.'
        }}
      </p>
      <button
        v-if="!ui.search"
        class="pressable bg-brand-500 hover:bg-brand-600 mt-4 rounded-[9px] px-3.5 py-2 text-label font-medium text-white"
        @click="store.load()"
      >
        Rescan
      </button>
    </BaseCard>

    <template v-else>
      <section class="stagger grid gap-3 sm:grid-cols-2">
        <DeviceCard
          v-for="d in filtered"
          :key="d.id"
          :device="d"
          :active="store.selectedId === d.id"
          @select="open(d.id)"
        />
      </section>

      <BaseCard v-if="lowBattery.length" class="mt-3 border-amber-400/50">
        <div class="flex items-center gap-3">
          <AppIcon name="alert" :size="16" class="shrink-0 text-amber-500" />
          <p class="min-w-0 flex-1 text-label">
            <span class="font-semibold">Battery low.</span>
            <span class="text-[var(--text-muted)]">
              {{ lowBattery.map((d) => d.name).join(', ') }} — a weak link causes dropouts.
            </span>
          </p>
        </div>
      </BaseCard>

      <section class="mt-8">
        <h2 class="mb-2.5 text-overline text-[var(--text-subtle)] uppercase">Configure</h2>
        <div class="stagger grid gap-3 sm:grid-cols-3">
          <BaseCard
            v-for="a in quickActions"
            :key="a.label"
            interactive
            @click="router.push(a.to)"
          >
            <AppIcon :name="a.icon" :size="17" class="text-[var(--text-muted)]" />
            <p class="mt-2.5 text-label font-semibold">{{ a.label }}</p>
            <p class="mt-0.5 text-caption text-[var(--text-muted)]">{{ a.detail }}</p>
          </BaseCard>
        </div>
      </section>

      <section v-if="store.profiles.length" class="mt-8">
        <h2 class="mb-2.5 text-overline text-[var(--text-subtle)] uppercase">Profiles</h2>
        <div class="flex flex-wrap gap-2">
          <button
            v-for="p in store.profiles"
            :key="p.id"
            class="pressable flex items-center gap-2 rounded-full border px-3 py-1.5 text-label font-medium hover:border-[var(--border-strong)]"
            :style="{
              background: 'var(--surface-raised)',
              borderColor: store.activeProfileId === p.id ? 'var(--color-brand-500)' : 'var(--border)',
            }"
            @click="store.setActiveProfile(p.id); ui.notify('Profile activated', p.name, 'success')"
          >
            <span class="size-2 rounded-full" :style="{ background: p.color }" />
            {{ p.name }}
            <AppIcon
              v-if="store.activeProfileId === p.id"
              name="check"
              :size="13"
              class="text-brand-500"
            />
          </button>
        </div>
      </section>
    </template>
  </div>
</template>
