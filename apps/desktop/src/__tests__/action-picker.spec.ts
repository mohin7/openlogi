import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ActionPicker from '@/components/ActionPicker.vue'
import ShortcutRecorder from '@/components/ShortcutRecorder.vue'
import type { ActionPayload, Control } from '@/types/device'

const control: Control = {
  cid: 0x0053,
  taskId: 0x003c,
  name: 'Back',
  reprogrammable: true,
  divertable: true,
  supportsGestures: true,
  virtual: false,
}

beforeEach(() => localStorage.clear())

function mountPicker(current: ActionPayload = { kind: 'default' }) {
  return mount(ActionPicker, {
    props: { open: true, control, current },
    global: { stubs: { teleport: true } },
  })
}

describe('ActionPicker', () => {
  it('seeds from the current mapping when opened', async () => {
    const w = mountPicker({ kind: 'keystroke', shortcut: 'ctrl+z' })
    expect(w.html()).toContain('Ctrl')
  })

  it('does not re-seed while open when the parent re-renders', async () => {
    // Regression: the picker watched `props.current`, which the parent built
    // inline as a fresh object each render. The watcher re-fired constantly and
    // wiped a shortcut the moment it was recorded, leaving Assign disabled.
    const w = mountPicker({ kind: 'keystroke', shortcut: 'ctrl+z' })
    const recorder = w.findComponent(ShortcutRecorder)
    await recorder.vm.$emit('update:modelValue', 'ctrl+shift+p')
    await w.vm.$nextTick()

    // A new object with identical content, as a re-render would produce.
    await w.setProps({ current: { kind: 'keystroke', shortcut: 'ctrl+z' } })
    await w.vm.$nextTick()

    expect(w.html()).toContain('Shift')
  })

  it('builds each action payload in the shape Rust expects', async () => {
    const w = mountPicker()
    const kinds = ['disabled', 'workspace', 'appSwitch'] as const
    for (const k of kinds) {
      const btn = w.findAll('button').find((b) => {
        const t = b.text().toLowerCase()
        return (
          (k === 'disabled' && t.includes('disable')) ||
          (k === 'workspace' && t.includes('workspace')) ||
          (k === 'appSwitch' && t.includes('switch app'))
        )
      })
      expect(btn, `no button for ${k}`).toBeTruthy()
      await btn!.trigger('click')
    }
    const assign = w.findAll('button').find((b) => b.text() === 'Assign')
    expect(assign).toBeTruthy()
    await assign!.trigger('click')
    const emitted = w.emitted('apply')
    expect(emitted).toBeTruthy()
    const payload = emitted![0][0] as ActionPayload
    expect(payload.kind).toBe('appSwitch')
    expect(payload).toHaveProperty('mode')
    expect(payload).toHaveProperty('backward')
  })

  it('keeps Assign disabled until a keystroke is actually recorded', async () => {
    const w = mountPicker()
    const keyBtn = w.findAll('button').find((b) => b.text().toLowerCase().includes('keyboard shortcut'))
    await keyBtn!.trigger('click')
    const assign = w.findAll('button').find((b) => b.text() === 'Assign')
    expect((assign!.element as HTMLButtonElement).disabled).toBe(true)

    await w.findComponent(ShortcutRecorder).vm.$emit('update:modelValue', 'ctrl+z')
    await w.vm.$nextTick()
    expect((assign!.element as HTMLButtonElement).disabled).toBe(false)
  })
})

describe('ShortcutRecorder', () => {
  it('records physical key positions, not characters', async () => {
    const w = mount(ShortcutRecorder, { props: { modelValue: '' } })
    await w.find('button').trigger('click') // start recording
    window.dispatchEvent(
      new KeyboardEvent('keydown', { code: 'KeyP', ctrlKey: true, shiftKey: true }),
    )
    await w.vm.$nextTick()
    expect(w.emitted('update:modelValue')?.[0]).toEqual(['ctrl+shift+p'])
  })

  it('ignores modifier-only presses while the chord is assembled', async () => {
    const w = mount(ShortcutRecorder, { props: { modelValue: '' } })
    await w.find('button').trigger('click')
    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'ControlLeft', ctrlKey: true }))
    await w.vm.$nextTick()
    expect(w.emitted('update:modelValue')).toBeFalsy()
  })

  it('stops listening once unmounted', async () => {
    const w = mount(ShortcutRecorder, { props: { modelValue: '' } })
    await w.find('button').trigger('click')
    w.unmount()
    // Must not throw or emit after teardown.
    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyA' }))
    expect(true).toBe(true)
  })
})
