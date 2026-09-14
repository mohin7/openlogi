//! `logi-probe` — the ground-truth tool.
//!
//! Before writing UI for a capability we ask the hardware whether it has it.
//! This binary walks every Logitech HID++ endpoint, every paired device, and
//! every feature, and prints what it finds. Its output is the input to the
//! device-capability model the desktop app renders.
//!
//! Run with `RUST_LOG=hidpp=trace` to see the raw packets on the wire.

use std::process::ExitCode;

use hidpp::features::{battery, controls, dpi, info};
use hidpp::{ids, Device, Error, Transport};

mod watch;

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .with_target(false)
        .without_time()
        .init();

    match run() {
        Ok(found) if found => ExitCode::SUCCESS,
        Ok(_) => {
            eprintln!("{YELLOW}No Logitech device answered.{RESET}");
            eprintln!("  • Is the mouse switched on and in range?");
            eprintln!("  • Is it paired to this receiver (or connected over Bluetooth)?");
            ExitCode::from(2)
        }
        Err(e) => {
            eprintln!("{RED}error:{RESET} {e}");
            let mut source = std::error::Error::source(&e);
            while let Some(s) = source {
                eprintln!("  caused by: {s}");
                source = s.source();
            }
            if matches!(e, Error::Permission { .. }) {
                eprintln!("\n  Run: sudo ./scripts/install-udev-rules.sh");
            }
            ExitCode::FAILURE
        }
    }
}

/// `logi-probe` dumps capabilities; `logi-probe watch [seconds]` proves that
/// button diversion works by diverting every divertable control and printing
/// the notifications that come back.
enum Mode {
    Dump,
    Watch {
        seconds: u64,
        raw_xy: bool,
    },
    /// Self-test the notification read path without needing a human.
    SelfTest,
}

fn parse_args() -> Mode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("selftest") => Mode::SelfTest,
        Some("watch") => Mode::Watch {
            seconds: args.get(1).and_then(|s| s.parse().ok()).unwrap_or(20),
            raw_xy: args.iter().any(|a| a == "--raw"),
        },
        _ => Mode::Dump,
    }
}

