import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import { defineComponent, h, Transition } from 'vue'
import { useDeviceStore } from '@/stores/devices'
import DeviceDetailPage from '@/pages/DeviceDetailPage.vue'

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

/**
 * A page rendered inside <Transition mode="out-in"> must have a single root
 * element. A fragment cannot be animated: Vue cannot attach the leave hooks,
 * so the transition never reports completion and the *next* page never mounts.
 * In a browser that looks like the content area going permanently blank.
 */
describe('pages rendered inside <Transition>', () => {
  it('DeviceDetailPage has a single animatable root', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const store = useDeviceStore()
    await store.load() // realise the full template, not the loading branch

    const router = createRouter({
      history: createMemoryHistory(),
      routes: [{ path: '/device/:id', component: DeviceDetailPage }],
    })
    router.push('/device/481871d4')
    await router.isReady()

    const warnings: string[] = []
    const spy = vi.spyOn(console, 'warn').mockImplementation((...a) => {
      warnings.push(a.map(String).join(' '))
    })

    const Host = defineComponent({
      render: () => h(Transition, { mode: 'out-in' }, () => h(DeviceDetailPage)),
    })
    const w = mount(Host, { global: { plugins: [router, pinia] } })
    await flushPromises()
    spy.mockRestore()

    const bad = warnings.filter((x) => /Transition|non-element root/i.test(x))
    expect(bad, `Vue warnings:\n${bad.join('\n')}`).toEqual([])
    w.unmount()
  })
})
