import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import AppShell from '@/layouts/AppShell.vue'

/**
 * Every route must mount, render and be navigable away from.
 *
 * This is the cheapest guard against the class of bug that presents as "the
 * app freezes when I click X": a reactive write during render loops forever,
 * which a human sees as a hang and a test sees as a timeout.
 */

// jsdom has no matchMedia; the UI store reads it to resolve the system theme.
beforeEach(() => {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: vi.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  })
  localStorage.clear()
  setActivePinia(createPinia())
})

const routes = [
  { path: '/', name: 'devices', component: () => import('@/pages/DashboardPage.vue') },
  { path: '/device/:id', name: 'device', component: () => import('@/pages/DeviceDetailPage.vue') },
  { path: '/profiles', name: 'profiles', component: () => import('@/pages/ProfilesPage.vue') },
  { path: '/gestures', name: 'gestures', component: () => import('@/pages/GesturesPage.vue') },
  { path: '/macros', name: 'macros', component: () => import('@/pages/MacrosPage.vue') },
  { path: '/settings', name: 'settings', component: () => import('@/pages/SettingsPage.vue') },
  { path: '/about', name: 'about', component: () => import('@/pages/AboutPage.vue') },
]

function makeRouter() {
  return createRouter({ history: createMemoryHistory(), routes })
}

async function mountShell() {
  const router = makeRouter()
  const pinia = createPinia()
  setActivePinia(pinia)
  router.push('/')
  await router.isReady()
  const wrapper = mount(AppShell, { global: { plugins: [router, pinia] } })
  await flushPromises()
  return { wrapper, router }
}

describe('routes', () => {
  for (const r of routes) {
    it(`renders ${r.path} without hanging`, async () => {
      const { wrapper, router } = await mountShell()
      await router.push(r.path === '/device/:id' ? '/device/481871d4' : r.path)
      await flushPromises()
      expect(wrapper.html().length).toBeGreaterThan(0)
      wrapper.unmount()
    })
  }

  it('survives navigating through every route in sequence', async () => {
    const { wrapper, router } = await mountShell()
    for (const r of routes) {
      await router.push(r.path === '/device/:id' ? '/device/481871d4' : r.path)
      await flushPromises()
    }
    // Back to the start — the common real-world path that triggers loops.
    await router.push('/')
    await flushPromises()
    expect(wrapper.html()).toContain('OpenLogi')
    wrapper.unmount()
  })
})