fn run() -> hidpp::Result<bool> {
    let mode = parse_args();
    let api = hidapi::HidApi::new()?;
    let endpoints = hidpp::discover(&api);

    if endpoints.is_empty() {
        eprintln!("{YELLOW}No Logitech HID++ interface found.{RESET}");
        eprintln!("  Looked for VID 046d with HID usage page 0xFF00.");
        return Ok(false);
    }

    let mut any_device = false;

    for endpoint in endpoints {
        println!(
            "\n{BOLD}◆ {} {}{RESET}",
            endpoint.product.as_deref().unwrap_or("Logitech device"),
            if endpoint.is_receiver {
                "(receiver)"
            } else {
                ""
            }
        );
        println!(
            "  {DIM}{:04x}:{:04x}  iface {}  {}{RESET}",
            endpoint.vendor_id, endpoint.product_id, endpoint.interface_number, endpoint.path
        );

        let mut transport = match Transport::open(&api, endpoint.clone()) {
            Ok(t) => t,
            Err(e) => {
                println!("  {RED}cannot open: {e}{RESET}");
                continue;
            }
        };

        // A receiver fronts up to six devices; a direct connection answers
        // only on the fixed index 0xFF.
        let indices: Vec<u8> = if endpoint.is_receiver {
            (1..=hidpp::MAX_PAIRED_DEVICES).collect()
        } else {
            vec![hidpp::protocol::DIRECT_INDEX]
        };

        for index in indices {
            match Device::probe(&mut transport, index) {
                Ok(Some(mut dev)) => {
                    any_device = true;
                    match mode {
                        Mode::Dump => report(&mut dev, &mut transport, index),
                        Mode::SelfTest => {
                            watch::selftest(&mut dev, &mut transport)?;
                            return Ok(true);
                        }
                        Mode::Watch { seconds, raw_xy } => {
                            let name = info::name(&mut dev, &mut transport)
                                .unwrap_or_else(|_| "device".into());
                            println!("\n  {GREEN}▸ [{index}] {BOLD}{name}{RESET}");
                            watch::watch(&mut dev, &mut transport, seconds, raw_xy)?;
                            return Ok(true);
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => println!("  {DIM}device {index}: {e}{RESET}"),
            }
        }
    }

    Ok(any_device)
}

fn report(dev: &mut Device, t: &mut Transport, index: u8) {
    let name = info::name(dev, t).unwrap_or_else(|_| "Unknown device".into());
    let kind = info::kind(dev, t)
        .map(|k| k.to_string())
        .unwrap_or_else(|_| "?".into());

    println!("\n  {GREEN}▸ [{index}] {BOLD}{name}{RESET}{GREEN} — {kind}{RESET}");
    println!("    protocol      HID++ {}", dev.protocol());

    // Prefer 0x0003 v4, which many devices carry even without 0x0004.
    match info::device_info(dev, t) {
        Ok(Some(di)) => {
            let mut transports = Vec::new();
            if di.supports_bluetooth() {
                transports.push("BT");
            }
            if di.supports_ble() {
                transports.push("BLE");
            }
            if di.supports_equad() {
                transports.push("eQuad");
            }
            if di.supports_usb() {
                transports.push("USB");
            }
            println!(
                "    unit id       {}  {DIM}(profile key, via 0x0003 v4){RESET}",
                di.unit_id_hex()
            );
            println!(
                "    transports    {}  {DIM}model ids {}{RESET}",
                transports.join(", "),
                di.model_ids
                    .iter()
                    .map(|m| format!("{m:04X}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        _ => match info::unit_id(dev, t) {
            Ok(uid) => println!("    unit id       {uid}  {DIM}(profile key, via 0x0004){RESET}"),
            Err(_) => println!("    unit id       {DIM}unavailable{RESET}"),
        },
    }

    match info::firmware(dev, t) {
        Ok(entities) => {
            for fw in entities {
                println!("    firmware      {fw}  {DIM}({:?}){RESET}", fw.kind);
            }
        }
        Err(e) => println!("    firmware      {DIM}unavailable: {e}{RESET}"),
    }

    match battery::read(dev, t) {
        Ok(b) => {
            let pct = b
                .approx_percentage()
                .map(|p| format!("{p}%"))
                .unwrap_or_else(|| "—".into());
            println!(
                "    battery       {pct}  {:?}{}  {DIM}via {} (0x{:04x}){RESET}",
                b.state,
                b.voltage_mv.map(|v| format!("  {v}mV")).unwrap_or_default(),
                ids::feature_name(b.source),
                b.source
            );
        }
        Err(e) => println!("    battery       {DIM}unavailable: {e}{RESET}"),
    }

    // DPI
    match dpi::sensor_count(dev, t) {
        Ok(n) => {
            for s in 0..n {
                match dpi::get(dev, t, s) {
                    Ok(d) => {
                        let values = d.range.values();
                        println!(
                            "    dpi sensor {s}  current {} (default {})  range {}–{}  {} steps",
                            d.current,
                            d.default,
                            d.range.min(),
                            d.range.max(),
                            values.len()
                        );
                    }
                    Err(e) => println!("    dpi sensor {s}  {DIM}{e}{RESET}"),
                }
            }
        }
        Err(_) => println!("    dpi           {DIM}not adjustable over HID++{RESET}"),
    }

    // Controls — the remapping surface.
    match controls::all(dev, t) {
        Ok(list) => {
            println!("    {BOLD}controls ({}){RESET}", list.len());
            for c in &list {
                println!(
                    "      cid 0x{:04x}  task 0x{:04x}  pos {}  grp {}/{:02x}  {:<20} {DIM}{:?} | {:?}{RESET}",
                    c.cid,
                    c.task_id,
                    c.position,
                    c.group,
                    c.group_mask,
                    c.name(),
                    c.flags,
                    c.extra
                );
            }
        }
        Err(_) => println!("    controls      {DIM}feature 0x1B04 unsupported{RESET}"),
    }

    // Full capability dump — this is what the UI gates screens on.
    match dev.enumerate_features(t) {
        Ok(features) => {
            println!("    {BOLD}features ({}){RESET}", features.len());
            for f in features {
                // Hidden/engineering features are still listed: undocumented
                // capabilities live there, and a gap in the index sequence is
                // itself a diagnostic signal worth seeing.
                let mark = match (f.hidden, f.engineering, f.obsolete) {
                    (true, _, _) => " [hidden]",
                    (_, true, _) => " [engineering]",
                    (_, _, true) => " [obsolete]",
                    _ => "",
                };
                println!(
                    "      0x{:04x} v{}  idx {:<3} {}{DIM}{mark}{RESET}",
                    f.id,
                    f.version,
                    f.index,
                    ids::feature_name(f.id)
                );
            }
        }
        Err(e) => println!("    features      {DIM}{e}{RESET}"),
    }
}
