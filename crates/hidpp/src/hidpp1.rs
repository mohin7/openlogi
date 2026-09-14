//! HID++ 1.0 register access.
//!
//! HID++ 2.0 replaced registers with features, but receivers never stopped
//! speaking 1.0 — pairing information, connection state and, crucially,
//! *notification enabling* all still live behind 1.0 registers.
//!
//! The wire format reuses the same 7/20-byte reports, but reinterprets the
//! header: where 2.0 has a feature index and a `function|swid` byte, 1.0 has a
//! **sub-id** and a **register address**.
//!
//! ```text
//! 2.0:  [0x10, dev, feature_idx, fn<<4|swid, p0, p1, p2]
//! 1.0:  [0x10, dev, sub_id,      address,    p0, p1, p2]
//! ```
//!
//! Because the sub-id and address are echoed in the reply, the same
//! request/response matching used for 2.0 works unchanged.

use crate::error::Result;
use crate::protocol::{Packet, RECEIVER_INDEX};
use crate::transport::Transport;

pub const SET_REGISTER: u8 = 0x80;
pub const GET_REGISTER: u8 = 0x81;
pub const SET_LONG_REGISTER: u8 = 0x82;
pub const GET_LONG_REGISTER: u8 = 0x83;

pub mod register {
    /// Enable/disable which notifications the receiver forwards.
    pub const NOTIFICATIONS: u8 = 0x00;
    /// Connection state; writing 0x02 asks the receiver to re-announce every
    /// paired device.
    pub const CONNECTION_STATE: u8 = 0x02;
    /// Pairing information: sub-address `0x20 + n`, `0x30 + n`, `0x40 + n`.
    pub const PAIRING_INFO: u8 = 0xB5;
}

/// Notification flags for register `0x00`, as a 24-bit big-endian field.
pub mod notification {
    /// Forward wireless device connect/disconnect notifications.
    pub const WIRELESS: u32 = 0x00_0100;
    /// Tell the device that configuration software is running. Several
    /// Logitech devices only emit diverted-button events while this is set.
    pub const SOFTWARE_PRESENT: u32 = 0x00_0800;
    /// Battery status change notifications.
    pub const BATTERY_STATUS: u32 = 0x10_0000;
}

/// Write a 3-byte register.
pub fn set_register(
    transport: &mut Transport,
    device_index: u8,
    address: u8,
    params: [u8; 3],
) -> Result<Packet> {
    let req = Packet::hidpp1_short(device_index, SET_REGISTER, address, params);
    let reply = transport.request(req)?;
    if reply.is_error_10() || reply.is_error_20() {
        return Err(reply.into_error(0));
    }
    Ok(reply)
}

/// Read a 3-byte register.
pub fn get_register(
    transport: &mut Transport,
    device_index: u8,
    address: u8,
    params: [u8; 3],
) -> Result<Packet> {
    let req = Packet::hidpp1_short(device_index, GET_REGISTER, address, params);
    let reply = transport.request(req)?;
    if reply.is_error_10() || reply.is_error_20() {
        return Err(reply.into_error(0));
    }
    Ok(reply)
}

/// Read a 16-byte register (the reply is a long report).
pub fn get_long_register(
    transport: &mut Transport,
    device_index: u8,
    address: u8,
    params: [u8; 3],
) -> Result<Packet> {
    let req = Packet::hidpp1_short(device_index, GET_LONG_REGISTER, address, params);
    let reply = transport.request(req)?;
    if reply.is_error_10() || reply.is_error_20() {
        return Err(reply.into_error(0));
    }
    Ok(reply)
}

/// Turn on the notification classes OpenLogi depends on.
///
/// This is not optional. A receiver forwards notifications only for the
/// classes enabled here, and `SOFTWARE_PRESENT` in particular is how the
/// device learns that configuration software is listening — without it, some
/// firmware accepts a diversion request and then never sends the events,
/// which looks exactly like a broken reader.
pub fn enable_notifications(transport: &mut Transport, device_index: u8) -> Result<u32> {
    let flags =
        notification::WIRELESS | notification::SOFTWARE_PRESENT | notification::BATTERY_STATUS;
    let bytes = flags.to_be_bytes(); // 4 bytes; the register takes the low 3
    set_register(
        transport,
        device_index,
        register::NOTIFICATIONS,
        [bytes[1], bytes[2], bytes[3]],
    )?;
    Ok(flags)
}

/// Read back the currently enabled notification flags.
pub fn notification_flags(transport: &mut Transport, device_index: u8) -> Result<u32> {
    let reply = get_register(transport, device_index, register::NOTIFICATIONS, [0; 3])?;
    let p = reply.params();
    Ok(u32::from_be_bytes([
        0,
        p.first().copied().unwrap_or(0),
        p.get(1).copied().unwrap_or(0),
        p.get(2).copied().unwrap_or(0),
    ]))
}

/// Ask the receiver to re-announce every paired device. Useful after startup
/// so the daemon learns about devices without waiting for the user to move
/// them.
pub fn announce_devices(transport: &mut Transport) -> Result<()> {
    set_register(
        transport,
        RECEIVER_INDEX,
        register::CONNECTION_STATE,
        [0x02, 0x00, 0x00],
    )?;
    Ok(())
}
