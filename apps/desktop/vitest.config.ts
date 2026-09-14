import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    // A render loop shows up as a hang, not a failure. Cap it so the suite
    // reports the offending test instead of blocking forever.
    testTimeout: 8000,
    hookTimeout: 8000,
  },
})
