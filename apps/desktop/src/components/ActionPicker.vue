<script setup lang="ts">
import { computed, ref, toRef, watch } from 'vue'
import { useDialog } from '@/composables/useDialog'
import AppIcon from './AppIcon.vue'
import ShortcutRecorder from './ShortcutRecorder.vue'
import type { ActionPayload, Control, SwitchMode, WorkspaceDirection } from '@/types/device'

const props = defineProps<{
  open: boolean
  control: Control | null
  current: ActionPayload
}>()
const emit = defineEmits<{ close: []; apply: [ActionPayload] }>()

useDialog(toRef(props, 'open'), () => emit('close'))

type Kind = ActionPayload['kind']

const kinds: { id: Kind; label: string; icon: string; hint: string }[] = [
  { id: 'default', label: 'Default', icon: 'refresh', hint: 'Let the firmware handle it' },
  { id: 'keystroke', label: 'Keyboard shortcut', icon: 'keyboard', hint: 'Send a key combination' },
  { id: 'media', label: 'Media key', icon: 'activity', hint: 'Playback and volume' },
  { id: 'launch', label: 'Launch app', icon: 'zap', hint: 'Start a program' },
  { id: 'command', label: 'Run command', icon: 'command', hint: 'Through your shell' },
  { id: 'url', label: 'Open URL', icon: 'sparkle', hint: 'In your default browser' },
  { id: 'mouseButton', label: 'Mouse button', icon: 'mouse', hint: 'Emit a different click' },
  { id: 'workspace', label: 'Workspace', icon: 'layers', hint: 'Switch or move a window' },
  { id: 'appSwitch', label: 'Switch app', icon: 'refresh', hint: 'Alt-Tab and overview' },
  { id: 'disabled', label: 'Disable', icon: 'x', hint: 'Button does nothing' },
]

const kind = ref<Kind>('default')
const shortcut = ref('')
const mediaKey = ref('playpause')
const text = ref('')
const mouseButton = ref('middle')
const wsDirection = ref<WorkspaceDirection>('right')
const wsMoveWindow = ref(false)
const switchMode = ref<SwitchMode>('applications')
const switchBackward = ref(false)

const switchModes: { id: SwitchMode; label: string; hint: string }[] = [
  { id: 'applications', label: 'Switch applications', hint: 'A quick tap returns to your last app' },
  { id: 'windows', label: 'Switch windows', hint: 'Individual windows, not applications' },
  { id: 'windowsOfApp', label: 'Windows of this app', hint: 'Cycle within the focused app' },
  { id: 'overview', label: 'Overview', hint: 'Show all windows and workspaces' },
  { id: 'appGrid', label: 'Application grid', hint: 'Show the installed apps' },
]

/** Direction only means something for the cycling switchers. */
const switchHasDirection = computed(
  () => !['overview', 'appGrid'].includes(switchMode.value),
)

const mediaKeys = [
  'playpause', 'nexttrack', 'prevtrack', 'stop',
  'volumeup', 'volumedown', 'mute',
  'brightnessup', 'brightnessdown', 'screenshot',
]
const mouseButtons = ['left', 'right', 'middle', 'back', 'forward']

// Seed the form from the current mapping when the dialog OPENS, and only
// then.
//
// Watching `props.current` as well looks harmless but is not: the parent
// builds that object inline, so it is a fresh reference on every re-render.
// The watcher would re-fire while the dialog is open and reset the fields the
// user is filling in — a recorded shortcut would vanish the instant it was
// captured, leaving Assign permanently disabled.
watch(
  () => props.open,
  (open) => {
    if (!open) return
    const c = props.current
    kind.value = c.kind
    shortcut.value = c.kind === 'keystroke' ? c.shortcut : ''
    mediaKey.value = c.kind === 'media' ? c.key : 'playpause'
    mouseButton.value = c.kind === 'mouseButton' ? c.button : 'middle'
    wsDirection.value = c.kind === 'workspace' ? c.direction : 'right'
    wsMoveWindow.value = c.kind === 'workspace' ? c.moveWindow : false
    switchMode.value = c.kind === 'appSwitch' ? c.mode : 'applications'
    switchBackward.value = c.kind === 'appSwitch' ? c.backward : false
    text.value =
      c.kind === 'launch' ? c.command : c.kind === 'command' ? c.script : c.kind === 'url' ? c.url : ''
  },
  { immediate: true },
)

const textLabel = computed(
  () =>
    ({ launch: 'Command', command: 'Shell script', url: 'URL' })[
      kind.value as 'launch' | 'command' | 'url'
    ] ?? '',
)
const textPlaceholder = computed(
  () =>
    ({
      launch: 'code, firefox, gnome-terminal…',
      command: 'notify-send "hello"',
      url: 'https://figma.com',
    })[kind.value as 'launch' | 'command' | 'url'] ?? '',
)

