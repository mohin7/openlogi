//! Wire format for HID++ 1.0 and 2.0.
//!
//! Every HID++ exchange is a fixed-size HID report on the vendor collection
//! (usage page `0xFF00`). There are exactly two shapes:
//!
//! ```text
//! SHORT  report 0x10, 7 bytes   [id, dev, feat, fn|swid, p0, p1, p2]
//! LONG   report 0x11, 20 bytes  [id, dev, feat, fn|swid, p0 .. p15]
//! ```
//!
//! A request is answered by a report with the *same* device index, feature
//! index and `fn|swid` byte. Because the software id is echoed back, several
//! programs can drive the same device concurrently without stealing each
//! other's replies — which is why OpenLogi must never use software id 0.

use crate::error::{Error, Hidpp10Error, Hidpp20Error, Hidpp20ErrorWrapper, Result};

pub const SHORT_REPORT_ID: u8 = 0x10;
pub const LONG_REPORT_ID: u8 = 0x11;
pub const SHORT_LEN: usize = 7;
pub const LONG_LEN: usize = 20;
pub const MAX_LEN: usize = LONG_LEN;

/// Software id stamped into the low nibble of byte 3. Must be 1..=15.
/// `0x0A` is OpenLogi's; Solaar uses `0x01..0x08`, so we do not collide.
pub const SW_ID: u8 = 0x0A;

/// Device index addressing the receiver itself rather than a paired device.
pub const RECEIVER_INDEX: u8 = 0xFF;

/// Device index used when the device is attached directly (USB cable or BLE),
/// i.e. there is no receiver in between.
pub const DIRECT_INDEX: u8 = 0xFF;

/// The root feature is guaranteed to live at index 0 on every HID++ 2.0 device.
pub const ROOT_INDEX: u8 = 0x00;

/// A single HID++ report, sized to whichever form it uses.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Packet {
    buf: [u8; MAX_LEN],
    len: usize,
}

impl Packet {
    pub fn short(device_index: u8, feature_index: u8, function: u8, params: [u8; 3]) -> Self {
        let mut buf = [0u8; MAX_LEN];
        buf[0] = SHORT_REPORT_ID;
        buf[1] = device_index;
        buf[2] = feature_index;
        buf[3] = (function << 4) | SW_ID;
        buf[4..7].copy_from_slice(&params);
        Self {
            buf,
            len: SHORT_LEN,
        }
    }

    pub fn long(device_index: u8, feature_index: u8, function: u8, params: [u8; 16]) -> Self {
        let mut buf = [0u8; MAX_LEN];
        buf[0] = LONG_REPORT_ID;
        buf[1] = device_index;
        buf[2] = feature_index;
        buf[3] = (function << 4) | SW_ID;
        buf[4..20].copy_from_slice(&params);
        Self { buf, len: LONG_LEN }
    }

    /// Build a HID++ **1.0** short request.
    ///
    /// Byte 2 carries a sub-id and byte 3 a register address, rather than a
    /// feature index and `function|swid`. Note there is no software id in 1.0,
    /// so concurrent configuration tools cannot disambiguate each other's
    /// register replies — keep 1.0 traffic to startup and pairing queries.
    pub fn hidpp1_short(device_index: u8, sub_id: u8, address: u8, params: [u8; 3]) -> Self {
        let mut buf = [0u8; MAX_LEN];
        buf[0] = SHORT_REPORT_ID;
        buf[1] = device_index;
        buf[2] = sub_id;
        buf[3] = address;
        buf[4..7].copy_from_slice(&params);
        Self {
            buf,
            len: SHORT_LEN,
        }
    }

    /// Wrap bytes read from the wire. Returns `None` for reports that are not
    /// HID++ (the receiver also emits ordinary mouse/keyboard reports).
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        let len = match *bytes.first()? {
            SHORT_REPORT_ID if bytes.len() >= SHORT_LEN => SHORT_LEN,
            LONG_REPORT_ID if bytes.len() >= LONG_LEN => LONG_LEN,
            _ => return None,
        };
        let mut buf = [0u8; MAX_LEN];
        buf[..len].copy_from_slice(&bytes[..len]);
        Some(Self { buf, len })
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
    #[inline]
    pub fn report_id(&self) -> u8 {
        self.buf[0]
    }
    #[inline]
    pub fn device_index(&self) -> u8 {
        self.buf[1]
    }
    #[inline]
    pub fn feature_index(&self) -> u8 {
        self.buf[2]
    }
    #[inline]
    pub fn function(&self) -> u8 {
        self.buf[3] >> 4
    }
    #[inline]
    pub fn software_id(&self) -> u8 {
        self.buf[3] & 0x0F
    }
    /// Payload bytes (everything after the 4-byte header).
    #[inline]
    pub fn params(&self) -> &[u8] {
        &self.buf[4..self.len]
    }

    /// A HID++ 1.0 error report: short, with `0x8F` where the feature index
    /// would be.
    pub fn is_error_10(&self) -> bool {
        self.report_id() == SHORT_REPORT_ID && self.feature_index() == 0x8F
    }

    /// A HID++ 2.0 error report: long, with `0xFF` where the feature index
    /// would be. Byte 3 carries the *failing* feature index, byte 4 the
    /// failing `fn|swid`, byte 5 the error code.
    pub fn is_error_20(&self) -> bool {
        self.report_id() == LONG_REPORT_ID && self.feature_index() == 0xFF
    }

    /// For an error report, the feature index / `fn|swid` of the request that
    /// caused it — needed to match an error to its outstanding request.
    pub fn error_origin(&self) -> Option<(u8, u8)> {
        if self.is_error_20() {
            Some((self.buf[3], self.buf[4]))
        } else if self.is_error_10() {
            // sub id, address
            Some((self.buf[3], self.buf[4]))
        } else {
            None
        }
    }

    pub fn error_code(&self) -> u8 {
        self.buf[5]
    }

    /// Convert an error report into a typed `Error`.
    pub fn into_error(self, feature_id: u16) -> Error {
        if self.is_error_20() {
            Error::Protocol20 {
                feature_id,
                function: self.buf[4] >> 4,
                source: Hidpp20ErrorWrapper(Hidpp20Error::from(self.error_code())),
            }
        } else {
            Error::Protocol10(Hidpp10Error(self.error_code()))
        }
    }

    /// Does this report answer `request`?
    pub fn answers(&self, request: &Packet) -> bool {
        if self.device_index() != request.device_index() {
            return false;
        }
        if let Some((feat, fnsw)) = self.error_origin() {
            return feat == request.feature_index() && fnsw == request.buf[3];
        }
        self.feature_index() == request.feature_index() && self.buf[3] == request.buf[3]
    }
}

impl std::fmt::Debug for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Packet[")?;
        for (i, b) in self.as_bytes().iter().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            write!(f, "{b:02x}")?;
        }
        f.write_str("]")
    }
}

/// Helper: read a big-endian u16 out of a params slice.
pub fn be16(params: &[u8], offset: usize) -> Result<u16> {
    let hi = *params.get(offset).ok_or(Error::Malformed("short params"))?;
    let lo = *params
        .get(offset + 1)
        .ok_or(Error::Malformed("short params"))?;
    Ok(u16::from_be_bytes([hi, lo]))
}

/// Helper: read one byte out of a params slice.
pub fn at(params: &[u8], offset: usize) -> Result<u8> {
    params
        .get(offset)
        .copied()
        .ok_or(Error::Malformed("short params"))
}
