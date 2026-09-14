<script setup lang="ts">
import { computed } from 'vue'
import AppIcon from './AppIcon.vue'
import type { Battery } from '@/types/device'

const props = withDefaults(
  defineProps<{ battery: Battery; variant?: 'inline' | 'ring'; showLabel?: boolean }>(),
  { variant: 'inline', showLabel: true },
)

const pct = computed(() => props.battery.percentage ?? 0)
const charging = computed(
  () => props.battery.state === 'charging' || props.battery.state === 'chargingSlow',
)
const unknown = computed(() => props.battery.percentage === null)

/**
 * Colour encodes urgency only. A healthy battery stays neutral so it does not
 * compete for attention; amber and red mean the user has something to do.
 * The state is never conveyed by colour alone — the percentage is always
 * present, and charging adds an icon.
 */
const tone = computed(() => {
  if (charging.value) return { text: 'text-brand-600 dark:text-brand-400', stroke: 'var(--color-brand-500)' }
  if (pct.value <= 15) return { text: 'text-red-600 dark:text-red-400', stroke: '#ef4444' }
  if (pct.value <= 30) return { text: 'text-amber-600 dark:text-amber-400', stroke: '#f59e0b' }
  return { text: 'text-[var(--text)]', stroke: 'var(--text)' }
})

const R = 15.5
const CIRC = 2 * Math.PI * R
const dash = computed(() => `${(pct.value / 100) * CIRC} ${CIRC}`)

const stateLabel = computed(() => {
  if (unknown.value) return 'Battery level unknown'
  if (charging.value) return `Battery ${pct.value} percent, charging`
  return `Battery ${pct.value} percent`
})
</script>

<template>
  <!-- Ring: the hero reading. A radial gauge reads as a status at a glance in
       a way a small bar cannot, and it scales without becoming a stripe. -->
  <div v-if="variant === 'ring'" class="flex items-center gap-3" :aria-label="stateLabel" role="img">
    <div class="relative size-[38px] shrink-0">
      <svg viewBox="0 0 38 38" class="size-full -rotate-90">
        <circle cx="19" cy="19" :r="R" fill="none" stroke="var(--border)" stroke-width="3" />
        <circle
          cx="19"
          cy="19"
          :r="R"
          fill="none"
          :stroke="tone.stroke"
          stroke-width="3"
          stroke-linecap="round"
          :stroke-dasharray="dash"
          class="transition-[stroke-dasharray] duration-700 ease-[var(--ease-out-quint)]"
        />
      </svg>
      <AppIcon
        v-if="charging"
        name="zap"
        :size="14"
        class="text-brand-500 absolute inset-0 m-auto"
      />
    </div>
    <div v-if="showLabel" class="min-w-0">
      <p class="text-title tabular-nums" :class="tone.text">
        {{ unknown ? '—' : `${pct}%` }}
      </p>
      <p class="text-caption text-[var(--text-muted)]">
        {{ charging ? 'Charging' : unknown ? 'Unknown' : 'Battery' }}
      </p>
    </div>
  </div>

  <!-- Inline: list rows and cards. -->
  <div v-else class="flex items-center gap-2" :aria-label="stateLabel" role="img">
    <div
      class="relative h-[13px] w-[24px] shrink-0 rounded-[4px] border-[1.5px]"
      :class="tone.text"
      style="border-color: currentColor"
    >
      <div class="absolute inset-[1.5px] overflow-hidden rounded-[2px]">
        <div
          class="h-full rounded-[2px] transition-[width] duration-700 ease-[var(--ease-out-quint)]"
          :style="{ width: `${Math.max(pct, 4)}%`, background: tone.stroke }"
        />
      </div>
      <span
        class="absolute top-1/2 -right-[2.5px] h-[5px] w-[2px] -translate-y-1/2 rounded-r-[1px] bg-current"
      />
    </div>
    <span v-if="showLabel" class="text-label font-medium tabular-nums" :class="tone.text">
      {{ unknown ? '—' : `${pct}%` }}
    </span>
    <AppIcon v-if="charging" name="zap" :size="12" class="text-brand-500 -ml-1" />
  </div>
</template>
