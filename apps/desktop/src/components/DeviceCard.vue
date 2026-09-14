<script setup lang="ts">
import { computed } from 'vue'
import AppIcon from './AppIcon.vue'
import BatteryIndicator from './BatteryIndicator.vue'
import type { Device } from '@/types/device'

const props = defineProps<{ device: Device; active?: boolean }>()
defineEmits<{ select: [] }>()

const connection = computed(() => {
  const map = {
    bolt: { icon: 'zap', label: 'Logi Bolt' },
    unifying: { icon: 'zap', label: 'Unifying' },
    bluetooth: { icon: 'bluetooth', label: 'Bluetooth' },
    usb: { icon: 'usb', label: 'USB' },
  } as const
  return map[props.device.connection]
})
</script>

<template>
  <button
    type="button"
    class="pressable group w-full rounded-[var(--radius-card)] border p-4 text-left hover:-translate-y-px hover:border-[var(--border-strong)] hover:shadow-[var(--elev-3)]"
    :style="{
      background: 'var(--surface-raised)',
      borderColor: active ? 'var(--color-brand-500)' : 'var(--border)',
      boxShadow: active ? 'var(--elev-2)' : 'var(--elev-1)',
    }"
    @click="$emit('select')"
  >
    <div class="flex items-start gap-3.5">
      <!-- Device mark. The subtle tint keeps it from reading as another button. -->
      <div
        class="grid size-10 shrink-0 place-items-center rounded-[10px] transition-colors duration-200"
        :style="{ background: 'var(--surface-sunken)', color: 'var(--text-muted)' }"
      >
        <AppIcon :name="device.kind === 'keyboard' ? 'keyboard' : 'mouse'" :size="19" />
      </div>

      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
          <h3 class="text-title truncate">{{ device.name }}</h3>
          <span
            class="size-1.5 shrink-0 rounded-full"
            :class="device.connected ? 'bg-brand-500' : 'bg-[var(--border-strong)]'"
          />
        </div>
        <p class="mt-0.5 flex items-center gap-1.5 text-caption text-[var(--text-muted)]">
          <AppIcon :name="connection.icon" :size="12.5" />
          {{ connection.label }}
          <span class="text-[var(--text-subtle)]">·</span>
          {{ device.currentDpi }} dpi
        </p>
      </div>

      <AppIcon
        name="chevronRight"
        :size="16"
        class="mt-2 shrink-0 text-[var(--text-subtle)] transition-transform duration-200 ease-[var(--ease-out-quint)] group-hover:translate-x-0.5"
      />
    </div>

    <div
      class="mt-3.5 flex items-center justify-between border-t pt-3.5"
      :style="{ borderColor: 'var(--border)' }"
    >
      <BatteryIndicator :battery="device.battery" />
      <span class="text-caption text-[var(--text-subtle)]">{{ device.protocol }}</span>
    </div>
  </button>
</template>
