import { createRouter, createWebHistory, type RouteComponent } from 'vue-router'
import { h } from 'vue'
import RouteLoadError from '@/pages/RouteLoadError.vue'

/**
 * Load a route component, reporting failure instead of hanging.
 *
 * Route components are code-split, so navigating fetches a chunk at runtime.
 * If that fetch fails — the dev server has stopped, or a packaged build's
 * files changed under a running app — the returned promise never resolves and
 * Vue Router simply never completes the navigation. To the user the window
 * appears frozen, with nothing in any log to explain it.
 *
 * One retry covers a transient blip; after that we resolve to a component that
 * says what happened. A visible error always beats a silent hang.
 */
function lazyPage(name: string, loader: () => Promise<RouteComponent>) {
  return async (): Promise<RouteComponent> => {
    try {
      return await loader()
    } catch (first) {
      try {
        return await loader()
      } catch (second) {
        const reason = second instanceof Error ? second.message : String(second)
        console.error(`[openlogi] failed to load route "${name}"`, first, second)
        window.dispatchEvent(
          new ErrorEvent('error', { message: `Route "${name}" failed to load: ${reason}` }),
        )
        return { render: () => h(RouteLoadError, { page: name, reason }) }
      }
    }
  }
}

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'devices', component: lazyPage('Devices', () => import('@/pages/DashboardPage.vue')) },
    { path: '/device/:id', name: 'device', component: lazyPage('Device', () => import('@/pages/DeviceDetailPage.vue')) },
    { path: '/profiles', name: 'profiles', component: lazyPage('Profiles', () => import('@/pages/ProfilesPage.vue')) },
    { path: '/gestures', name: 'gestures', component: lazyPage('Gestures', () => import('@/pages/GesturesPage.vue')) },
    { path: '/macros', name: 'macros', component: lazyPage('Macros', () => import('@/pages/MacrosPage.vue')) },
    { path: '/settings', name: 'settings', component: lazyPage('Settings', () => import('@/pages/SettingsPage.vue')) },
    { path: '/about', name: 'about', component: lazyPage('About', () => import('@/pages/AboutPage.vue')) },
    { path: '/:pathMatch(.*)*', redirect: '/' },
  ],
})

// A navigation that fails outright must not leave the UI on the old page with
// no explanation.
router.onError((error, to) => {
  console.error(`[openlogi] navigation to ${to.fullPath} failed`, error)
  window.dispatchEvent(
    new ErrorEvent('error', {
      message: `Navigation to ${to.fullPath} failed: ${error.message}`,
    }),
  )
})
