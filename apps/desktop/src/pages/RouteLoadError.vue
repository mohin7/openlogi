<script setup lang="ts">
import BaseCard from '@/components/BaseCard.vue'
import AppIcon from '@/components/AppIcon.vue'

defineProps<{ page: string; reason: string }>()
</script>

<template>
  <div class="mx-auto max-w-2xl px-7 py-7">
    <BaseCard class="border-red-400/40">
      <div class="flex items-start gap-3">
        <AppIcon name="alert" :size="18" class="mt-0.5 shrink-0 text-red-500" />
        <div class="min-w-0 flex-1">
          <h1 class="text-title">Couldn't load “{{ page }}”</h1>
          <p class="mt-1.5 text-label leading-relaxed text-[var(--text-muted)]">
            This page is loaded on demand and the request failed. In development that usually
            means the Vite dev server is no longer running; in a packaged build it means an
            application file is missing or was replaced by an update.
          </p>
          <p class="mt-2 font-mono text-caption break-words text-[var(--text-subtle)]">
            {{ reason }}
          </p>
          <button
            class="bg-brand-500 hover:bg-brand-600 mt-4 rounded-[10px] px-3.5 py-2 text-label font-medium text-white transition-colors"
            @click="$router.go(0)"
          >
            Reload
          </button>
        </div>
      </div>
    </BaseCard>
  </div>
</template>
