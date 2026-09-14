<div align="center">

# OpenLogi

**Configure Logitech mice and keyboards natively on Linux.**

No proprietary software. No cloud account. No `sudo` after setup.

[![Licence](https://img.shields.io/badge/licence-GPL--3.0--or--later-blue)](#licence)
[![Platform](https://img.shields.io/badge/platform-Linux-informational)](#requirements)
[![Display server](https://img.shields.io/badge/Wayland%20%26%20X11-supported-success)](#why-uinput)

[Website](https://openlogi.uxatom.com) ·
[Documentation](https://openlogi.uxatom.com/docs) ·
[Download](https://github.com/mohin7/openlogi/releases/latest) ·
[Report an issue](https://github.com/mohin7/openlogi/issues)

</div>

---

Logitech has never shipped Options+ for Linux, so the hardware most people
already own is only half configurable. OpenLogi speaks Logitech's own HID++
protocol directly over the kernel's `hidraw` interface — the same way the
official Windows and macOS software does — and gives those devices a native
Linux application.

> [!NOTE]
> **Status: early but working.** The protocol layer, the hardware probe and the
> desktop application all function today. Profile persistence and
> per-application profile switching are designed but not yet implemented.

## Features

| | |
| --- | --- |
| **Remap any button** | Bind reprogrammable controls to keystrokes, media keys, mouse buttons, shell commands, URLs, or workspace and window actions. |
| **Onboard DPI** | Sensitivity is written to the device's own flash, so it survives a reboot and keeps working with OpenLogi closed. |
| **Real battery state** | Charge level and charging status read directly from the device, not estimated from voltage curves. |
| **Runtime discovery** | Panels appear because your hardware reports the matching HID++ feature — never because a model name is on a list. |
| **Shares the bus** | A distinct software id means OpenLogi coexists with Solaar and fwupd rather than fighting them for the device. |
| **Unprivileged** | One udev rule at install time. After that, access follows your login session and is revoked at logout. |

Works across **Bolt**, **Unifying**, **Lightspeed**, **Bluetooth LE** and
**direct USB** — all through one code path, because endpoints are matched by
report-descriptor shape rather than by product id.

## Install

Download the latest packages from the
[releases page](https://github.com/mohin7/openlogi/releases/latest).

```bash
# Debian / Ubuntu
sudo apt install ./openlogi_0.1.0_amd64.deb

# Fedora / RHEL
sudo dnf install ./openlogi-0.1.0-1.x86_64.rpm

# Any distribution
chmod +x OpenLogi_0.1.0_amd64.AppImage && ./OpenLogi_0.1.0_amd64.AppImage
```

The `.deb` and `.rpm` install the udev rule for you. **Log out and back in**
afterwards — group membership only applies to new sessions.

> [!IMPORTANT]
> The AppImage runs no install script, so the udev rule is not added and the
> app will not see your devices. Run `sudo ./scripts/install-udev-rules.sh`
> from this repository once, then log out and back in.

### Requirements

- 64-bit Linux with `systemd` (for the `uaccess` permission mechanism)
- Wayland or X11
- A Logitech device speaking HID++

## Build from source

Needs a Rust toolchain and Node.js 20 or newer.

```bash
git clone https://github.com/mohin7/openlogi
cd openlogi
./scripts/setup.sh    # asks for sudo once: build dependencies + udev rule
pnpm install
pnpm app              # run the desktop app
```

### Inspect your hardware

`logi-probe` reports what your device actually implements — the most useful
thing to attach to a bug report.

```bash
pnpm probe            # full capability dump
pnpm watch            # divert every button and print presses live
```

## How it works

Logitech devices can be configured two ways, and the choice shapes everything:

**Onboard.** Settings are written to the device's flash. They survive reboots
and need no software running, but you are limited to what the firmware already
implements.

**Host-side.** HID++ *diversion* tells the firmware to stop emitting a button's
normal report and send a notification instead; the application then synthesises
whatever the user mapped. Unlimited actions, but it needs a running process.

OpenLogi does **both**, preferring onboard where the firmware supports it, and
the interface marks which is which — because the difference is visible in
behaviour.

### Why `uinput`

Input is synthesised through the kernel's virtual input device rather than
injected into the display server:

| Approach | X11 | Wayland |
| --- | --- | --- |
| `XTEST` / client-side injection | Works | **Does not work** |
| `uinput` (kernel virtual device) | Works | Works |

Wayland compositors deliberately refuse client-side injection. Writing to
`uinput` produces a device the compositor treats as real hardware, so one
implementation serves both.

## Security

A tool that can remap buttons can, by construction, synthesise keystrokes. The
privilege model is therefore deliberately narrow:

- **Never runs as root.** The single privileged action is installing a udev
  rule at setup time.
- **Session-scoped access.** `TAG+="uaccess"` has `systemd-logind` grant an ACL
  to the user at the active seat, revoked at logout — not permanent group
  membership.
- **Minimum capability.** `uinput` write access uses a dedicated `openlogi`
  group rather than `input`, which would grant *read* access to every
  `/dev/input/event*` — the ability to log every keystroke on the system.
- **No network code.** No account, no telemetry, no update ping.

Full details in [Permissions](https://openlogi.uxatom.com/docs/permissions).

## Project layout

```
crates/
  hidpp           HID++ 1.0/2.0 over hidraw — transport, features, discovery
  button-engine   Diversion handling and uinput event synthesis
  logi-probe      CLI hardware inspector
apps/desktop/
  src/            Vue 3 + Vite frontend
  src-tauri/      Tauri v2 shell and device service
scripts/          udev rules and setup
```

## Documentation

| Guide | Contents |
| --- | --- |
| [Architecture](docs/architecture.md) | Onboard vs host-side, protocol layer, bus sharing |
| [Device notes](docs/devices/) | Per-model findings from real hardware |
| [Releasing](docs/releasing.md) | Tagging, CI builds, publishing |
| [Installation](https://openlogi.uxatom.com/docs/installation) | Packages, source builds, permission setup |
| [Troubleshooting](https://openlogi.uxatom.com/docs/troubleshooting) | Device not detected, buttons not responding |

## Contributing

Issues and pull requests are welcome. Useful bug reports include the output of
`pnpm probe`, which identifies the device, its connection path and its full
capability list.

CI runs `cargo fmt`, `clippy`, `cargo test` and the frontend suite on every
push, so running those locally first saves a round trip.

## Licence

GPL-3.0-or-later.

Not affiliated with or endorsed by Logitech. "Logitech", "Options+", "Unifying",
"Bolt" and "Lightspeed" are trademarks of Logitech International S.A.
