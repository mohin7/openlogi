//! `logi-probe watch` — prove that button diversion works on real hardware.
//!
//! Diversion is the mechanism every remapping feature depends on: we ask the
//! firmware to stop emitting a button's normal HID report and send us a
//! `0x1B04` notification instead. This mode diverts every divertable control,
//! prints the notifications as they arrive, and — importantly — puts
//! everything back.
//!
//! Restoring matters. A diverted button does nothing at all: no click reaches
//! the desktop. If we exited without un-diverting, the user's back/forward
//! buttons would stay dead until the mouse power-cycles. So restoration runs
//! from a `Drop` guard *and* from a signal handler.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use hidpp::features::controls::{self, Control, DivertedButtonEvent, RawXyEvent};
use hidpp::{hidpp1, ids, Device, Transport};

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

extern "C" fn on_signal(_: libc::c_int) {
    INTERRUPTED.store(true, Ordering::SeqCst);
}

fn install_signal_handler() {
    // SAFETY: `on_signal` only touches an atomic, which is async-signal-safe.
    unsafe {
        libc::signal(libc::SIGINT, on_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGTERM, on_signal as *const () as libc::sighandler_t);
    }
}

/// Un-diverts every control it was given, however the program exits.
struct DiversionGuard<'a> {
    diverted: Vec<u16>,
    dev: &'a mut Device,
    transport: &'a mut Transport,
}

impl Drop for DiversionGuard<'_> {
    fn drop(&mut self) {
        for cid in &self.diverted {
            let _ = controls::set_reporting(
                self.dev,
                self.transport,
                *cid,
                Some(false),
                None,
                Some(false),
                None,
            );
        }
        if !self.diverted.is_empty() {
            println!(
                "\n  restored {} control(s) to normal operation",
                self.diverted.len()
            );
        }
    }
}

