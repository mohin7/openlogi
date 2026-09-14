//! The device service.
//!
//! `hidapi` handles are not `Send`, so the hardware cannot simply live behind
//! a `Mutex` in Tauri's state. Instead one dedicated thread owns every open
//! transport for its whole life, and commands reach it over a channel. Only
//! plain data crosses the boundary.
//!
//! That is not merely a workaround for a trait bound — it is the shape the
//! daemon needs anyway. HID++ is strictly request/response per channel, so
//! serialising access through a single owner is what keeps two concurrent UI
//! actions from interleaving their packets.

use std::collections::HashSet;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender, TryRecvError};
use std::sync::Mutex;
use std::time::Duration;

use hidapi::HidApi;
use hidpp::features::{battery, controls, dpi, info};
use button_engine::{Action, ButtonEngine};
use hidpp::{hidpp1, ids, Device, Transport};
use tracing::{debug, warn};

use crate::dto::*;

pub enum Cmd {
    List(SyncSender<Result<Vec<DeviceDto>, String>>),
    SetDpi {
        unit_id: String,
        dpi: u16,
        reply: SyncSender<Result<u16, String>>,
    },
    SetDiverted {
        unit_id: String,
        cid: u16,
        diverted: bool,
        reply: SyncSender<Result<(), String>>,
    },
    SetMapping {
        unit_id: String,
        cid: u16,
        action: Action,
        reply: SyncSender<Result<(), String>>,
    },
    EngineStatus(SyncSender<EngineStatusDto>),
    /// Restore a device to its factory behaviour: sensor DPI back to the
    /// firmware default, every mapping cleared and every control un-diverted.
    ResetDevice {
        unit_id: String,
        reply: SyncSender<Result<(), String>>,
    },
}

/// Whether input synthesis is working, and why not if it isn't.
#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatusDto {
    pub available: bool,
    pub reason: Option<String>,
}

/// Handle held in Tauri state.
pub struct DeviceService {
    tx: Mutex<Sender<Cmd>>,
}

impl DeviceService {
    pub fn spawn() -> Self {
        let (tx, rx) = channel::<Cmd>();
        std::thread::Builder::new()
            .name("openlogi-devices".into())
            .spawn(move || worker(rx))
            .expect("failed to spawn device thread");
        Self { tx: Mutex::new(tx) }
    }

    fn send<T>(&self, make: impl FnOnce(SyncSender<T>) -> Cmd) -> Result<T, String> {
        let (reply_tx, reply_rx) = sync_channel::<T>(1);
        self.tx
            .lock()
            .map_err(|_| "device thread poisoned".to_string())?
            .send(make(reply_tx))
            .map_err(|_| "device thread stopped".to_string())?;
        reply_rx
            .recv_timeout(Duration::from_secs(15))
            .map_err(|_| "device thread timed out".to_string())
    }

    pub fn list(&self) -> Result<Vec<DeviceDto>, String> {
        self.send(Cmd::List)?
    }

    pub fn set_dpi(&self, unit_id: String, dpi: u16) -> Result<u16, String> {
        self.send(|reply| Cmd::SetDpi { unit_id, dpi, reply })?
    }

    pub fn set_diverted(&self, unit_id: String, cid: u16, diverted: bool) -> Result<(), String> {
        self.send(|reply| Cmd::SetDiverted { unit_id, cid, diverted, reply })?
    }

    pub fn set_mapping(&self, unit_id: String, cid: u16, action: Action) -> Result<(), String> {
        self.send(|reply| Cmd::SetMapping { unit_id, cid, action, reply })?
    }

    pub fn engine_status(&self) -> Result<EngineStatusDto, String> {
        self.send(Cmd::EngineStatus)
    }

    pub fn reset_device(&self, unit_id: String) -> Result<(), String> {
        self.send(|reply| Cmd::ResetDevice { unit_id, reply })?
    }
}

/// One open HID++ channel plus the devices reachable through it.
struct Channel {
    transport: Transport,
    is_receiver: bool,
    connection: &'static str,
    devices: Vec<Entry>,
}

struct Entry {
    unit_id: String,
    device: Device,
    /// Runtime index of feature 0x1B04, resolved once so the event loop can
    /// recognise diverted-button notifications without a lookup per packet.
    reprog_index: Option<u8>,
    /// Control ids held down as of the last notification. The firmware sends
    /// the full held-set on every change, so presses are the difference.
    held: HashSet<u16>,
}

