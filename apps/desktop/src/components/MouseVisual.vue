<script setup lang="ts">
import { computed } from 'vue'
import type { Control } from '@/types/device'

const props = defineProps<{
  controls: Control[]
  selectedCid: number | null
  /** CIDs that currently have a non-default action, for the assigned dot. */
  assigned?: number[]
}>()
const emit = defineEmits<{ select: [cid: number] }>()

/** Hit regions, drawn to match the Signature M650's actual button layout. */
const regions: Record<number, string> = {
  0x0050: 'M100 10 C56 10 30 58 26 132 L96 132 L96 10 Z',
  0x0051: 'M100 10 C144 10 170 58 174 132 L104 132 L104 10 Z',
  0x0052: 'M92 52 h16 a8 8 0 0 1 8 8 v34 a16 16 0 0 1 -32 0 v-34 a8 8 0 0 1 8 -8 Z',
  0x0056: 'M27 142 h20 a6 6 0 0 1 6 6 v22 a6 6 0 0 1 -6 6 h-22 Z',
  0x0053: 'M25 180 h22 a6 6 0 0 1 6 6 v22 a6 6 0 0 1 -6 6 h-24 Z',
}

const byCid = computed(() => new Map(props.controls.map((c) => [c.cid, c])))

function state(cid: number) {
  const c = byCid.value.get(cid)
  if (!c) return 'absent'
  if (props.selectedCid === cid) return 'selected'
  if (!c.reprogrammable && !c.divertable) return 'locked'
  return 'idle'
}

const fills: Record<string, string> = {
  selected: 'var(--color-brand-500)',
  locked: 'var(--border)',
  idle: 'var(--surface-sunken)',
  absent: 'transparent',
}
</script>

<template>
  <div class="relative mx-auto w-full max-w-[260px]">
    <svg viewBox="0 0 200 330" class="w-full" role="img" aria-label="Signature M650 buttons">
      <!-- Body -->
      <path
        d="M100 8 C150 8 176 60 176 140 C176 252 148 314 100 314 C52 314 24 252 24 140 C24 60 50 8 100 8 Z"
        :fill="'var(--surface-raised)'"
        :stroke="'var(--border-strong)'"
        stroke-width="2"
      />

      <!-- Clickable regions -->
      <g>
        <path
          v-for="(d, cid) in regions"
          :key="cid"
          :d="d"
          :fill="fills[state(Number(cid))]"
          :stroke="state(Number(cid)) === 'selected' ? 'var(--color-brand-600)' : 'var(--border-strong)'"
          stroke-width="1.5"
          :class="[
            'transition-all duration-200 ease-[var(--ease-out-quint)]',
            byCid.has(Number(cid)) && (byCid.get(Number(cid))!.reprogrammable || byCid.get(Number(cid))!.divertable)
              ? 'cursor-pointer hover:brightness-95'
              : 'cursor-not-allowed',
          ]"
          :tabindex="byCid.has(Number(cid)) ? 0 : -1"
          role="button"
          :aria-label="byCid.get(Number(cid))?.name"
          :aria-pressed="selectedCid === Number(cid)"
          @click="emit('select', Number(cid))"
          @keydown.enter.prevent="emit('select', Number(cid))"
          @keydown.space.prevent="emit('select', Number(cid))"
        />
      </g>

      <!-- Assigned markers -->
      <g>
        <circle
          v-for="cid in assigned ?? []"
          :key="`dot-${cid}`"
          :cx="cid === 0x0050 ? 61 : cid === 0x0051 ? 139 : cid === 0x0052 ? 100 : 40"
          :cy="cid === 0x0056 ? 159 : cid === 0x0053 ? 197 : cid === 0x0052 ? 118 : 108"
          r="3.5"
          fill="var(--color-brand-500)"
          class="animate-pulse"
        />
      </g>

      <!-- Wheel detail -->
      <rect x="94" y="56" width="12" height="42" rx="6" fill="var(--border-strong)" opacity="0.5" />
    </svg>
  </div>
</template>
