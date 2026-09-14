//! Feature `0x1B04` — Reprogrammable Controls v4.
//!
//! This is the feature that makes remapping possible at all.
//!
//! Every physical control has a stable **Control ID (CID)** and a **Task ID**
//! (the action the firmware performs by default). Two mechanisms exist:
//!
//! * **Remapping** — point a CID at a different task id. Handled entirely in
//!   firmware, survives reboots, but you can only choose from tasks Logitech
//!   defined.
//! * **Diversion** — tell the firmware "stop emitting this control's normal
//!   HID report; send me a `0x1B04` notification instead". The button then
//!   does nothing on its own, and OpenLogi decides what happens, synthesising
//!   input through `uinput`. This is how arbitrary shortcuts, macros, shell
//!   commands and gestures are implemented.
//!
//! Only controls whose info flags advertise `DIVERTABLE` may be diverted. A
//! diverted control stays diverted until it is un-diverted or the device
//! power-cycles, so the daemon must re-apply diversion on every reconnect.

use crate::device::Device;
use crate::error::Result;
use crate::ids;
use crate::protocol::{at, be16, Packet};
use crate::transport::Transport;

bitflags_lite! {
    /// Capability flags from `getControlInfo` byte 4.
    pub struct ControlFlags: u8 {
        const MOUSE_BUTTON          = 0x01;
        const FKEY                  = 0x02;
        const HOTKEY                = 0x04;
        const FN_TOGGLE             = 0x08;
        const REPROGRAMMABLE        = 0x10;
        const DIVERTABLE            = 0x20;
        const PERSISTENTLY_DIVERTABLE = 0x40;
        const VIRTUAL               = 0x80;
    }
}

bitflags_lite! {
    /// Extra flags from `getControlInfo` byte 8.
    pub struct ExtraFlags: u8 {
        /// The control can stream raw XY movement while held — the
        /// prerequisite for gesture support.
        const RAW_XY        = 0x01;
        const FORCE_RAW_XY  = 0x02;
        const ANALYTICS_KEY = 0x04;
    }
}

bitflags_lite! {
    /// Mapping state of a control.
    ///
    /// These are the *value* bits, used both by `getControlReporting` replies
    /// and as the values written by `setControlReporting`.
    ///
    /// The write encoding has a quirk worth stating plainly, because getting
    /// it wrong produces a misleading `InvalidArgument` rather than an obvious
    /// failure: **every value bit is paired with a "valid" bit sitting one
    /// position to its left** (`valid == value << 1`). A field is only applied
    /// when its valid bit is set, which is what lets a caller change diversion
    /// without disturbing an existing remap. So requesting plain diversion is
    /// `DIVERTED | (DIVERTED << 1)` = `0x03`, *not* `0x01`.
    pub struct MappingFlags: u16 {
        const DIVERTED              = 0x0001;
        const PERSISTENTLY_DIVERTED = 0x0004;
        const RAW_XY                = 0x0010;
        const FORCE_RAW_XY          = 0x0040;
        const ANALYTICS_KEY_EVENTS  = 0x0100;
    }
}

