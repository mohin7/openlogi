<script setup lang="ts">
import { toRef } from 'vue'
import AppIcon from './AppIcon.vue'
import { useDialog } from '@/composables/useDialog'

const props = withDefaults(
  defineProps<{
    open: boolean
    title: string
    message: string
    confirmLabel?: string
    tone?: 'default' | 'danger'
    busy?: boolean
  }>(),
  { confirmLabel: 'Confirm', tone: 'default', busy: false },
)
const emit = defineEmits<{ close: []; confirm: [] }>()

useDialog(toRef(props, 'open'), () => emit('close'))
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-200 ease-[var(--ease-out-quint)]"
      enter-from-class="opacity-0"
      leave-active-class="transition duration-150"
      leave-to-class="opacity-0"
    >
      <div
        v-if="open"
        class="fixed inset-0 z-50 grid place-items-center bg-black/50 p-6"
        @click.self="emit('close')"
      >
        <div
          class="w-full max-w-sm rounded-[var(--radius-panel)] border p-5 shadow-[var(--shadow-pop)]"
          :style="{ background: 'var(--surface-raised)', borderColor: 'var(--border)' }"
          role="alertdialog"
          aria-modal="true"
          :aria-label="title"
        >
          <div class="flex items-start gap-3">
            <div
              class="grid size-9 shrink-0 place-items-center rounded-xl"
              :style="{ background: 'var(--surface-sunken)' }"
            >
              <AppIcon
                :name="tone === 'danger' ? 'alert' : 'refresh'"
                :size="17"
                :class="tone === 'danger' ? 'text-red-500' : 'text-[var(--text-muted)]'"
              />
            </div>
            <div class="min-w-0 flex-1">
              <h2 class="text-title">{{ title }}</h2>
              <p class="mt-1 text-caption leading-relaxed text-[var(--text-muted)]">
                {{ message }}
              </p>
            </div>
          </div>

          <div class="mt-5 flex justify-end gap-2">
            <button
              class="pressable rounded-[9px] px-3.5 py-2 text-label font-medium text-[var(--text-muted)] hover:bg-[var(--surface-hover)]"
              @click="emit('close')"
            >
              Cancel
            </button>
            <button
              class="rounded-[10px] px-3.5 py-2 text-label font-medium text-white transition-colors disabled:opacity-50"
              :class="tone === 'danger' ? 'bg-red-500 hover:bg-red-600' : 'bg-brand-500 hover:bg-brand-600'"
              :disabled="busy"
              @click="emit('confirm')"
            >
              {{ busy ? 'Working…' : confirmLabel }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
