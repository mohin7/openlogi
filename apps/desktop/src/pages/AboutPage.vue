<script setup lang="ts">
import BaseCard from '@/components/BaseCard.vue'
import AppIcon from '@/components/AppIcon.vue'
</script>

<template>
  <div class="mx-auto max-w-2xl px-7 py-7">
    <div class="mb-8 flex items-center gap-4">
      <div class="bg-brand-500 grid size-12 place-items-center rounded-2xl text-white">
        <AppIcon name="mouse" :size="24" />
      </div>
      <div>
        <h1 class="text-display">OpenLogi</h1>
        <p class="text-label text-[var(--text-muted)]">Version 0.1.0 · GPL-3.0-or-later</p>
      </div>
    </div>

    <BaseCard class="mb-4">
      <p class="text-label leading-relaxed text-[var(--text-muted)]">
        A native Linux alternative to Logitech Options+. Speaks the HID++ protocol directly over
        <code class="text-caption">hidraw</code>, with no proprietary software and no root
        privileges after installation.
      </p>
    </BaseCard>

    <BaseCard>
      <h2 class="mb-3 text-label font-semibold">Permissions</h2>
      <ul class="space-y-2.5 text-label">
        <li class="flex items-start gap-2.5">
          <AppIcon name="check" :size="15" class="text-brand-500 mt-px shrink-0" />
          <span>
            <span class="font-medium">hidraw access</span>
            <span class="block text-caption text-[var(--text-muted)]">
              Granted per-session by udev <code class="text-caption">uaccess</code>, revoked at logout.
            </span>
          </span>
        </li>
        <li class="flex items-start gap-2.5">
          <AppIcon name="check" :size="15" class="text-brand-500 mt-px shrink-0" />
          <span>
            <span class="font-medium">uinput write</span>
            <span class="block text-caption text-[var(--text-muted)]">
              Via a dedicated <code class="text-caption">openlogi</code> group — not the
              <code class="text-caption">input</code> group, which would allow reading every
              keystroke on the system.
            </span>
          </span>
        </li>
      </ul>
    </BaseCard>
  </div>
</template>
