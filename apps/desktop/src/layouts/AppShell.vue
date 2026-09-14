<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import AppSidebar from './AppSidebar.vue'
import AppIcon from '@/components/AppIcon.vue'
import ToastHost from '@/components/ToastHost.vue'
import { useUiStore } from '@/stores/ui'
import { useDeviceStore } from '@/stores/devices'
import { isTauri } from '@/services/backend'

const ui = useUiStore()
const devices = useDeviceStore()
const route = useRoute()

const themeIcon = computed(
  () => ({ light: 'sun', dark: 'moon', system: 'monitor' })[ui.theme],
)

async function rescan() {
  await devices.load()
  ui.notify(
    'Rescan complete',
    `${devices.connectedCount} device${devices.connectedCount === 1 ? '' : 's'} found`,
    'success',
  )
}

function cycleTheme() {
  const order = ['light', 'dark', 'system'] as const
  ui.setTheme(order[(order.indexOf(ui.theme) + 1) % order.length])
}
</script>

<template>
  <div class="flex h-full" :style="{ background: 'var(--surface-sunken)' }">
    <AppSidebar />

    <div class="flex min-w-0 flex-1 flex-col">
      <!-- Top bar -->
      <header
        class="flex h-14 shrink-0 items-center gap-3 border-b px-4"
        :style="{ borderColor: 'var(--border)', background: 'var(--surface)' }"
      >
        <div class="relative w-full max-w-[340px]">
          <AppIcon
            name="search"
            :size="14.5"
            class="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-[var(--text-subtle)]"
          />
          <input
            v-model="ui.search"
            type="search"
            placeholder="Search devices and actions"
            aria-label="Search"
            class="h-8 w-full rounded-[9px] border pr-12 pl-8 text-label outline-none transition-colors duration-150 placeholder:text-[var(--text-subtle)] focus:border-[var(--color-brand-500)]"
            :style="{ background: 'var(--surface-sunken)', borderColor: 'var(--border)' }"
          />
          <kbd
            class="pointer-events-none absolute top-1/2 right-2 -translate-y-1/2 rounded border px-1.5 py-px text-[10.5px] font-medium text-[var(--text-subtle)]"
            :style="{ borderColor: 'var(--border)', background: 'var(--surface)' }"
          >
            /
          </kbd>
        </div>

        <div class="ml-auto flex items-center gap-0.5">
          <button
            class="pressable grid size-8 place-items-center rounded-[9px] text-[var(--text-muted)] hover:bg-[var(--surface-hover)] hover:text-[var(--text)]"
            :aria-label="`Theme: ${ui.theme}. Click to change.`"
            :title="`Theme: ${ui.theme}`"
            @click="cycleTheme"
          >
            <AppIcon :name="themeIcon" :size="16.5" />
          </button>
          <button
            class="pressable grid size-8 place-items-center rounded-[9px] text-[var(--text-muted)] hover:bg-[var(--surface-hover)] hover:text-[var(--text)] disabled:opacity-50"
            aria-label="Rescan for devices"
            title="Rescan for devices"
            :disabled="devices.loading"
            @click="rescan"
          >
            <AppIcon
              name="refresh"
              :size="16.5"
              :class="devices.loading ? 'animate-spin' : ''"
            />
          </button>
        </div>
      </header>

      <!-- Page -->
      <main class="min-h-0 flex-1 overflow-y-auto">
        <RouterView v-slot="{ Component }">
          <!-- Enter-only, and deliberately no `mode="out-in"`.
               out-in defers mounting the next page until the previous one
               signals its leave finished. Any page whose leave cannot complete
               — a fragment root, a transition on a detached element, a
               cancelled animation — then blocks every subsequent navigation
               and the content area stays blank. A fade-in is not worth a class
               of bug that makes the app unusable, so the outgoing page is
               removed immediately and only the incoming one animates. -->
          <Transition
            enter-active-class="transition duration-200 ease-[var(--ease-out-quint)]"
            enter-from-class="opacity-0 translate-y-1"
          >
            <component :is="Component" :key="route.path" />
          </Transition>
        </RouterView>
      </main>

      <!-- Status bar -->
      <footer
        class="flex h-7 shrink-0 items-center gap-3 border-t px-4 text-caption text-[var(--text-subtle)]"
        :style="{ borderColor: 'var(--border)', background: 'var(--surface)' }"
      >
        <span class="flex items-center gap-1.5">
          <span
            class="size-1.5 rounded-full"
            :class="devices.connectedCount ? 'bg-brand-500' : 'bg-[var(--border-strong)]'"
          />
          {{ devices.connectedCount }} device{{ devices.connectedCount === 1 ? '' : 's' }}
        </span>
        <span v-if="devices.selected" class="hidden sm:inline">{{ devices.selected.protocol }}</span>
        <span v-if="devices.activeProfile" class="hidden md:inline">
          {{ devices.activeProfile.name }}
        </span>
        <span class="ml-auto flex items-center gap-1.5">
          <template v-if="isTauri">
            <span class="bg-brand-500 size-1.5 rounded-full" />
            Connected
          </template>
          <template v-else>
            <AppIcon name="alert" :size="12" class="text-amber-500" />
            Preview — captured hardware data
          </template>
        </span>
      </footer>
    </div>

    <ToastHost />
  </div>
</template>
