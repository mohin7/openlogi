/**
 * Route frontend faults into the Rust log.
 *
 * A webview error is invisible to anyone reading the terminal, which is
 * exactly the situation where you most need it — "the app froze" with an empty
 * log is unfalsifiable. These handlers make a UI fault leave a trace.
 */
import { isTauri } from './backend'

async function send(level: 'error' | 'warn' | 'info', message: string, detail?: string) {
  // Always keep the console copy: devtools is the faster path when open.
  const logger = level === 'error' ? console.error : level === 'warn' ? console.warn : console.info
  logger(`[openlogi] ${message}`, detail ?? '')
  if (!isTauri) return
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('log_frontend', { level, message, detail })
  } catch {
    // Never let diagnostics become the thing that breaks the app.
  }
}

export function installDiagnostics() {
  window.addEventListener('error', (e) => {
    void send('error', e.message, e.error?.stack ?? `${e.filename}:${e.lineno}`)
  })

  window.addEventListener('unhandledrejection', (e) => {
    const reason = e.reason
    void send(
      'error',
      `Unhandled rejection: ${reason instanceof Error ? reason.message : String(reason)}`,
      reason instanceof Error ? reason.stack : undefined,
    )
  })

  // A long task blocking the main thread is what a user calls a freeze. This
  // does not prevent one, but it records that it happened and for how long.
  if ('PerformanceObserver' in window) {
    try {
      const observer = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          if (entry.duration > 500) {
            void send('warn', `UI blocked for ${Math.round(entry.duration)}ms`, entry.name)
          }
        }
      })
      observer.observe({ entryTypes: ['longtask'] })
    } catch {
      // longtask is not supported everywhere; not worth a fuss.
    }
  }
}

/**
 * Record which route the user is on, and whether it actually rendered.
 *
 * Logging only the navigation start cannot distinguish "the router never
 * finished" from "the router finished but the DOM stayed empty" — and those
 * have completely different causes. Checking the content element after the
 * next frame separates them.
 */
export function logNavigation(path: string) {
  void send('info', `navigated to ${path}`)

  // Sample repeatedly rather than once. A page transition can legitimately
  // leave the content area empty for a moment, so a single early check
  // produces false alarms; only content still missing after everything has
  // had time to settle is a real fault.
  const checkpoints = [150, 600, 2000]
  let settled = false

  checkpoints.forEach((delay, index) => {
    window.setTimeout(() => {
      if (settled) return
      const main = document.querySelector('main')
      const count = main ? main.querySelectorAll('*').length : -1
      if (count > 0) {
        settled = true
        return
      }
      // Only report on the final checkpoint; earlier ones are just waiting.
      if (index === checkpoints.length - 1) {
        void send(
          'error',
          `route ${path} still empty after ${delay}ms`,
          main ? 'main element present but has no children' : 'main element missing',
        )
      }
    }, delay)
  })
}
