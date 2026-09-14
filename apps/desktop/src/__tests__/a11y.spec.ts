import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

import ActionPicker from '@/components/ActionPicker.vue'
import type { Control } from '@/types/device'

const control: Control = {
  cid: 0x53, taskId: 0x3c, name: 'Back',
  reprogrammable: true, divertable: true, supportsGestures: true, virtual: false,
}

describe('dialog keyboard handling', () => {
  it('closes on Escape', async () => {
    const w = mount(ConfirmDialog, {
      props: { open: true, title: 'Reset?', message: 'Sure?' },
      attachTo: document.body,
      global: { stubs: { teleport: true } },
    })
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await w.vm.$nextTick()
    expect(w.emitted('close'), 'Escape should close the dialog').toBeTruthy()
    w.unmount()
  })

  it('closes the action picker on Escape too', async () => {
    const w = mount(ActionPicker, {
      props: { open: true, control, current: { kind: 'default' } },
      attachTo: document.body,
      global: { stubs: { teleport: true } },
    })
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await w.vm.$nextTick()
    expect(w.emitted('close')).toBeTruthy()
    w.unmount()
  })

  it('stops listening once closed, so Escape elsewhere is unaffected', async () => {
    const w = mount(ConfirmDialog, {
      props: { open: false, title: 'Reset?', message: 'Sure?' },
      global: { stubs: { teleport: true } },
    })
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await w.vm.$nextTick()
    expect(w.emitted('close')).toBeFalsy()
    w.unmount()
  })
})
