# OpenLogi — Architecture

## The one decision everything else follows from

Logitech devices can be configured two ways, and picking between them shapes
the whole product:

**Onboard configuration.** Write settings into the device's own flash. Survives
reboots and works with no software running. But you are limited to the actions
Logitech's firmware already implements, and cheaper devices (the Signature
M650 included) expose very little onboard storage.

**Host-side interception.** Use HID++ *diversion* to tell the firmware "stop
emitting this button's normal report; notify me instead", then synthesise
whatever input we like through `uinput`. Unlimited actions, per-application
behaviour, macros, gestures — but it needs a process running, and it must
re-apply diversion every time the device reconnects.

OpenLogi does **both**, and prefers onboard where the firmware supports it.
A DPI change is written onboard, so it persists even with OpenLogi closed. A
button mapped to "open Figma" must be host-side, because no firmware has that
concept. The UI tells the user which is which, because the difference is
visible in behaviour: one survives a reboot without the app, the other doesn't.

This is also why there is a **daemon**, not just a GUI. The window is a client.

---

## Process model

```
┌────────────────────────────────────────────────────────────────┐
│  openlogi (Tauri window)                 user session, no root │
│  Vue 3 + Vite frontend                                         │
└───────────────┬────────────────────────────────────────────────┘
                │ Tauri IPC (in-process commands)
                │  — or D-Bus when the window is closed
┌───────────────┴────────────────────────────────────────────────┐
│  openlogid (systemd --user service)      user session, no root │
│                                                                │
│  device-service ─ hotplug watch, connection state              │
│  button-engine  ─ diversion, uinput emission                   │
│  gesture-engine ─ raw XY → direction classification            │
│  profile-engine ─ SQLite, active-window → profile switching    │
│  system-service ─ tray, notifications, autostart               │
└───────────────┬────────────────────────────────────────────────┘
                │ hidpp crate
┌───────────────┴────────────────────────────────────────────────┐
│  /dev/hidraw*  (udev uaccess ACL)   /dev/uinput                │
└────────────────────────────────────────────────────────────────┘
```

Nothing above runs as root. The single privileged action is installing a udev
rule at install time, which grants the *logged-in seat user* an ACL on Logitech
hidraw nodes via `TAG+="uaccess"`. That is better than a `plugdev` group
membership because access follows the active session and is revoked at logout.

## Crates

| Crate | Responsibility | Depends on |
|---|---|---|
| `hidpp` | Wire protocol, transport, feature wrappers. Pure protocol, no policy. | hidapi |
| `device-service` | Enumerate, hotplug, cache state, poll battery | `hidpp` |
| `button-engine` | Diversion lifecycle, action dispatch, `uinput` | `hidpp` |
| `gesture-engine` | Raw XY stream → gesture classification | — |
| `profile-engine` | Persistence, per-app profile activation | `rusqlite` |
| `system-service` | Daemon lifecycle, D-Bus, notifications, tray | `zbus` |

`hidpp` deliberately knows nothing about profiles, uinput or the UI. It is
publishable on its own, and that constraint is what keeps it testable — every
layer above it can be exercised against a fake transport.

## Why the capability model is discovered, not hard-coded

Feature *indices* differ per device and per firmware revision, so they must be
resolved at runtime through the root feature anyway. Once you are doing that,
enumerating feature `0x0001` to get the device's *entire* capability list costs
one extra round trip — and it means an MX Master 4 released after this code was
written still gets a correct UI, because the screens are gated on
"device implements `0x2201`", never on "device name matches a table".

`logi-probe` exists to print exactly that capability list. Design UI from its
output, not from assumptions.

## Sharing the device with other software

`/dev/hidraw` is a broadcast read channel: every process holding the node open
sees every reply, not just its own. Logitech's answer is the **software id**,
a 4-bit tag echoed in each reply. OpenLogi uses `0x0A`; Solaar uses `0x01`–
`0x08`; `fwupd` — present by default on Ubuntu — uses `7`.

So reply matching on software id is a correctness requirement, not hardening,
and OpenLogi must never assume it is the only program on the bus. It also means
OpenLogi and Solaar can run side by side, which is genuinely useful while
developing.

One asymmetry to remember: HID++ 1.0 has **no** software id. Byte 3 of a 1.0
notification is data. Keep 1.0 traffic confined to startup and pairing queries,
where a collision is tolerable.

## Wayland vs X11

This is the sharpest platform split in the project, and it only affects the
layers *above* `hidpp` — the protocol itself is identical.

| Concern | X11 | Wayland |
|---|---|---|
| Synthesising input | `uinput` (works) or XTEST | `uinput` only — there is no client-side injection API |
| Which app is focused | `_NET_ACTIVE_WINDOW` | No portable API; needs per-compositor D-Bus (GNOME extension, KWin script) or `xdg-desktop-portal` |
| Global shortcuts | X grabs | `org.freedesktop.portal.GlobalShortcuts` |

`uinput` being the common path is the reason it is the *only* input mechanism
OpenLogi implements: one code path, both sessions. Per-app profile detection is
the genuinely hard part on Wayland and is scoped to Phase 2 with an explicit
degradation — if the compositor cannot be queried, per-app profiles are shown
as unavailable rather than silently not working.

Your current session is X11, so Phase 2 has a working path from day one; the
Wayland path must be built and tested deliberately, not assumed.
