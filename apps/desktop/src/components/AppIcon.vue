<script setup lang="ts">
/**
 * Hand-rolled icon set.
 *
 * An icon library would add a dependency and a bundle for the ~20 glyphs this
 * app uses, so the paths live here. All are drawn on a 24-unit grid with a
 * 1.75 stroke so they optically match Inter's weight.
 */
const props = withDefaults(defineProps<{ name: string; size?: number | string }>(), { size: 18 })

const paths: Record<string, string> = {
  mouse: 'M12 3a6 6 0 0 0-6 6v6a6 6 0 0 0 12 0V9a6 6 0 0 0-6-6Zm0 0v5',
  keyboard: 'M3 6h18v12H3zM7 10h.01M11 10h.01M15 10h.01M7 14h10',
  layers: 'm12 3 9 5-9 5-9-5 9-5Zm9 11-9 5-9-5m18-3-9 5-9-5',
  gesture: 'M9 11V5.5a1.5 1.5 0 0 1 3 0V11m0-1.5a1.5 1.5 0 0 1 3 0V12m0-1a1.5 1.5 0 0 1 3 0v4a6 6 0 0 1-6 6h-1a7 7 0 0 1-7-7v-4a1.5 1.5 0 0 1 3 0V12',
  command: 'M9 6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3Z',
  settings: 'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Zm7.4-3a7.4 7.4 0 0 0-.1-1.2l2-1.5-2-3.4-2.3 1a7.6 7.6 0 0 0-2-1.2L14.6 3H9.4L9 5.7a7.6 7.6 0 0 0-2 1.2l-2.3-1-2 3.4 2 1.5a7.4 7.4 0 0 0 0 2.4l-2 1.5 2 3.4 2.3-1a7.6 7.6 0 0 0 2 1.2l.4 2.7h5.2l.4-2.7a7.6 7.6 0 0 0 2-1.2l2.3 1 2-3.4-2-1.5c.06-.4.1-.8.1-1.2Z',
  info: 'M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Zm0-9v4m0-8h.01',
  search: 'M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16Zm10 2-4.35-4.35',
  sun: 'M12 17a5 5 0 1 0 0-10 5 5 0 0 0 0 10Zm0-14v2m0 14v2M5.6 5.6l1.4 1.4m10 10 1.4 1.4M3 12h2m14 0h2M5.6 18.4 7 17m10-10 1.4-1.4',
  moon: 'M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8Z',
  monitor: 'M3 5h18v11H3zM8 20h8m-4-4v4',
  chevronRight: 'm9 6 6 6-6 6',
  chevronDown: 'm6 9 6 6 6-6',
  chevronLeft: 'm15 6-6 6 6 6',
  battery: 'M3 8h14v8H3zM20 11v2',
  zap: 'M13 2 4 14h7l-1 8 9-12h-7l1-8Z',
  check: 'm4 12 5 5L20 6',
  x: 'M6 6 18 18M18 6 6 18',
  plus: 'M12 5v14M5 12h14',
  lock: 'M6 11h12v9H6zM9 11V8a3 3 0 0 1 6 0v3',
  refresh: 'M21 12a9 9 0 1 1-3-6.7M21 4v5h-5',
  activity: 'M3 12h4l3 8 4-16 3 8h4',
  usb: 'M12 21V5m0 0 3 3m-3-3L9 8m3 5 4-3m-4 6-4-3',
  bluetooth: 'm7 7 10 10-5 4V3l5 4L7 17',
  alert: 'M12 9v4m0 4h.01M10.3 3.9 2.5 17a2 2 0 0 0 1.7 3h15.6a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z',
  sparkle: 'M12 3v4m0 10v4M3 12h4m10 0h4M6 6l2.5 2.5M15.5 15.5 18 18M18 6l-2.5 2.5M8.5 15.5 6 18',
}
</script>

<template>
  <svg
    :width="props.size"
    :height="props.size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.75"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    focusable="false"
  >
    <path :d="paths[props.name] ?? paths.info" />
  </svg>
</template>
