# Logitech Signature M650 — measured capabilities

Captured with `logi-probe` over a Logi Bolt receiver (`046d:c548`),
firmware `RBM16.02.B0013`, HID++ **4.5**.

This file is **measured, not assumed**. Everything below came off the wire.

## Identity

| | |
|---|---|
| Name (`0x0005`) | `Signature M650 Mouse` |
| Type | mouse |
| Main firmware (`0x0003`) | `RBM16.02.B0013` |
| Bootloader | `BL138.02.B0013` |
| Protocol | HID++ 4.5 |
| Device index | 2 (behind receiver) |
| **Unit id** | `481871D4` — via `0x0003` **v4**, not `0x0004` |
| Model id (WPID) | `B02A` |
| Transport | **BLE only** (Logi Bolt is BLE-based; Unifying is eQuad) |
| Features implemented | 30 (14 user-relevant, 16 manufacturing/test) |

## Controls (`0x1B04` v5)

| CID | Name | Reprogrammable | Divertable | Raw XY | Group / mask |
|---|---|---|---|---|---|
| `0x0050` | Left Click | ✗ | ✗ | ✗ | 1 / `0x00` |
| `0x0051` | Right Click | ✗ | ✗ | ✗ | 1 / `0x00` |
| `0x0052` | Middle Click | ✓ | ✓ | ✓ | 3 / `0x07` |
| `0x0053` | Back | ✓ | ✓ | ✓ | 2 / `0x03` |
| `0x0056` | Forward | ✓ | ✓ | ✓ | 2 / `0x03` |
| `0x00D7` | Gesture (**virtual**) | ✗ | ✓ | ✓ forced | 4 / `0x00` |

Consequences:

* **Left and right click cannot be remapped or diverted.** The firmware
  refuses, and that is deliberate — a user who remapped left-click would have
  no way to click "undo". OpenLogi must render them as permanently locked
  rather than letting the user try and fail.
* **Three physical buttons are remappable**: middle, back, forward.
* `0x00D7` is **virtual and forced-raw-XY** — it is not a physical button. It
  is the hook the firmware provides for gesture recognition: divert it and the
  device streams pointer deltas. **This is what makes the Phase 2 Gesture
  Editor buildable on this device.**
* The group mask governs *onboard task swapping*: middle click (mask `0x07`)
  can take on any group 1–3 task, back/forward (mask `0x03`) only groups 1–2.
  That covers "swap buttons" and "middle-click customisation" in firmware.
  Everything richer needs diversion.

## Pointer

| | |
|---|---|
| `0x2201` Adjustable DPI v2 | ✓ |
| Range | 400–4000, step 100 (37 values) |
| Current / default | 1000 / 1000 |
| `0x2205` Pointer Speed | ✗ **absent** |

**The "pointer speed" slider must be backed by DPI.** The device has no
separate pointer-speed feature, so the slider maps directly onto `0x2201`.
DPI is written onboard, so it persists with OpenLogi closed.

## Battery

| | |
|---|---|
| Feature | `0x1004` Unified Battery v3 |
| Reading at capture | 30%, discharging |
| Charging states | n/a — AA cell, not rechargeable |

## Scrolling

| | |
|---|---|
| `0x2130` Low-Res Wheel | ✓ (notification only) |
| `0x2121` Hi-Res Wheel | ✗ |
| `0x2110`/`0x2111` SmartShift | ✗ |
| `0x2150` Thumb Wheel | ✗ |

**SmartWheel is not configurable on this device.** Logitech's "SmartWheel"
marketing for the M650 refers to a *mechanical* silent-scroll design, not a
software-controlled ratchet — there is no corresponding HID++ feature. Nor is
there horizontal-scroll hardware.

## Absent features worth naming

| Feature | Impact |
|---|---|
| `0x1C00` Persistent Remappable Action | Remaps **cannot** be stored onboard — the daemon must re-apply diversion on every connect |
| `0x8100` Onboard Profiles | No device-side profiles; all profiles are host-side |
| `0x8060` Report Rate | Polling rate cannot be read or set over HID++ |
| `0x0004` Device Unit ID | **No stable unit id** — profiles need a different key (see below) |
| `0x6501` Gesture v2 | No firmware gesture engine; recognition is ours to do from raw XY |

### The unit-id problem — solved

Profiles must be keyed to something that survives re-pairing and index changes.
This device does not implement `0x0004`, which is the obvious source.

It turns out not to matter: feature `0x0003` is **v4** here, and v4 extended
`getDeviceInfo` to return the unit id, transport bitfield and per-transport
model ids alongside the entity count. So the key is `481871D4`, read from
`0x0003` function 0, and no receiver-register fallback is needed.

`features::info::device_info()` implements this and returns `None` below v4, at
which point `0x0004` and then a composite key remain as fallbacks.

## Revised phase scope for this device

| Spec item | Verdict |
|---|---|
| Battery percentage | ✓ `0x1004` |
| Firmware version | ✓ `0x0003` |
| Connection type | ✓ from endpoint/PID |
| DPI presets | ✓ `0x2201`, 400–4000 |
| Pointer speed slider | ✓ but **backed by DPI**, not `0x2205` |
| Button remapping | ✓ 3 of 5 buttons; L/R locked by firmware |
| Gesture button | ✓ via virtual CID `0x00D7` |
| SmartWheel customisation | ✗ not supported by hardware |
| Horizontal scrolling | ✗ no hardware |
| Per-app profiles | ✓ host-side (needs unit-id fix) |
| Macros | ✓ host-side |
| Polling rate in diagnostics | ✗ not exposed over HID++ |


## Behavioural findings (verified on hardware)

### Notifications must be enabled explicitly

Diverting a control succeeds and reads back as `diverted=true`, but the device
still sends **nothing** until notification forwarding is switched on through
HID++ 1.0 register `0x00`. The flags that matter are `WIRELESS` (`0x000100`)
and `SOFTWARE_PRESENT` (`0x000800`) — the latter is how the device learns that
configuration software is listening.

Readback on this receiver returns `0x000900`; the `BATTERY_STATUS` bit
(`0x100000`) is not retained, so battery must be polled rather than awaited.

This is a startup step for the daemon, not a per-device one, and it must be
re-applied whenever the receiver is re-plugged.

### hidraw is a broadcast channel

Every process with the node open receives **every** reply, including replies to
other programs' requests. On this machine `fwupd` queries the mouse with
software id `7` (pinging with mark `0xAA`, reading `0x0003`), and those replies
land in our read buffer.

Two consequences, both already handled:

* Matching replies on the echoed software id is **mandatory**, not defensive
  programming. OpenLogi uses `0x0A`; Solaar uses `0x01`–`0x08`; fwupd uses `7`.
* `poll_event` must not filter on `software_id == 0`. That byte is only a
  software id in HID++ 2.0; in 1.0 notifications (sub-ids `0x40`–`0x4F`) it
  carries data, so such a filter silently drops connection and pairing events.

### Diversion is confirmed working

All four divertable controls accept diversion and read back as diverted. The
flags byte for `setControlReporting` interleaves each value bit with its own
validity bit one position to the left (`valid == value << 1`), so plain
diversion is `0x03`. An encoding that groups the value bits separately produces
`0x0C`, which the firmware reads as *persistent* diversion and rejects with
`InvalidArgument` on any device lacking `0x1C00`.
