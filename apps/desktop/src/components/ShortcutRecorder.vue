<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'
import AppIcon from './AppIcon.vue'

const props = defineProps<{ modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [string] }>()

const recording = ref(false)

/**
 * Map a physical key to the names `button_engine::keys` understands.
 *
 * We deliberately read `event.code` (physical position) rather than
 * `event.key` (the character produced). uinput emits scancodes, so a recorder
 * that captured characters would record the wrong key on any non-US layout —
 * pressing the key engraved Z on an AZERTY keyboard must record as the same
 * scancode Rust will later emit.
 */
function codeToName(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3).toLowerCase()
  if (/^Digit[0-9]$/.test(code)) return code.slice(5)
  if (/^F([1-9]|1[0-2])$/.test(code)) return code.toLowerCase()
  if (/^Arrow/.test(code)) return code.slice(5).toLowerCase()
  const table: Record<string, string> = {
    Escape: 'escape', Tab: 'tab', Enter: 'enter', NumpadEnter: 'enter',
    Space: 'space', Backspace: 'backspace', Delete: 'delete', Insert: 'insert',
    Home: 'home', End: 'end', PageUp: 'pageup', PageDown: 'pagedown',
    Minus: 'minus', Equal: 'equal', Comma: 'comma', Period: 'dot',
    Slash: 'slash', Semicolon: 'semicolon', Quote: 'apostrophe',
    Backquote: 'grave', Backslash: 'backslash',
    BracketLeft: 'leftbrace', BracketRight: 'rightbrace',
  }
  return table[code] ?? null
}

function onKeydown(e: KeyboardEvent) {
  e.preventDefault()
  e.stopPropagation()

  const name = codeToName(e.code)
  // Modifier-only presses are ignored: the user is still assembling the chord.
  if (!name) return

  const parts: string[] = []
  if (e.ctrlKey) parts.push('ctrl')
  if (e.shiftKey) parts.push('shift')
  if (e.altKey) parts.push('alt')
  if (e.metaKey) parts.push('super')
  parts.push(name)

  emit('update:modelValue', parts.join('+'))
  stop()
}

function start() {
  if (recording.value) return
  recording.value = true
  window.addEventListener('keydown', onKeydown, { capture: true })
}

function stop() {
  recording.value = false
  window.removeEventListener('keydown', onKeydown, { capture: true })
}

onBeforeUnmount(stop)

const chips = () => (props.modelValue ? props.modelValue.split('+') : [])
const pretty: Record<string, string> = {
  ctrl: 'Ctrl', shift: 'Shift', alt: 'Alt', super: 'Super',
}
</script>

<template>
  <div>
    <button
      type="button"
      class="flex h-11 w-full items-center justify-center gap-1.5 rounded-xl border-2 border-dashed text-label transition-colors"
      :class="recording ? 'border-brand-500 bg-brand-50 dark:bg-brand-900/20' : 'hover:border-[var(--border-strong)]'"
      :style="{ borderColor: recording ? undefined : 'var(--border)' }"
      @click="recording ? stop() : start()"
    >
      <template v-if="recording">
        <span class="bg-brand-500 size-2 animate-pulse rounded-full" />
        <span class="text-brand-600 dark:text-brand-400 font-medium">Press keys…</span>
      </template>
      <template v-else-if="modelValue">
        <kbd
          v-for="(c, i) in chips()"
          :key="i"
          class="rounded-md border px-1.5 py-0.5 font-sans text-caption font-medium"
          :style="{ background: 'var(--surface-sunken)', borderColor: 'var(--border)' }"
        >
          {{ pretty[c] ?? c.toUpperCase() }}
        </kbd>
      </template>
      <template v-else>
        <AppIcon name="keyboard" :size="15" class="text-[var(--text-subtle)]" />
        <span class="text-[var(--text-muted)]">Click, then press a shortcut</span>
      </template>
    </button>
    <p v-if="recording" class="mt-1.5 text-center text-caption text-[var(--text-subtle)]">
      Modifiers alone are ignored — finish with a normal key. Esc records as Escape.
    </p>
  </div>
</template>
