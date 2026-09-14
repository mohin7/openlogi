<script setup lang="ts">
import { RouterLink, useRoute } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import { useUiStore } from '@/stores/ui'
import { useDeviceStore } from '@/stores/devices'

const ui = useUiStore()
const devices = useDeviceStore()
const route = useRoute()

const primary = [
  { to: '/', icon: 'mouse', label: 'Devices' },
  { to: '/profiles', icon: 'layers', label: 'Profiles' },
  { to: '/gestures', icon: 'gesture', label: 'Gestures' },
  { to: '/macros', icon: 'command', label: 'Macros' },
]
const secondary = [
  { to: '/settings', icon: 'settings', label: 'Settings' },
  { to: '/about', icon: 'info', label: 'About' },
]

function isActive(to: string) {
  return to === '/'
    ? route.path === '/' || route.path.startsWith('/device')
    : route.path.startsWith(to)
}

function batteryTone(pct: number | null) {
  if (pct === null) return 'text-[var(--text-subtle)]'
  if (pct <= 15) return 'text-red-500'
  if (pct <= 30) return 'text-amber-500'
  return 'text-[var(--text-subtle)]'
}
</script>

<template>
  <aside
    class="flex shrink-0 flex-col border-r transition-[width] duration-300 ease-[var(--ease-out-quint)]"
    :class="ui.sidebarCollapsed ? 'w-[60px]' : 'w-[224px]'"
    :style="{ borderColor: 'var(--border)', background: 'var(--surface)' }"
  >
    <!-- Wordmark -->
    <div class="flex h-14 shrink-0 items-center gap-2.5 px-3.5">
      <div
        class="grid size-[26px] shrink-0 place-items-center rounded-[8px] text-white"
        :style="{
          background: 'linear-gradient(160deg, var(--color-brand-400), var(--color-brand-600))',
          boxShadow: '0 1px 3px rgb(0 118 60 / 0.35)',
        }"
      >
        <AppIcon name="mouse" :size="15" />
      </div>
      <Transition
        enter-active-class="transition-opacity duration-200 delay-100"
        enter-from-class="opacity-0"
        leave-active-class="transition-opacity duration-75"
        leave-to-class="opacity-0"
      >
        <span v-if="!ui.sidebarCollapsed" class="text-title tracking-[-0.015em]">OpenLogi</span>
      </Transition>
    </div>

    <nav class="flex-1 overflow-y-auto px-2.5 pb-2">
      <ul class="space-y-0.5">
        <li v-for="item in primary" :key="item.to">
          <RouterLink
            :to="item.to"
            class="pressable group relative flex items-center gap-2.5 rounded-[9px] px-2.5 py-[7px] text-label font-medium"
            :class="
              isActive(item.to)
                ? 'text-[var(--text)]'
                : 'text-[var(--text-muted)] hover:bg-[var(--surface-hover)] hover:text-[var(--text)]'
            "
            :style="isActive(item.to) ? { background: 'var(--surface-hover)' } : undefined"
            :title="ui.sidebarCollapsed ? item.label : undefined"
            :aria-current="isActive(item.to) ? 'page' : undefined"
          >
            <!-- Active marker: a rail rather than a filled block, so the
                 accent marks position without shouting. -->
            <span
              class="bg-brand-500 absolute top-1/2 -left-2.5 h-[18px] w-[3px] origin-center -translate-y-1/2 rounded-r-full transition-transform duration-200 ease-[var(--ease-spring)]"
              :class="isActive(item.to) ? 'scale-y-100' : 'scale-y-0'"
            />
            <AppIcon :name="item.icon" :size="16.5" class="shrink-0" />
            <span v-if="!ui.sidebarCollapsed" class="truncate">{{ item.label }}</span>
          </RouterLink>
        </li>
      </ul>

      <!-- Connected devices -->
      <div v-if="!ui.sidebarCollapsed && devices.devices.length" class="mt-6">
        <p class="px-2.5 pb-1.5 text-overline text-[var(--text-subtle)] uppercase">Connected</p>
        <ul class="space-y-0.5">
          <li v-for="d in devices.devices" :key="d.id">
            <RouterLink
              :to="`/device/${d.id}`"
              class="pressable flex items-center gap-2.5 rounded-[9px] px-2.5 py-[7px] text-label text-[var(--text-muted)] hover:bg-[var(--surface-hover)] hover:text-[var(--text)]"
              :style="route.path === `/device/${d.id}` ? { background: 'var(--surface-hover)', color: 'var(--text)' } : undefined"
            >
              <span class="relative flex size-1.5 shrink-0">
                <span
                  v-if="d.connected"
                  class="bg-brand-500 absolute inline-flex size-full animate-ping rounded-full opacity-60"
                  style="animation-duration: 2.5s"
                />
                <span
                  class="relative inline-flex size-1.5 rounded-full"
                  :class="d.connected ? 'bg-brand-500' : 'bg-[var(--border-strong)]'"
                />
              </span>
              <span class="truncate">{{ d.name }}</span>
              <span class="ml-auto text-caption tabular-nums" :class="batteryTone(d.battery.percentage)">
                {{ d.battery.percentage ?? '—' }}%
              </span>
            </RouterLink>
          </li>
        </ul>
      </div>
    </nav>

    <!-- Secondary nav sits apart from primary, per navigation hierarchy -->
    <div class="border-t px-2.5 py-2" :style="{ borderColor: 'var(--border)' }">
      <ul class="space-y-0.5">
        <li v-for="item in secondary" :key="item.to">
          <RouterLink
            :to="item.to"
            class="pressable relative flex items-center gap-2.5 rounded-[9px] px-2.5 py-[7px] text-label font-medium"
            :class="
              isActive(item.to)
                ? 'text-[var(--text)]'
                : 'text-[var(--text-muted)] hover:bg-[var(--surface-hover)] hover:text-[var(--text)]'
            "
            :style="isActive(item.to) ? { background: 'var(--surface-hover)' } : undefined"
            :title="ui.sidebarCollapsed ? item.label : undefined"
            :aria-current="isActive(item.to) ? 'page' : undefined"
          >
            <span
              class="bg-brand-500 absolute top-1/2 -left-2.5 h-[18px] w-[3px] -translate-y-1/2 rounded-r-full transition-transform duration-200 ease-[var(--ease-spring)]"
              :class="isActive(item.to) ? 'scale-y-100' : 'scale-y-0'"
            />
            <AppIcon :name="item.icon" :size="16.5" class="shrink-0" />
            <span v-if="!ui.sidebarCollapsed" class="truncate">{{ item.label }}</span>
          </RouterLink>
        </li>
      </ul>

      <button
        class="pressable mt-1 flex w-full items-center justify-center rounded-[9px] px-2.5 py-[7px] text-[var(--text-subtle)] hover:bg-[var(--surface-hover)] hover:text-[var(--text)]"
        :aria-label="ui.sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'"
        @click="ui.toggleSidebar()"
      >
        <AppIcon :name="ui.sidebarCollapsed ? 'chevronRight' : 'chevronLeft'" :size="15" />
      </button>
    </div>
  </aside>
</template>
