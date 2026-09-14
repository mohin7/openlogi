<script setup lang="ts">
const props = defineProps<{
  modelValue: boolean
  label: string
  description?: string
  disabled?: boolean
}>()
const emit = defineEmits<{ 'update:modelValue': [boolean] }>()
</script>

<template>
  <div class="flex items-start justify-between gap-6 py-3.5">
    <div class="min-w-0">
      <p class="text-label font-semibold">{{ label }}</p>
      <p v-if="description" class="mt-0.5 text-caption leading-relaxed text-[var(--text-muted)]">
        {{ description }}
      </p>
    </div>
    <button
      type="button"
      role="switch"
      :aria-checked="modelValue"
      :aria-label="label"
      :disabled="disabled"
      class="relative mt-0.5 h-[26px] w-[44px] shrink-0 rounded-full transition-colors duration-200 ease-[var(--ease-out-quint)] disabled:opacity-40"
      :style="{ background: modelValue ? 'var(--color-brand-500)' : 'var(--border-strong)' }"
      @click="!props.disabled && emit('update:modelValue', !modelValue)"
    >
      <span
        class="absolute top-[3px] size-5 rounded-full bg-white shadow-sm transition-[left] duration-200 ease-[var(--ease-out-quint)]"
        :style="{ left: modelValue ? '21px' : '3px' }"
      />
    </button>
  </div>
</template>
