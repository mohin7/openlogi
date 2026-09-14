<script setup lang="ts">
withDefaults(
  defineProps<{ padded?: boolean; interactive?: boolean; elevation?: 1 | 2 | 3 }>(),
  { padded: true, interactive: false, elevation: 1 },
)
</script>

<template>
  <div
    class="rounded-[var(--radius-card)] border"
    :class="[
      padded ? 'p-4' : '',
      interactive
        ? 'pressable cursor-pointer hover:-translate-y-px hover:shadow-[var(--elev-3)] hover:border-[var(--border-strong)]'
        : '',
    ]"
    :style="{
      background: 'var(--surface-raised)',
      borderColor: 'var(--border)',
      boxShadow: elevation === 3 ? 'var(--elev-3)' : elevation === 2 ? 'var(--elev-2)' : 'var(--elev-1)',
    }"
  >
    <slot />
  </div>
</template>