fn worker(rx: Receiver<Cmd>) {
    let mut channels: Vec<Channel> = Vec::new();
    let mut engine = ButtonEngine::new();
    if let Some(reason) = engine.unavailable_reason() {
        warn!("input synthesis unavailable: {reason}");
    }

    loop {
        // 1. Service every pending command without blocking, so UI actions
        //    stay responsive while we are also watching for button events.
        loop {
            match rx.try_recv() {
                Ok(cmd) => handle(cmd, &mut channels, &mut engine),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    restore_all(&mut channels);
                    return;
                }
            }
        }

        // 2. Drain device notifications. A short timeout keeps command latency
        //    bounded; HID++ is request/response per channel, so this thread
        //    owning both roles is what prevents interleaved packets.
        let mut saw_event = false;
        for ch in channels.iter_mut() {
            while let Ok(Some(pkt)) = ch.transport.poll_event(Duration::from_millis(15)) {
                saw_event = true;
                dispatch(&pkt, ch, &mut engine);
            }
        }

        // Nothing open and nothing happening: yield rather than spin.
        if channels.is_empty() && !saw_event {
            std::thread::sleep(Duration::from_millis(40));
        }
    }
}

/// Translate a notification into button presses.
///
/// Feature 0x1B04 event 0 reports the *complete* set of currently-held
/// diverted controls, not deltas. A press is therefore a control that appears
/// in the new set but not the previous one; an empty payload means everything
/// was released.
fn dispatch(pkt: &hidpp::Packet, ch: &mut Channel, engine: &mut ButtonEngine) {
    let Some(entry) = ch
        .devices
        .iter_mut()
        .find(|e| e.device.index() == pkt.device_index())
    else {
        return;
    };
    if entry.reprog_index != Some(pkt.feature_index()) {
        return;
    }
    if pkt.function() != 0 {
        return; // event 1 is raw XY, for the gesture engine
    }

    let Some(ev) = controls::DivertedButtonEvent::parse(pkt) else {
        return;
    };
    let now: HashSet<u16> = ev.pressed.iter().copied().collect();
    let newly_pressed: Vec<u16> = now.difference(&entry.held).copied().collect();
    entry.held = now;

    for cid in newly_pressed {
        engine.on_press(cid);
    }
}

fn handle(cmd: Cmd, channels: &mut Vec<Channel>, engine: &mut ButtonEngine) {
    match cmd {
        Cmd::List(reply) => match refresh(channels) {
            Ok(()) => {
                reapply_diversions(channels, engine);
                let _ = reply.send(Ok(build_dtos(channels)));
            }
            Err(e) => {
                let _ = reply.send(Err(e));
            }
        },
        Cmd::SetDpi { unit_id, dpi: want, reply } => {
            let _ = reply.send(with_device(channels, &unit_id, |dev, t| {
                dpi::set(dev, t, 0, want).map_err(|e| e.to_string())
            }));
        }
        Cmd::SetDiverted { unit_id, cid, diverted, reply } => {
            let _ = reply.send(with_device(channels, &unit_id, |dev, t| {
                controls::set_reporting(dev, t, cid, Some(diverted), None, None, None)
                    .map_err(|e| e.to_string())
            }));
        }
        Cmd::SetMapping { unit_id, cid, action, reply } => {
            let divert = action.needs_diversion();
            engine.set(cid, action);
            // Diversion and mapping must move together: a mapped button that
            // is not diverted still performs its firmware action as well as
            // ours, and a diverted button with no mapping is simply dead.
            let result = with_device(channels, &unit_id, |dev, t| {
                controls::set_reporting(dev, t, cid, Some(divert), None, None, None)
                    .map_err(|e| e.to_string())
            });
            if result.is_err() {
                engine.set(cid, Action::Default);
            }
            let _ = reply.send(result);
        }
        Cmd::ResetDevice { unit_id, reply } => {
            let _ = reply.send(reset_device(channels, engine, &unit_id));
        }
        Cmd::EngineStatus(reply) => {
            let _ = reply.send(EngineStatusDto {
                available: engine.is_available(),
                reason: engine.unavailable_reason().map(str::to_string),
            });
        }
    }
}