impl MappingFlags {
    /// The companion "valid" bit for each set value bit.
    const fn valid_bits(self) -> u16 {
        self.bits() << 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Control {
    /// Slot in the control table (not stable across firmware).
    pub position_index: u8,
    /// Stable control id — what profiles are keyed on.
    pub cid: u16,
    /// Default task the firmware performs.
    pub task_id: u16,
    pub flags: ControlFlags,
    pub extra: ExtraFlags,
    /// Physical position hint for F-keys; 0 for mouse buttons.
    pub position: u8,
    pub group: u8,
    /// Bitmask of groups this control may be remapped into.
    pub group_mask: u8,
}

impl Control {
    pub fn is_divertable(&self) -> bool {
        self.flags.contains(ControlFlags::DIVERTABLE)
    }
    pub fn supports_gestures(&self) -> bool {
        self.extra.contains(ExtraFlags::RAW_XY)
    }
    pub fn name(&self) -> &'static str {
        control_name(self.cid)
    }
}

pub fn count(dev: &mut Device, t: &mut Transport) -> Result<u8> {
    at(
        dev.call(t, ids::REPROG_CONTROLS_V4, 0x00, [0; 3])?.params(),
        0,
    )
}

/// Function 1 — `getControlInfo(index)`.
pub fn info(dev: &mut Device, t: &mut Transport, index: u8) -> Result<Control> {
    let reply = dev.call(t, ids::REPROG_CONTROLS_V4, 0x01, [index, 0, 0])?;
    let p = reply.params();
    Ok(Control {
        position_index: index,
        cid: be16(p, 0)?,
        task_id: be16(p, 2)?,
        flags: ControlFlags::from_bits_truncate(at(p, 4)?),
        position: at(p, 5).unwrap_or(0),
        group: at(p, 6).unwrap_or(0),
        group_mask: at(p, 7).unwrap_or(0),
        extra: ExtraFlags::from_bits_truncate(at(p, 8).unwrap_or(0)),
    })
}

/// Enumerate every control on the device.
pub fn all(dev: &mut Device, t: &mut Transport) -> Result<Vec<Control>> {
    let n = count(dev, t)?;
    (0..n).map(|i| info(dev, t, i)).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reporting {
    pub cid: u16,
    pub flags: MappingFlags,
    /// Control id this one currently performs the task of; 0 means "default".
    pub remapped_to: u16,
}

impl Reporting {
    pub fn diverted(&self) -> bool {
        self.flags.contains(MappingFlags::DIVERTED)
    }
    pub fn raw_xy_diverted(&self) -> bool {
        self.flags.contains(MappingFlags::RAW_XY)
    }
    pub fn persisted(&self) -> bool {
        self.flags.contains(MappingFlags::PERSISTENTLY_DIVERTED)
    }
}

/// Function 2 — `getControlReporting(cid)`.
///
/// The reply carries only value bits (no valid bits). Feature version 5 adds a
/// second flags byte for analytics-key reporting.
pub fn reporting(dev: &mut Device, t: &mut Transport, cid: u16) -> Result<Reporting> {
    let [hi, lo] = cid.to_be_bytes();
    let reply = dev.call(t, ids::REPROG_CONTROLS_V4, 0x02, [hi, lo, 0])?;
    let p = reply.params();
    let low = at(p, 2)? as u16;
    let high = at(p, 5).unwrap_or(0) as u16;
    let flags = MappingFlags::from_bits_truncate(low | (high << 8));
    Ok(Reporting {
        cid: be16(p, 0)?,
        flags,
        remapped_to: be16(p, 3).unwrap_or(0),
    })
}

/// Build the flags word for `setControlReporting`.
///
/// Split out from the request so the interleaved value/valid encoding can be
/// tested without hardware — an off-by-one position here is not a crash, it is
/// a silent `InvalidArgument` from the firmware, which is far harder to spot.
fn encode_reporting(divert: Option<bool>, persist: Option<bool>, raw_xy: Option<bool>) -> u16 {
    let mut bits: u16 = 0;
    // For each field the caller expressed an opinion about: always set the
    // valid bit, and set the value bit only when enabling.
    for (flag, value) in [
        (MappingFlags::DIVERTED, divert),
        (MappingFlags::PERSISTENTLY_DIVERTED, persist),
        (MappingFlags::RAW_XY, raw_xy),
    ] {
        if let Some(v) = value {
            bits |= flag.valid_bits();
            if v {
                bits |= flag.bits();
            }
        }
    }
    bits
}

/// Function 3 — `setControlReporting`.
///
/// Five parameter bytes are needed (`cid`, flags, `remap`), so the request
/// must travel in a long report. Each `Option` left as `None` clears that
/// field's valid bit and so leaves the device's current setting alone.
pub fn set_reporting(
    dev: &mut Device,
    t: &mut Transport,
    cid: u16,
    divert: Option<bool>,
    persist: Option<bool>,
    raw_xy: Option<bool>,
    remap_to: Option<u16>,
) -> Result<()> {
    let bits = encode_reporting(divert, persist, raw_xy);

    let [cid_hi, cid_lo] = cid.to_be_bytes();
    let [rm_hi, rm_lo] = remap_to.unwrap_or(0).to_be_bytes();

    let mut params = [0u8; 16];
    params[0] = cid_hi;
    params[1] = cid_lo;
    params[2] = (bits & 0xFF) as u8;
    params[3] = rm_hi;
    params[4] = rm_lo;
    // Second flags byte (analytics key events) — only meaningful on v5+.
    params[5] = ((bits >> 8) & 0xFF) as u8;

    dev.call_long(t, ids::REPROG_CONTROLS_V4, 0x03, params)?;
    Ok(())
}

/// A `0x1B04` notification: up to four CIDs currently held down. An empty list
/// means every diverted button was released.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DivertedButtonEvent {
    pub pressed: Vec<u16>,
}

impl DivertedButtonEvent {
    /// Parse event id 0 of feature `0x1B04` from a notification packet.
    pub fn parse(pkt: &Packet) -> Option<Self> {
        let p = pkt.params();
        let mut pressed = Vec::new();
        for chunk in p.chunks_exact(2).take(4) {
            let cid = u16::from_be_bytes([chunk[0], chunk[1]]);
            if cid != 0 {
                pressed.push(cid);
            }
        }
        Some(Self { pressed })
    }
}

/// Raw XY movement streamed by a control diverted with `RAW_XY_DIVERT` —
/// event id 1 of feature `0x1B04`. This is the input to the gesture engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawXyEvent {
    pub dx: i16,
    pub dy: i16,
}

