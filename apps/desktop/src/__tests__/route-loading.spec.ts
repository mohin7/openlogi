import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createRouter, createMemoryHistory } from 'vue-router'
import { h, defineComponent } from 'vue'
import RouteLoadError from '@/pages/RouteLoadError.vue'

/**
 * Regression: a route chunk that fails to load used to hang the navigation
 * forever, which presented as a frozen window with nothing in the logs.
 */
function lazyPage(name: string, loader: () => Promise<unknown>) {
  return async () => {
    try {
      return (await loader()) as never
    } catch {
      try {
        return (await loader()) as never
      } catch (second) {
        const reason = second instanceof Error ? second.message : String(second)
        return { render: () => h(RouteLoadError, { page: name, reason }) } as never
      }
    }
  }
}

const Host = defineComponent({ template: '<RouterView />' })

describe('route chunk loading', () => {
  it('shows an error page instead of hanging when a chunk fails', async () => {
    const loader = vi.fn().mockRejectedValue(new Error('Failed to fetch dynamically imported module'))
    const router = createRouter({
      history: createMemoryHistory(),
      routes: [{ path: '/about', component: lazyPage('About', loader) }],
    })
    const w = mount(Host, { global: { plugins: [router] } })
    await router.push('/about')
    await router.isReady()
    await new Promise((r) => setTimeout(r, 0))
    await w.vm.$nextTick()

    expect(loader).toHaveBeenCalledTimes(2) // original + one retry
    expect(w.html()).toContain("Couldn't load")
    w.unmount()
  })

  it('retries once and succeeds on a transient failure', async () => {
    const good = { template: '<div>About works</div>' }
    const loader = vi
      .fn()
      .mockRejectedValueOnce(new Error('transient'))
      .mockResolvedValue(good)
    const router = createRouter({
      history: createMemoryHistory(),
      routes: [{ path: '/about', component: lazyPage('About', loader) }],
    })
    const w = mount(Host, { global: { plugins: [router] } })
    await router.push('/about')
    await new Promise((r) => setTimeout(r, 0))
    await w.vm.$nextTick()
    expect(w.html()).toContain('About works')
    w.unmount()
  })
})
