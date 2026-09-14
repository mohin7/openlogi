<script setup lang="ts">
import BaseCard from '@/components/BaseCard.vue'
import AppIcon from '@/components/AppIcon.vue'
</script>

<template>
  <div class="mx-auto max-w-5xl px-7 py-7">
    <header class="mb-8">
      <h1 class="text-display">Macros</h1>
      <p class="mt-1 text-[14px] text-[var(--text-muted)]">
        Record a sequence of keystrokes and replay it from a button.
      </p>
    </header>

    <BaseCard class="py-16 text-center">
      <div
        class="mx-auto grid size-12 place-items-center rounded-2xl"
        :style="{ background: 'var(--surface-sunken)' }"
      >
        <AppIcon name="command" :size="22" class="text-[var(--text-subtle)]" />
      </div>
      <h3 class="mt-4 text-title">Not built yet</h3>
      <p class="mx-auto mt-1.5 max-w-md text-label leading-relaxed text-[var(--text-muted)]">
        The macro engine depends on the button-diversion pipeline, which is proven on hardware but
        not yet wired to input synthesis. This screen stays empty rather than showing a recorder
        that cannot record.
      </p>
    </BaseCard>
  </div>
</template>
