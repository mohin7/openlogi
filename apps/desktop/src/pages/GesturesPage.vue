<script setup lang="ts">
import { computed, ref } from 'vue'
import BaseCard from '@/components/BaseCard.vue'
import AppIcon from '@/components/AppIcon.vue'
import SliderControl from '@/components/SliderControl.vue'
import { useDeviceStore } from '@/stores/devices'

const store = useDeviceStore()
const sensitivity = ref(50)

/** Gestures require a control that streams raw XY (0x1B04 RAW_XY flag). */
const gestureControls = computed(
  () => store.selected?.controls.filter((c) => c.supportsGestures) ?? [],
)
const supported = computed(() => gestureControls.value.length > 0)

const directions = [
  { id: 'up', label: 'Up', path: 'M12 19V5m0 0-5 5m5-5 5 5' },
  { id: 'down', label: 'Down', path: 'M12 5v14m0 0 5-5m-5 5-5-5' },
  { id: 'left', label: 'Left', path: 'M19 12H5m0 0 5-5m-5 5 5 5' },
  { id: 'right', label: 'Right', path: 'M5 12h14m0 0-5-5m5 5-5 5' },
]
const assignments = ref<Record<string, string>>({
  up: 'Show all windows',
  down: 'Minimise window',
  left: 'Workspace left',
  right: 'Workspace right',
})
</script>

<template>
  <div class="mx-auto max-w-5xl px-7 py-7">
    <header class="mb-8">
      <h1 class="text-display">Gestures</h1>
      <p class="mt-1 text-[14px] text-[var(--text-muted)]">
        Hold a button and move the mouse to trigger an action.
      </p>
    </header>

    <BaseCard v-if="!supported">
      <div class="flex items-start gap-3">
        <AppIcon name="alert" :size="17" class="mt-0.5 shrink-0 text-amber-500" />
        <div>
          <p class="text-label font-semibold">Not available on this device</p>
          <p class="mt-1 text-label text-[var(--text-muted)]">
            No control reports raw XY streaming, which gesture recognition requires.
          </p>
        </div>
      </div>
    </BaseCard>

    <template v-else>
      <BaseCard class="mb-6">
        <div class="flex items-start gap-3">
          <AppIcon name="check" :size="17" class="text-brand-500 mt-0.5 shrink-0" />
          <div>
            <p class="text-label font-semibold">Supported on {{ store.selected?.name }}</p>
            <p class="mt-1 text-caption leading-relaxed text-[var(--text-muted)]">
              {{ gestureControls.length }} control{{ gestureControls.length === 1 ? '' : 's' }}
              report raw XY streaming — including the virtual gesture control
              <code class="text-caption">0x00D7</code>, which the firmware provides specifically for
              this. Recognition itself runs in OpenLogi, since the device has no gesture engine.
            </p>
          </div>
        </div>
      </BaseCard>

      <div class="grid gap-6 lg:grid-cols-[1fr_320px]">
        <BaseCard>
          <h2 class="mb-4 text-title">Directions</h2>
          <div class="grid gap-3 sm:grid-cols-2">
            <div
              v-for="d in directions"
              :key="d.id"
              class="flex items-center gap-3 rounded-[10px] border p-3"
              :style="{ borderColor: 'var(--border)' }"
            >
              <div
                class="grid size-9 shrink-0 place-items-center rounded-lg"
                :style="{ background: 'var(--surface-sunken)' }"
              >
                <svg
                  width="17"
                  height="17"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.75"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <path :d="d.path" />
                </svg>
              </div>
              <div class="min-w-0">
                <p class="text-label font-medium">{{ d.label }}</p>
                <p class="truncate text-caption text-[var(--text-muted)]">
                  {{ assignments[d.id] }}
                </p>
              </div>
            </div>
          </div>
        </BaseCard>

        <BaseCard class="self-start">
          <h2 class="mb-1 text-title">Sensitivity</h2>
          <p class="mb-5 text-label text-[var(--text-muted)]">
            How far the pointer must travel before a direction is committed.
          </p>
          <SliderControl v-model="sensitivity" :min="10" :max="100" :step="5" label="Threshold" />
        </BaseCard>
      </div>
    </template>
  </div>
</template>
