<script setup lang="ts">
import BaseCard from '@/components/BaseCard.vue'
import AppIcon from '@/components/AppIcon.vue'
import { useDeviceStore } from '@/stores/devices'
import { useUiStore } from '@/stores/ui'

const store = useDeviceStore()
const ui = useUiStore()
</script>

<template>
  <div class="mx-auto max-w-5xl px-7 py-7">
    <header class="mb-8 flex items-start justify-between gap-4">
      <div>
        <h1 class="text-display">Profiles</h1>
        <p class="mt-1 text-[14px] text-[var(--text-muted)]">
          Different button actions per application, switched automatically.
        </p>
      </div>
      <button
        class="bg-brand-500 hover:bg-brand-600 flex items-center gap-1.5 rounded-[10px] px-3.5 py-2 text-label font-medium text-white transition-colors"
        @click="ui.notify('Not yet implemented', 'Profile creation lands with the daemon bridge.')"
      >
        <AppIcon name="plus" :size="15" />
        New profile
      </button>
    </header>

    <div class="grid gap-4 sm:grid-cols-2">
      <BaseCard v-for="p in store.profiles" :key="p.id" interactive>
        <div class="flex items-start gap-3">
          <span class="mt-1.5 size-2.5 shrink-0 rounded-full" :style="{ background: p.color }" />
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <h3 class="truncate text-title">{{ p.name }}</h3>
              <span
                v-if="p.isDefault"
                class="rounded px-1.5 py-px text-[10px] font-semibold tracking-wide text-[var(--text-subtle)] uppercase"
                :style="{ background: 'var(--surface-sunken)' }"
              >
                Default
              </span>
            </div>
            <p class="mt-0.5 text-caption text-[var(--text-muted)]">
              {{ p.appMatch ? `Activates for “${p.appMatch}”` : 'Fallback for every application' }}
            </p>

            <ul class="mt-3 space-y-1">
              <li
                v-for="m in p.mappings.filter((x) => x.action.kind !== 'default')"
                :key="m.cid"
                class="flex items-center justify-between gap-3 text-caption"
              >
                <span class="text-[var(--text-muted)]">
                  {{ store.selected?.controls.find((c) => c.cid === m.cid)?.name ?? 'Button' }}
                </span>
                <span class="truncate font-medium">{{ m.action.label }}</span>
              </li>
              <li
                v-if="!p.mappings.some((x) => x.action.kind !== 'default')"
                class="text-caption text-[var(--text-subtle)]"
              >
                No overrides — everything behaves as the device default.
              </li>
            </ul>
          </div>
        </div>
      </BaseCard>
    </div>

    <BaseCard class="mt-6">
      <div class="flex items-start gap-3">
        <AppIcon name="alert" :size="17" class="mt-0.5 shrink-0 text-amber-500" />
        <div>
          <p class="text-label font-semibold">Automatic switching needs a compositor query</p>
          <p class="mt-1 text-caption leading-relaxed text-[var(--text-muted)]">
            Detecting the focused application is straightforward on X11 via
            <code class="text-caption">_NET_ACTIVE_WINDOW</code>. Wayland has no portable equivalent,
            so it needs a per-compositor path. Your session is X11, so this will work here — but
            it is not universal, and OpenLogi will say so rather than failing quietly.
          </p>
        </div>
      </div>
    </BaseCard>
  </div>
</template>
