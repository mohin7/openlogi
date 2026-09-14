import { onBeforeUnmount, watch, type Ref } from 'vue'

/**
 * Shared modal behaviour: Escape to close, and focus returned to wherever it
 * was when the dialog opened.
 *
 * The naive approach — `@keydown.esc` on the overlay element — silently does
 * nothing, because a `div` is not focusable and so never receives key events.
 * The listener has to live on `document` for as long as the dialog is open.
 */
export function useDialog(open: Ref<boolean> | (() => boolean), onClose: () => void) {
  const isOpen = typeof open === 'function' ? open : () => open.value
  let previouslyFocused: HTMLElement | null = null

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.stopPropagation()
      onClose()
    }
  }

  function attach() {
    previouslyFocused = document.activeElement as HTMLElement | null
    document.addEventListener('keydown', onKeydown)
  }

  function detach() {
    document.removeEventListener('keydown', onKeydown)
    // Returning focus matters for keyboard and screen-reader users: without
    // it, focus falls back to the top of the document and they lose their
    // place in the list they opened the dialog from.
    previouslyFocused?.focus?.()
    previouslyFocused = null
  }

  watch(
    () => isOpen(),
    (v) => (v ? attach() : detach()),
    { immediate: true },
  )

  onBeforeUnmount(detach)
}
