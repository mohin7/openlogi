# OpenLogi

A Linux alternative to Logitech Options+ — configure Logitech mice and
keyboards natively, without proprietary software, and without `sudo`.

> Status: **early.** The HID++ protocol layer, the hardware probe and the
> desktop app are working. Profile persistence and per-application profile
> switching are designed but not yet implemented.

## Quick start

```bash
git clone <repo> && cd openlogi
./scripts/setup.sh          # asks for sudo once: build deps + udev rules
./target/release/logi-probe # dump everything about your hardware
```

## What works today

- `crates/hidpp` — HID++ 1.0/2.0 implementation over Linux `hidraw`
  - endpoint discovery by report-descriptor shape (works across Bolt,
    Unifying, direct USB and Bluetooth)
  - request/reply matching by software id, so it coexists with Solaar
  - runtime feature discovery
  - battery (`0x1000`/`0x1001`/`0x1004`), DPI (`0x2201`),
    reprogrammable controls (`0x1B04`), device info (`0x0003`/`0x0005`)
- `crates/logi-probe` — prints your device's real capability list

## Documentation

- [Architecture](docs/architecture.md)
- [Releasing](docs/releasing.md)

## Licence

GPL-3.0-or-later.