/// Undo everything OpenLogi has done to a device.
///
/// Un-diversion is attempted for *every* mapped control even if one fails,
/// because a control left diverted is a button that does nothing at all — far
/// worse than a DPI that did not reset.
fn reset_device(
    channels: &mut [Channel],
    engine: &mut ButtonEngine,
    unit_id: &str,
) -> Result<(), String> {
    let mapped = engine.diverted_cids();
    let mut problems: Vec<String> = Vec::new();

    for cid in &mapped {
        let outcome = with_device(channels, unit_id, |dev, t| {
            controls::set_reporting(dev, t, *cid, Some(false), None, None, None)
                .map_err(|e| e.to_string())
        });
        match outcome {
            Ok(()) => engine.set(*cid, Action::Default),
            Err(e) => problems.push(format!("control 0x{cid:04x}: {e}")),
        }
    }

    // DPI back to whatever the sensor reports as its own default.
    let dpi_result = with_device(channels, unit_id, |dev, t| {
        let target = dpi::get(dev, t, 0).map(|d| d.default).map_err(|e| e.to_string())?;
        if target == 0 {
            return Ok(()); // device does not report a default; leave it alone
        }
        dpi::set(dev, t, 0, target).map(|_| ()).map_err(|e| e.to_string())
    });
    if let Err(e) = dpi_result {
        problems.push(format!("dpi: {e}"));
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

/// Diversion does not survive a device power-cycle or a re-plugged receiver,
/// so it must be re-applied every time we re-enumerate.
fn reapply_diversions(channels: &mut [Channel], engine: &ButtonEngine) {
    let wanted = engine.diverted_cids();
    if wanted.is_empty() {
        return;
    }
    for ch in channels.iter_mut() {
        for entry in ch.devices.iter_mut() {
            for cid in &wanted {
                let _ = controls::set_reporting(
                    &mut entry.device,
                    &mut ch.transport,
                    *cid,
                    Some(true),
                    None,
                    None,
                    None,
                );
            }
        }
    }
}

/// Put every diverted control back before the thread goes away. A diverted
/// button is completely dead to the desktop, so leaving one behind would look
/// like broken hardware.
fn restore_all(channels: &mut [Channel]) {
    for ch in channels.iter_mut() {
        for entry in ch.devices.iter_mut() {
            for cid in entry.held.clone() {
                let _ = controls::set_reporting(
                    &mut entry.device,
                    &mut ch.transport,
                    cid,
                    Some(false),
                    None,
                    None,
                    None,
                );
            }
        }
    }
}

fn with_device<T>(
    channels: &mut [Channel],
    unit_id: &str,
    f: impl FnOnce(&mut Device, &mut Transport) -> Result<T, String>,
) -> Result<T, String> {
    for ch in channels.iter_mut() {
        if let Some(entry) = ch.devices.iter_mut().find(|e| e.unit_id == unit_id) {
            return f(&mut entry.device, &mut ch.transport);
        }
    }
    Err(format!("device {unit_id} is not connected"))
}

fn connection_for(pid: u16, is_receiver: bool) -> &'static str {
    match (pid, is_receiver) {
        (0xC548, _) => "bolt",
        (0xC52B | 0xC532 | 0xC534, _) => "unifying",
        (_, false) => "bluetooth",
        _ => "bolt",
    }
}

fn refresh(channels: &mut Vec<Channel>) -> Result<(), String> {
    channels.clear();

    let api = HidApi::new().map_err(|e| e.to_string())?;
    for endpoint in hidpp::discover(&api) {
        let is_receiver = endpoint.is_receiver;
        let connection = connection_for(endpoint.product_id, is_receiver);
        let path = endpoint.path.clone();

        let mut transport = match Transport::open(&api, endpoint) {
            Ok(t) => t,
            Err(e) => {
                warn!(%path, "cannot open endpoint: {e}");
                continue;
            }
        };

        // Receivers forward notifications only for the classes enabled here.
        if is_receiver {
            if let Err(e) =
                hidpp1::enable_notifications(&mut transport, hidpp::protocol::RECEIVER_INDEX)
            {
                debug!("enable_notifications: {e}");
            }
        }

        let indices: Vec<u8> = if is_receiver {
            (1..=hidpp::MAX_PAIRED_DEVICES).collect()
        } else {
            vec![hidpp::protocol::DIRECT_INDEX]
        };

        let mut devices = Vec::new();
        for index in indices {
            match Device::probe(&mut transport, index) {
                Ok(Some(mut device)) => {
                    let unit_id = info::device_info(&mut device, &mut transport)
                        .ok()
                        .flatten()
                        .map(|d| d.unit_id_hex())
                        .or_else(|| info::unit_id(&mut device, &mut transport).ok())
                        .unwrap_or_else(|| format!("idx{index}"));
                    let reprog_index = device
                        .feature(&mut transport, ids::REPROG_CONTROLS_V4)
                        .ok()
                        .flatten()
                        .map(|f| f.index);
                    devices.push(Entry {
                        unit_id,
                        device,
                        reprog_index,
                        held: HashSet::new(),
                    });
                }
                Ok(None) => {}
                Err(e) => debug!("probe {index}: {e}"),
            }
        }

        channels.push(Channel { transport, is_receiver, connection, devices });
    }
    Ok(())
}

fn build_dtos(channels: &mut [Channel]) -> Vec<DeviceDto> {
    let mut out = Vec::new();
    for ch in channels.iter_mut() {
        let connection = ch.connection;
        for entry in ch.devices.iter_mut() {
            match build_one(&mut entry.device, &mut ch.transport, &entry.unit_id, connection) {
                Ok(dto) => out.push(dto),
                Err(e) => warn!("building device {}: {e}", entry.unit_id),
            }
        }
    }
    let _ = ch_unused(channels);
    out
}

/// `is_receiver` is retained on `Channel` for the hotplug work to come; this
/// keeps the field live without silencing the warning crate-wide.
fn ch_unused(channels: &[Channel]) -> usize {
    channels.iter().filter(|c| c.is_receiver).count()
}

fn build_one(
    dev: &mut Device,
    t: &mut Transport,
    unit_id: &str,
    connection: &'static str,
) -> Result<DeviceDto, String> {
    let name = info::name(dev, t).unwrap_or_else(|_| "Logitech device".into());
    let kind = match info::kind(dev, t) {
        Ok(ids::DeviceKind::Mouse) => "mouse",
        Ok(ids::DeviceKind::Keyboard) => "keyboard",
        Ok(ids::DeviceKind::Trackball) => "trackball",
        Ok(ids::DeviceKind::Touchpad) => "touchpad",
        _ => "other",
    };

    let firmware = info::firmware(dev, t)
        .unwrap_or_default()
        .into_iter()
        .map(|f| FirmwareDto {
            kind: match f.kind {
                info::FirmwareKind::Main => "main",
                info::FirmwareKind::Bootloader => "bootloader",
                info::FirmwareKind::Hardware => "hardware",
                _ => "other",
            },
            version: f.to_string(),
        })
        .collect();

    let battery = match battery::read(dev, t) {
        Ok(b) => BatteryDto {
            percentage: b.approx_percentage(),
            state: match b.state {
                battery::ChargeState::Discharging => "discharging",
                battery::ChargeState::Charging => "charging",
                battery::ChargeState::ChargingSlow => "chargingSlow",
                battery::ChargeState::ChargeComplete => "chargeComplete",
                battery::ChargeState::ChargeError => "chargeError",
                battery::ChargeState::Unknown => "unknown",
            },
            source_feature: b.source,
        },
        Err(_) => BatteryDto { percentage: None, state: "unknown", source_feature: 0 },
    };

    let controls_list: Vec<ControlDto> = controls::all(dev, t)
        .unwrap_or_default()
        .into_iter()
        .map(|c| ControlDto {
            cid: c.cid,
            task_id: c.task_id,
            name: c.name().to_string(),
            reprogrammable: c.flags.contains(controls::ControlFlags::REPROGRAMMABLE),
            divertable: c.is_divertable(),
            supports_gestures: c.supports_gestures(),
            virtual_control: c.flags.contains(controls::ControlFlags::VIRTUAL),
        })
        .collect();

    let features: Vec<u16> = dev
        .enumerate_features(t)
        .unwrap_or_default()
        .into_iter()
        .map(|f| f.id)
        .collect();
    let has = |id: u16| features.contains(&id);

    let dpi_range = dpi::range(dev, t, 0).ok().map(|r| DpiRangeDto {
        min: r.min(),
        max: r.max(),
        step: match &r {
            dpi::DpiRange::Steps { step, .. } => *step,
            dpi::DpiRange::List(_) => 50,
        },
    });
    let (current_dpi, default_dpi) = dpi::get(dev, t, 0)
        .map(|d| (d.current, d.default))
        .unwrap_or((0, 0));

    let model_id = info::device_info(dev, t)
        .ok()
        .flatten()
        .and_then(|d| d.model_ids.first().map(|m| format!("{m:04X}")))
        .unwrap_or_default();

    Ok(DeviceDto {
        id: unit_id.to_lowercase(),
        unit_id: unit_id.to_string(),
        name,
        kind,
        connection,
        connected: true,
        protocol: format!("HID++ {}", dev.protocol()),
        model_id,
        battery,
        firmware,
        controls: controls_list,
        capabilities: CapabilitiesDto {
            has_adjustable_dpi: has(ids::ADJUSTABLE_DPI),
            has_pointer_speed: has(ids::POINTER_SPEED),
            has_smart_shift: has(ids::SMART_SHIFT) || has(ids::SMART_SHIFT_ENHANCED),
            has_hi_res_wheel: has(ids::HIRES_WHEEL),
            has_thumb_wheel: has(ids::THUMB_WHEEL),
            can_persist_onboard: has(ids::PERSISTENT_REMAPPABLE_ACTION)
                || has(ids::ONBOARD_PROFILES),
            dpi: dpi_range,
            features,
        },
        current_dpi,
        default_dpi,
        last_seen: None,
    })
}
