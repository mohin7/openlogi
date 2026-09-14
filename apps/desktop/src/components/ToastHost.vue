<script setup lang="ts">
import { useUiStore } from '@/stores/ui'
import AppIcon from './AppIcon.vue'
const ui = useUiStore()
const icons = { default: 'info', success: 'check', warning: 'alert', danger: 'alert' } as const
</script>

<template>
  <div class="pointer-events-none fixed right-5 bottom-14 z-50 flex w-[320px] flex-col gap-2">
    <TransitionGroup
      enter-active-class="transition duration-300 ease-[var(--ease-out-quint)]"
      enter-from-class="translate-y-2 opacity-0 scale-[0.98]"
      leave-active-class="transition duration-200"
      leave-to-class="translate-x-4 opacity-0"
    >
      <div
        v-for="t in ui.toasts"
        :key="t.id"
        class="pointer-events-auto flex items-start gap-3 rounded-2xl border p-3.5 shadow-[var(--shadow-pop)]"
        :style="{ background: 'var(--surface-raised)', borderColor: 'var(--border)' }"
      >
        <AppIcon
          :name="icons[t.tone]"
          :size="16"
          class="mt-0.5 shrink-0"
          :class="{
            'text-brand-500': t.tone === 'success',
            'text-amber-500': t.tone === 'warning',
            'text-red-500': t.tone === 'danger',
            'text-[var(--text-subtle)]': t.tone === 'default',
          }"
        />
        <div class="min-w-0 flex-1">
          <p class="text-label font-medium">{{ t.title }}</p>
          <p v-if="t.detail" class="mt-0.5 text-caption text-[var(--text-muted)]">{{ t.detail }}</p>
        </div>
        <button
          class="shrink-0 rounded-md p-0.5 text-[var(--text-subtle)] transition-colors hover:text-[var(--text)]"
          aria-label="Dismiss"
          @click="ui.dismiss(t.id)"
        >
          <AppIcon name="x" :size="14" />
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>