const valid = computed(() => {
  switch (kind.value) {
    case 'keystroke':
      return shortcut.value.length > 0
    case 'launch':
    case 'command':
    case 'url':
      return text.value.trim().length > 0
    default:
      return true
  }
})

function build(): ActionPayload {
  switch (kind.value) {
    case 'keystroke': return { kind: 'keystroke', shortcut: shortcut.value }
    case 'media': return { kind: 'media', key: mediaKey.value }
    case 'launch': return { kind: 'launch', command: text.value.trim() }
    case 'command': return { kind: 'command', script: text.value.trim() }
    case 'url': return { kind: 'url', url: text.value.trim() }
    case 'mouseButton': return { kind: 'mouseButton', button: mouseButton.value }
    case 'workspace':
      return { kind: 'workspace', direction: wsDirection.value, moveWindow: wsMoveWindow.value }
    case 'appSwitch':
      return {
        kind: 'appSwitch',
        mode: switchMode.value,
        backward: switchHasDirection.value ? switchBackward.value : false,
      }
    case 'disabled': return { kind: 'disabled' }
    default: return { kind: 'default' }
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-200 ease-[var(--ease-out-quint)]"
      enter-from-class="opacity-0"
      leave-active-class="transition duration-150"
      leave-to-class="opacity-0"
    >
      <div
        v-if="open"
        class="fixed inset-0 z-50 grid place-items-center bg-black/50 p-6"
        @click.self="emit('close')"
      >
        <Transition
          appear
          enter-active-class="transition duration-250 ease-[var(--ease-out-quint)]"
          enter-from-class="translate-y-3 scale-[0.98] opacity-0"
        >
          <div
            class="flex max-h-[85vh] w-full max-w-lg flex-col overflow-hidden rounded-[var(--radius-panel)] border shadow-[var(--shadow-pop)]"
            :style="{ background: 'var(--surface-raised)', borderColor: 'var(--border)' }"
            role="dialog"
            aria-modal="true"
            :aria-label="`Assign action to ${control?.name}`"
          >
            <header
              class="flex items-center justify-between border-b px-5 py-4"
              :style="{ borderColor: 'var(--border)' }"
            >
              <div>
                <h2 class="text-title">Assign action</h2>
                <p class="mt-0.5 text-caption text-[var(--text-muted)]">
                  {{ control?.name }} · 0x{{ control?.cid.toString(16).padStart(4, '0') }}
                </p>
              </div>
              <button
                class="rounded-lg p-1.5 text-[var(--text-subtle)] transition-colors hover:bg-[var(--surface-sunken)] hover:text-[var(--text)]"
                aria-label="Close"
                @click="emit('close')"
              >
                <AppIcon name="x" :size="16" />
              </button>
            </header>

            <div class="min-h-0 flex-1 overflow-y-auto p-5">
              <div class="grid grid-cols-2 gap-2">
                <button
                  v-for="k in kinds"
                  :key="k.id"
                  class="flex items-start gap-2.5 rounded-[10px] border p-3 text-left transition-colors"
                  :class="kind === k.id ? 'border-brand-500' : 'hover:border-[var(--border-strong)]'"
                  :style="{
                    borderColor: kind === k.id ? undefined : 'var(--border)',
                    background: kind === k.id ? 'var(--surface-sunken)' : 'transparent',
                  }"
                  @click="kind = k.id"
                >
                  <AppIcon
                    :name="k.icon"
                    :size="16"
                    class="mt-0.5 shrink-0"
                    :class="kind === k.id ? 'text-brand-500' : 'text-[var(--text-muted)]'"
                  />
                  <span class="min-w-0">
                    <span class="block text-label font-medium">{{ k.label }}</span>
                    <span class="block text-caption text-[var(--text-muted)]">{{ k.hint }}</span>
                  </span>
                </button>
              </div>

              <div v-if="kind !== 'default' && kind !== 'disabled'" class="mt-5">
                <ShortcutRecorder v-if="kind === 'keystroke'" v-model="shortcut" />

                <div v-else-if="kind === 'media'">
                  <label class="mb-2 block text-label font-medium text-[var(--text-muted)]">
                    Key
                  </label>
                  <select
                    v-model="mediaKey"
                    class="h-11 w-full rounded-xl border px-3 text-label outline-none focus:border-[var(--color-brand-500)]"
                    :style="{ background: 'var(--surface-sunken)', borderColor: 'var(--border)' }"
                  >
                    <option v-for="o in mediaKeys" :key="o" :value="o">{{ o }}</option>
                  </select>
                </div>

                <div v-else-if="kind === 'mouseButton'">
                  <label class="mb-2 block text-label font-medium text-[var(--text-muted)]">
                    Button
                  </label>
                  <select
                    v-model="mouseButton"
                    class="h-11 w-full rounded-xl border px-3 text-label outline-none focus:border-[var(--color-brand-500)]"
                    :style="{ background: 'var(--surface-sunken)', borderColor: 'var(--border)' }"
                  >
                    <option v-for="o in mouseButtons" :key="o" :value="o">{{ o }}</option>
                  </select>
                </div>

                <div v-else-if="kind === 'appSwitch'">
                  <label class="mb-2 block text-label font-medium text-[var(--text-muted)]">
                    Switcher
                  </label>
                  <div class="space-y-2">
                    <button
                      v-for="m in switchModes"
                      :key="m.id"
                      class="flex w-full items-start gap-2.5 rounded-[10px] border p-3 text-left transition-colors"
                      :class="switchMode === m.id ? 'border-brand-500' : 'hover:border-[var(--border-strong)]'"
                      :style="{
                        borderColor: switchMode === m.id ? undefined : 'var(--border)',
                        background: switchMode === m.id ? 'var(--surface-sunken)' : 'transparent',
                      }"
                      @click="switchMode = m.id"
                    >
                      <span class="min-w-0">
                        <span class="block text-label font-medium">{{ m.label }}</span>
                        <span class="block text-caption text-[var(--text-muted)]">{{ m.hint }}</span>
                      </span>
                    </button>
                  </div>

                  <label
                    v-if="switchHasDirection"
                    class="mt-3 flex cursor-pointer items-start gap-2.5 rounded-[10px] border p-3"
                    :style="{ borderColor: 'var(--border)' }"
                  >
                    <input v-model="switchBackward" type="checkbox" class="mt-0.5 accent-[var(--color-brand-500)]" />
                    <span class="min-w-0">
                      <span class="block text-label font-medium">Cycle backwards</span>
                      <span class="block text-caption text-[var(--text-muted)]">
                        Useful on a second button, so one goes each way.
                      </span>
                    </span>
                  </label>

                  <p class="mt-2.5 text-caption leading-relaxed text-[var(--text-subtle)]">
                    Read from your desktop settings at press time, so rebinding these keys keeps
                    the button working.
                  </p>
                </div>

                <div v-else-if="kind === 'workspace'">
                  <label class="mb-2 block text-label font-medium text-[var(--text-muted)]">
                    Direction
                  </label>
                  <div class="grid grid-cols-4 gap-2">
                    <button
                      v-for="d in (['left', 'right', 'up', 'down'] as WorkspaceDirection[])"
                      :key="d"
                      class="rounded-xl border py-2.5 text-label font-medium capitalize transition-colors"
                      :class="wsDirection === d ? 'border-brand-500' : 'hover:border-[var(--border-strong)]'"
                      :style="{
                        borderColor: wsDirection === d ? undefined : 'var(--border)',
                        background: wsDirection === d ? 'var(--surface-sunken)' : 'transparent',
                      }"
                      @click="wsDirection = d"
                    >
                      {{ d }}
                    </button>
                  </div>

                  <label
                    class="mt-3 flex cursor-pointer items-start gap-2.5 rounded-[10px] border p-3"
                    :style="{ borderColor: 'var(--border)' }"
                  >
                    <input v-model="wsMoveWindow" type="checkbox" class="mt-0.5 accent-[var(--color-brand-500)]" />
                    <span class="min-w-0">
                      <span class="block text-label font-medium">Take the focused window with me</span>
                      <span class="block text-caption text-[var(--text-muted)]">
                        Moves the window to that workspace instead of just switching.
                      </span>
                    </span>
                  </label>

                  <p class="mt-2.5 text-caption leading-relaxed text-[var(--text-subtle)]">
                    The actual key chord is read from your desktop settings when the button is
                    pressed, so rebinding your workspace shortcuts keeps this working.
                  </p>
                </div>

                <div v-else>
                  <label class="mb-2 block text-label font-medium text-[var(--text-muted)]">
                    {{ textLabel }}
                  </label>
                  <input
                    v-model="text"
                    type="text"
                    :placeholder="textPlaceholder"
                    class="h-11 w-full rounded-xl border px-3 text-label outline-none placeholder:text-[var(--text-subtle)] focus:border-[var(--color-brand-500)]"
                    :style="{ background: 'var(--surface-sunken)', borderColor: 'var(--border)' }"
                  />
                </div>
              </div>
            </div>

            <footer
              class="flex items-center justify-end gap-2 border-t px-5 py-4"
              :style="{ borderColor: 'var(--border)' }"
            >
              <button
                class="pressable rounded-[9px] px-3.5 py-2 text-label font-medium text-[var(--text-muted)] hover:bg-[var(--surface-hover)]"
                @click="emit('close')"
              >
                Cancel
              </button>
              <button
                class="bg-brand-500 hover:bg-brand-600 rounded-[10px] px-3.5 py-2 text-label font-medium text-white transition-colors disabled:opacity-40"
                :disabled="!valid"
                @click="emit('apply', build())"
              >
                Assign
              </button>
            </footer>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>
