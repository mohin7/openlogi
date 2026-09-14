import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import AppShell from '@/layouts/AppShell.vue'

/**
 * Regression: visiting the device page and then using the sidebar left the
 * content area blank.
 *
 * `AppShell` wraps the router view in <Transition mode="out-in">, which
 * requires a single root element. DeviceDetailPage rendered a fragment (page +
 * two teleported dialogs), so the transition could not track the leave, never
 * signalled completion, and the next page never mounted.
 */
beforeEach(() => {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: vi.fn().mockImplementation((q: string) => ({
      matches: false, media: q,
      addEventListener: vi.fn(), removeEventListener: vi.fn(),
      addListener: vi.fn(), removeListener: vi.fn(), dispatchEvent: vi.fn(),
    })),
  })
  localStorage.clear()
  setActivePinia(createPinia())
})

const routes = [
  { path: '/', component: () => import('@/pages/DashboardPage.vue') },
  { path: '/device/:id', component: () => import('@/pages/DeviceDetailPage.vue') },
  { path: '/profiles', component: () => import('@/pages/ProfilesPage.vue') },
  { path: '/settings', component: () => import('@/pages/SettingsPage.vue') },
  { path: '/about', component: () => import('@/pages/AboutPage.vue') },
]

async function shell() {
  const router = createRouter({ history: createMemoryHistory(), routes })
  const pinia = createPinia()
  setActivePinia(pinia)
  router.push('/')
  await router.isReady()
  const wrapper = mount(AppShell, { global: { plugins: [router, pinia] } })
  await flushPromises()
  return { wrapper, router }
}

describe('navigating away from the device page', () => {
  it('renders the next page content', async () => {
    const { wrapper, router } = await shell()

    await router.push('/device/481871d4')
    await flushPromises()

    await router.push('/profiles')
    await flushPromises()
    expect(wrapper.text(), 'Profiles content should render').toContain(
      'Different button actions per application',
    )

    await router.push('/about')
    await flushPromises()
    expect(wrapper.text(), 'About content should render').toContain('GPL-3.0')

    wrapper.unmount()
  })

  it('mounts the device page inside <Transition> without a fragment warning', async () => {
    // jsdom runs no real CSS transitions, so a fragment root does not visibly
    // break here — but Vue still warns, and in a real browser that same
    // condition leaves <Transition mode="out-in"> waiting forever.
    const warnings: string[] = []
    const spy = vi.spyOn(console, 'warn').mockImplementation((...args) => {
      warnings.push(args.map(String).join(' '))
    })

    const { wrapper, router } = await shell()
    await router.push('/device/481871d4')
    await flushPromises()
    spy.mockRestore()

    const transitionWarnings = warnings.filter((w) => /Transition/i.test(w))
    expect(
      transitionWarnings,
      `Vue warned about <Transition> children:\n${transitionWarnings.join('\n')}`,
    ).toEqual([])
    wrapper.unmount()
  })
})
