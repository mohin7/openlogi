//! Feature `0x0003` (Device FW Version) and `0x0005` (Device Name & Type).

use crate::device::Device;
use crate::error::Result;
use crate::ids::{self, DeviceKind};
use crate::protocol::{at, be16};
use crate::transport::Transport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirmwareEntity {
    pub kind: FirmwareKind,
    /// Three-character ASCII prefix, e.g. `RQM`.
    pub prefix: String,
    /// Firmware number and revision, both BCD-encoded on the wire.
    pub number: u8,
    pub revision: u8,
    pub build: u16,
}

impl std::fmt::Display for FirmwareEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{:02x}.{:02x}.B{:04x}",
            self.prefix, self.number, self.revision, self.build
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirmwareKind {
    Main,
    Bootloader,
    Hardware,
    Touchpad,
    OpticalSensor,
    Other(u8),
}

impl From<u8> for FirmwareKind {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Main,
            1 => Self::Bootloader,
            2 => Self::Hardware,
            3 => Self::Touchpad,
            4 => Self::OpticalSensor,
            o => Self::Other(o),
        }
    }
}

/// Read every firmware entity the device reports (main firmware, bootloader,
/// hardware revision, sometimes the optical sensor).
pub fn firmware(dev: &mut Device, t: &mut Transport) -> Result<Vec<FirmwareEntity>> {
    let count = at(
        dev.call(t, ids::DEVICE_FW_VERSION, 0x00, [0; 3])?.params(),
        0,
    )?;
    let mut out = Vec::with_capacity(count as usize);
    for entity in 0..count {
        let reply = dev.call(t, ids::DEVICE_FW_VERSION, 0x01, [entity, 0, 0])?;
        let p = reply.params();
        out.push(FirmwareEntity {
            kind: FirmwareKind::from(at(p, 0)?),
            prefix: p
                .get(1..4)
                .map(|b| String::from_utf8_lossy(b).trim().to_string())
                .unwrap_or_default(),
            number: at(p, 4)?,
            revision: at(p, 5)?,
            build: be16(p, 6)?,
        });
    }
    Ok(out)
}

/// Read the marketing name, e.g. `Signature M650`.
///
/// The name is fetched 16 bytes at a time because it has to travel in a single
/// long report's payload.
pub fn name(dev: &mut Device, t: &mut Transport) -> Result<String> {
    let len = at(dev.call(t, ids::DEVICE_NAME, 0x00, [0; 3])?.params(), 0)? as usize;
    let mut buf = Vec::with_capacity(len);
    while buf.len() < len {
        let reply = dev.call(t, ids::DEVICE_NAME, 0x01, [buf.len() as u8, 0, 0])?;
        let chunk = reply.params();
        if chunk.is_empty() {
            break;
        }
        buf.extend_from_slice(chunk);
    }
    buf.truncate(len);
    Ok(String::from_utf8_lossy(&buf)
        .trim_end_matches('\0')
        .trim()
        .to_string())
}

pub fn kind(dev: &mut Device, t: &mut Transport) -> Result<DeviceKind> {
    let reply = dev.call(t, ids::DEVICE_NAME, 0x02, [0; 3])?;
    Ok(DeviceKind::from(at(reply.params(), 0)?))
}

/// Identity read from feature `0x0003` **v4+** function 0.
///
/// Version 4 of Device FW Version extended `getDeviceInfo` to return the unit
/// id, transport bitfield and model id alongside the entity count. That
/// matters because many devices — the Signature M650 among them — do not
/// implement feature `0x0004` at all, so this is the only way to obtain a
/// stable per-unit identifier. Profiles are keyed on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub entity_count: u8,
    /// Stable 4-byte identifier, unique to this physical unit.
    pub unit_id: [u8; 4],
    /// Which transports the device supports (bit 0 Bluetooth, bit 1 BLE,
    /// bit 2 eQuad, bit 3 USB).
    pub transport: u16,
    /// Per-transport product ids, in the order the transport bits are set.
    pub model_ids: Vec<u16>,
}

impl DeviceInfo {
    /// Hex form used as the profile key, e.g. `481871D4`.
    pub fn unit_id_hex(&self) -> String {
        self.unit_id.iter().map(|b| format!("{b:02X}")).collect()
    }
    pub fn supports_bluetooth(&self) -> bool {
        self.transport & 0x0001 != 0
    }
    pub fn supports_ble(&self) -> bool {
        self.transport & 0x0002 != 0
    }
    pub fn supports_equad(&self) -> bool {
        self.transport & 0x0004 != 0
    }
    pub fn supports_usb(&self) -> bool {
        self.transport & 0x0008 != 0
    }
}

/// Read `0x0003` function 0. Returns `Ok(None)` when the feature is older than
/// v4 and therefore carries only an entity count.
pub fn device_info(dev: &mut Device, t: &mut Transport) -> Result<Option<DeviceInfo>> {
    let entry = dev.require(t, ids::DEVICE_FW_VERSION)?;
    if entry.version < 4 {
        return Ok(None);
    }
    let reply = dev.call(t, ids::DEVICE_FW_VERSION, 0x00, [0; 3])?;
    let p = reply.params();

    let mut unit_id = [0u8; 4];
    unit_id.copy_from_slice(p.get(1..5).ok_or(crate::error::Error::Malformed(
        "device info: missing unit id",
    ))?);

    let transport = be16(p, 5)?;
    // One model id per set transport bit, packed consecutively from byte 7.
    let mut model_ids = Vec::new();
    for bit in 0..4 {
        if transport & (1 << bit) != 0 {
            if let Ok(id) = be16(p, 7 + model_ids.len() * 2) {
                model_ids.push(id);
            }
        }
    }

    Ok(Some(DeviceInfo {
        entity_count: at(p, 0)?,
        unit_id,
        transport,
        model_ids,
    }))
}

/// Feature `0x0004`: the 4-byte unit id, stable across re-pairings. This is
/// the key OpenLogi stores profiles against — the device index is not stable,
/// and the serial can change between transports.
pub fn unit_id(dev: &mut Device, t: &mut Transport) -> Result<String> {
    let reply = dev.call(t, ids::DEVICE_UNIT_ID, 0x00, [0; 3])?;
    let p = reply.params();
    Ok(p.get(0..4)
        .map(|b| b.iter().map(|x| format!("{x:02X}")).collect::<String>())
        .unwrap_or_default())
}