pub fn watch(
    dev: &mut Device,
    transport: &mut Transport,
    seconds: u64,
    raw_xy: bool,
) -> hidpp::Result<()> {
    install_signal_handler();

    // Enable notification forwarding before diverting anything. A receiver
    // only forwards the notification classes enabled here, and SOFTWARE_PRESENT
    // tells the device that configuration software is listening.
    // Only the receiver is asked: a HID++ 2.0-only device has no 1.0
    // registers and answers InvalidFeatureIndex, which is expected rather
    // than a fault.
    match hidpp1::enable_notifications(transport, hidpp::protocol::RECEIVER_INDEX) {
        Ok(_) => {
            let got =
                hidpp1::notification_flags(transport, hidpp::protocol::RECEIVER_INDEX).unwrap_or(0);
            println!("  notifications enabled on receiver (flags 0x{got:06x})");
        }
        Err(e) => println!("  could not enable notifications: {e}"),
    }

    let all = controls::all(dev, transport)?;
    let feature_index = dev.require(transport, ids::REPROG_CONTROLS_V4)?.index;

    // Divert everything the firmware will let us divert.
    let targets: Vec<Control> = all.into_iter().filter(Control::is_divertable).collect();
    if targets.is_empty() {
        println!("  no divertable controls — remapping is not possible on this device");
        return Ok(());
    }

    let mut diverted = Vec::new();
    for c in &targets {
        // Virtual controls with FORCE_RAW_XY stream pointer deltas as fast as
        // the sensor reports, which drowns out button events — so raw XY is
        // opt-in, and only for the controls that actually support it.
        let want_raw = raw_xy && c.supports_gestures();
        match controls::set_reporting(
            dev,
            transport,
            c.cid,
            Some(true),
            None,
            want_raw.then_some(true),
            None,
        ) {
            Ok(()) => {
                println!("  diverted 0x{:04x} {}", c.cid, c.name());
                diverted.push(c.cid);
            }
            Err(e) => println!("  could NOT divert 0x{:04x} {}: {e}", c.cid, c.name()),
        }
    }

    // Read the state back. A successful write is not proof the firmware
    // actually entered the state we asked for, and without this a silent
    // no-op is indistinguishable from "the user pressed nothing".
    println!("\n  readback:");
    for cid in &diverted {
        match controls::reporting(dev, transport, *cid) {
            Ok(r) => println!(
                "    0x{:04x} {:<14} diverted={} rawXY={} remap=0x{:04x}  {DIM}{:?}{RESET}",
                r.cid,
                controls::control_name(r.cid),
                r.diverted(),
                r.raw_xy_diverted(),
                r.remapped_to,
                r.flags
            ),
            Err(e) => println!("    0x{cid:04x} readback failed: {e}"),
        }
    }

    println!("\n  Press the diverted buttons. Listening {seconds}s (Ctrl-C to stop early).\n");

    let guard = DiversionGuard {
        diverted,
        dev,
        transport,
    };
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let mut events = 0usize;
    let mut raw_count = 0usize;

    while Instant::now() < deadline && !INTERRUPTED.load(Ordering::SeqCst) {
        let pkt = match guard.transport.poll_event(Duration::from_millis(200)) {
            Ok(Some(p)) => p,
            Ok(None) => continue,
            Err(e) => {
                println!("  read error: {e}");
                break;
            }
        };
        events += 1;
        if pkt.feature_index() != feature_index {
            // Not a 0x1B04 event — still worth showing: battery and
            // connection notifications arrive on this same channel, and a
            // surprise here would otherwise be silently dropped.
            println!(
                "  · other notification: feature idx {} {pkt:?}",
                pkt.feature_index()
            );
            continue;
        }
        // For notifications the high nibble of byte 3 carries the event id.
        match pkt.function() {
            0 => {
                let ev = DivertedButtonEvent::parse(&pkt).unwrap_or(DivertedButtonEvent {
                    pressed: Vec::new(),
                });
                if ev.pressed.is_empty() {
                    println!("  ↑ all released");
                } else {
                    let names: Vec<String> = ev
                        .pressed
                        .iter()
                        .map(|c| format!("{} (0x{c:04x})", controls::control_name(*c)))
                        .collect();
                    println!("  ↓ {}", names.join(", "));
                }
            }
            1 => {
                // Rate-limit: the sensor can emit these faster than a terminal
                // can scroll, and one line per report tells us nothing extra.
                raw_count += 1;
                if raw_count % 25 == 1 {
                    if let Some(xy) = RawXyEvent::parse(&pkt) {
                        println!("  ~ raw XY  dx={:+5} dy={:+5}  (every 25th)", xy.dx, xy.dy);
                    }
                }
            }
            other => println!("  ? event {other}: {pkt:?}"),
        }
    }

    println!("\n  {events} notification(s) received");
    Ok(())
}

/// Verify the notification read path with a notification we generate
/// ourselves, so the test needs no human at the mouse.
///
/// Writing HID++ 1.0 register `0x02` asks the receiver to re-announce every
/// paired device, which produces a `0x41` device-connection notification. If
/// that arrives, reads and decoding work, and any absence of button events is
/// about the buttons — not about us.
pub fn selftest(dev: &mut Device, transport: &mut Transport) -> hidpp::Result<()> {
    use hidpp::protocol::RECEIVER_INDEX;

    println!("\n  notification read-path self-test");

    match hidpp1::enable_notifications(transport, RECEIVER_INDEX) {
        Ok(_) => {
            let got = hidpp1::notification_flags(transport, RECEIVER_INDEX).unwrap_or(0);
            println!("    notifications enabled (flags 0x{got:06x})");
        }
        Err(e) => println!("    enable failed: {e}"),
    }

    println!("    asking receiver to announce paired devices...");
    hidpp1::announce_devices(transport)?;

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut seen = 0usize;
    while Instant::now() < deadline {
        match transport.poll_event(Duration::from_millis(200)) {
            Ok(Some(pkt)) => {
                seen += 1;
                println!(
                    "    got: sub/feat 0x{:02x}  dev {}  {pkt:?}",
                    pkt.feature_index(),
                    pkt.device_index()
                );
            }
            Ok(None) => {}
            Err(e) => {
                println!("    read error: {e}");
                break;
            }
        }
    }

    if seen == 0 {
        println!("\n    RESULT: no notifications arrived — the read path is NOT working.");
    } else {
        println!("\n    RESULT: {seen} notification(s) read. The read path works.");
        println!("    So zero button events means the diverted buttons were not pressed.");
    }
    let _ = dev;
    Ok(())
}