impl RawXyEvent {
    pub fn parse(pkt: &Packet) -> Option<Self> {
        let p = pkt.params();
        Some(Self {
            dx: i16::from_be_bytes([*p.first()?, *p.get(1)?]),
            dy: i16::from_be_bytes([*p.get(2)?, *p.get(3)?]),
        })
    }
}

/// Names for the control ids OpenLogi currently understands. Unknown ids still
/// work — they are shown as `Button 0x…` and remain fully remappable.
pub fn control_name(cid: u16) -> &'static str {
    match cid {
        0x0050 => "Left Click",
        0x0051 => "Right Click",
        0x0052 => "Middle Click",
        0x0053 => "Back",
        0x0056 => "Forward",
        0x005B => "Gesture Button",
        0x005D => "DPI Switch",
        0x00C3 => "Smart Shift",
        0x00C4 => "Gesture Button (MX)",
        0x00D7 => "Side Button",
        0x00E0 => "Wheel Left",
        0x00E1 => "Wheel Right",
        0x00E2 => "Top Button",
        0x0100..=0x0117 => "F-Key",
        _ => "Button",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: an earlier revision numbered the flags as a block of value
    /// bits followed by a block of valid bits. That encodes plain diversion as
    /// 0x0C, which the firmware reads as *persistent* diversion and rejects
    /// with InvalidArgument on any device lacking 0x1C00.
    #[test]
    fn divert_encodes_as_value_plus_adjacent_valid_bit() {
        assert_eq!(encode_reporting(Some(true), None, None), 0x03);
        assert_eq!(encode_reporting(Some(false), None, None), 0x02);
    }

    #[test]
    fn untouched_fields_have_no_valid_bit() {
        // Nothing requested -> nothing valid -> device keeps every setting.
        assert_eq!(encode_reporting(None, None, None), 0x00);
    }

    #[test]
    fn raw_xy_and_persist_sit_in_their_own_slots() {
        assert_eq!(encode_reporting(None, Some(true), None), 0x0C);
        assert_eq!(encode_reporting(None, None, Some(true)), 0x30);
        assert_eq!(encode_reporting(Some(true), None, Some(true)), 0x33);
    }

    #[test]
    fn valid_bit_is_always_one_left_of_its_value_bit() {
        for f in [
            MappingFlags::DIVERTED,
            MappingFlags::PERSISTENTLY_DIVERTED,
            MappingFlags::RAW_XY,
            MappingFlags::FORCE_RAW_XY,
            MappingFlags::ANALYTICS_KEY_EVENTS,
        ] {
            assert_eq!(f.valid_bits(), f.bits() << 1);
            // Value and valid bits must not collide with another flag's value.
            assert_eq!(f.bits() & f.valid_bits(), 0);
        }
    }
}
