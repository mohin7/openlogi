<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    modelValue: number
    min: number
    max: number
    step?: number
    label?: string
    unit?: string
    ticks?: number[]
    disabled?: boolean
  }>(),
  { step: 1, disabled: false },
)
const emit = defineEmits<{ 'update:modelValue': [number] }>()

const percent = computed(
  () => ((props.modelValue - props.min) / (props.max - props.min)) * 100,
)

function onInput(e: Event) {
  emit('update:modelValue', Number((e.target as HTMLInputElement).value))
}
</script>

<template>
  <div class="w-full">
    <div v-if="label" class="mb-3 flex items-baseline justify-between">
      <label class="text-label font-medium text-[var(--text-muted)]">{{ label }}</label>
      <span class="text-title tabular-nums">
        {{ modelValue }}<span class="ml-0.5 text-[var(--text-subtle)]">{{ unit }}</span>
      </span>
    </div>

    <div class="relative h-6">
      <!-- Track -->
      <div
        class="absolute top-1/2 h-1.5 w-full -translate-y-1/2 rounded-full"
        :style="{ background: 'var(--border)' }"
      />
      <!-- Fill -->
      <div
        class="bg-brand-500 pointer-events-none absolute top-1/2 h-1.5 -translate-y-1/2 rounded-full transition-[width] duration-100"
        :style="{ width: `${percent}%` }"
      />
      <!-- Thumb: rendered separately so it can carry a shadow the native
           control cannot, while the real input stays on top for a11y. -->
      <div
        class="pointer-events-none absolute top-1/2 size-[18px] -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white bg-[var(--surface-raised)] shadow-[0_1px_4px_rgba(0,0,0,.25)] transition-transform duration-150 ease-[var(--ease-out-quint)]"
        :style="{ left: `${percent}%`, background: 'var(--color-brand-500)' }"
      />
      <input
        type="range"
        class="absolute inset-0 h-6 w-full cursor-pointer opacity-0 disabled:cursor-not-allowed"
        :min="min"
        :max="max"
        :step="step"
        :value="modelValue"
        :disabled="disabled"
        :aria-label="label"
        @input="onInput"
      />
    </div>

    <div v-if="ticks?.length" class="mt-2 flex justify-between">
      <button
        v-for="t in ticks"
        :key="t"
        class="rounded-md px-1.5 py-0.5 text-[11px] font-medium tabular-nums text-[var(--text-subtle)] transition-colors hover:text-[var(--text)]"
        :class="modelValue === t ? 'text-brand-600 dark:text-brand-400' : ''"
        @click="emit('update:modelValue', t)"
      >
        {{ t }}
      </button>
    </div>
  </div>
</template>
