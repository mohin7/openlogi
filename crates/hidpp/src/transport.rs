//! Finding and talking to the HID++ endpoint on Linux.
//!
//! A Logitech receiver exposes several HID interfaces: a boot keyboard, a boot
//! mouse, and a vendor-defined one carrying HID++. Matching on interface
//! number is fragile (it differs between Bolt, Unifying, direct-USB and
//! Bluetooth), so we identify the endpoint by its *report descriptor shape*
//! instead: usage page `0xFF00`, with report ids `0x10` and/or `0x11`.
//! `hidapi` surfaces the parsed usage page for us, which is exactly the
//! discriminator we want.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use hidapi::{HidApi, HidDevice};
use tracing::{debug, trace};

use crate::error::{Error, Result};
use crate::protocol::Packet;

pub const LOGITECH_VID: u16 = 0x046D;

/// Vendor-defined usage page used by every HID++ transport we support.
const HIDPP_USAGE_PAGE: u16 = 0xFF00;

/// How long to wait for a reply before declaring the device unreachable.
/// Sleeping wireless devices legitimately take a few hundred ms to answer.
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(1200);

/// A discovered HID++ endpoint, before it is opened.
#[derive(Debug, Clone)]
pub struct Endpoint {
    pub path: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface_number: i32,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    /// `true` when the product id looks like a receiver rather than a device
    /// attached directly over USB or Bluetooth.
    pub is_receiver: bool,
}

/// Product-id ranges that identify Logitech receivers.
/// Unifying: 0xC52B/0xC532/0xC534..., Bolt: 0xC548, Lightspeed: 0xC539 etc.
fn looks_like_receiver(pid: u16) -> bool {
    matches!(
        pid,
        0xC52B | 0xC532 | 0xC534 | 0xC539 | 0xC53A | 0xC53D | 0xC53F | 0xC542 | 0xC547 | 0xC548
    )
}

/// Enumerate every Logitech HID++ endpoint currently present.
///
/// The HID++ collection declares two top-level usages on the same interface
/// (`0xFF00:0x01` for short reports, `0xFF00:0x02` for long ones), so a
/// descriptor-parsing backend reports the same `/dev/hidraw` node more than
/// once. We deduplicate by device path — one node is one channel.
pub fn discover(api: &HidApi) -> Vec<Endpoint> {
    let mut found: Vec<Endpoint> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for info in api.device_list() {
        if info.vendor_id() != LOGITECH_VID {
            continue;
        }
        if info.usage_page() != HIDPP_USAGE_PAGE {
            continue;
        }
        let path = info.path().to_string_lossy().into_owned();
        if !seen.insert(path.clone()) {
            continue;
        }
        let pid = info.product_id();
        debug!(%path, pid = format!("{pid:04x}"), "found HID++ endpoint");
        found.push(Endpoint {
            path,
            vendor_id: info.vendor_id(),
            product_id: pid,
            interface_number: info.interface_number(),
            manufacturer: info.manufacturer_string().map(str::to_owned),
            product: info.product_string().map(str::to_owned),
            is_receiver: looks_like_receiver(pid),
        });
    }

    // Receivers first: they front the most devices, so probing them first
    // gets something on screen soonest.
    found.sort_by_key(|e| (!e.is_receiver, e.path.clone()));
    found
}

/// An opened HID++ channel. One channel may front up to six paired devices,
/// distinguished by the device index byte.
pub struct Transport {
    handle: HidDevice,
    endpoint: Endpoint,
    timeout: Duration,
    /// Unsolicited reports (battery changes, button events, pairing
    /// notifications) received while waiting for a reply. Callers drain this
    /// to drive the event layer instead of losing the notifications.
    pending_events: VecDeque<Packet>,
}

impl Transport {
    pub fn open(api: &HidApi, endpoint: Endpoint) -> Result<Self> {
        let handle = api
            .open_path(
                &std::ffi::CString::new(endpoint.path.clone())
                    .map_err(|_| Error::Malformed("device path contains NUL"))?,
            )
            .map_err(|e| match e {
                hidapi::HidError::HidApiError { message } if message.contains("Permission") => {
                    Error::Permission {
                        path: endpoint.path.clone(),
                    }
                }
                other => Error::Hid(other),
            })?;

        Ok(Self {
            handle,
            endpoint,
            timeout: DEFAULT_TIMEOUT,
            pending_events: VecDeque::new(),
        })
    }

    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Send a request and wait for the matching reply.
    ///
    /// Reports that are not a reply to *this* request are either HID++
    /// notifications (software id 0) or replies destined for another program;
    /// notifications are queued, anything else is discarded.
    pub fn request(&mut self, req: Packet) -> Result<Packet> {
        trace!(?req, "-> tx");
        self.handle.write(req.as_bytes())?;

        let deadline = Instant::now() + self.timeout;
        let mut buf = [0u8; crate::protocol::MAX_LEN];

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(Error::Timeout {
                    device_index: req.device_index(),
                });
            }

            let n = self
                .handle
                .read_timeout(&mut buf, remaining.as_millis().min(i32::MAX as u128) as i32)?;
            if n == 0 {
                continue; // spurious wakeup; the deadline check above ends the loop
            }

            let Some(pkt) = Packet::parse(&buf[..n]) else {
                continue; // not a HID++ report
            };
            trace!(?pkt, "<- rx");

            if pkt.answers(&req) {
                return Ok(pkt);
            }
            // Software id 0 marks a device-initiated notification. The
            // queue is bounded: a device that spams notifications must not be
            // able to grow our memory without limit.
            if pkt.software_id() == 0 && self.pending_events.len() < 64 {
                self.pending_events.push_back(pkt);
            }
        }
    }

    /// Poll for a device-initiated notification without sending anything.
    pub fn poll_event(&mut self, timeout: Duration) -> Result<Option<Packet>> {
        if let Some(pkt) = self.pending_events.pop_front() {
            return Ok(Some(pkt));
        }
        let mut buf = [0u8; crate::protocol::MAX_LEN];
        let n = self
            .handle
            .read_timeout(&mut buf, timeout.as_millis().min(i32::MAX as u128) as i32)?;
        if n == 0 {
            return Ok(None);
        }
        // No software-id filter here, deliberately. Nothing is outstanding
        // while polling, so every HID++ report is by definition unsolicited.
        // Filtering on `software_id() == 0` would also be wrong: that byte is
        // only a software id in HID++ 2.0. In 1.0 notifications (sub-ids
        // 0x40..0x4F) the same byte carries data, so the filter discarded
        // connection and pairing events whenever their low nibble was set.
        Ok(Packet::parse(&buf[..n]))
    }

    pub fn drain_events(&mut self) -> Vec<Packet> {
        self.pending_events.drain(..).collect()
    }
}
