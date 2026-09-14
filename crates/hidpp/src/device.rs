//! A single HID++ 2.0 device sitting behind a [`Transport`].
//!
//! One transport (one `/dev/hidraw` node) fronts up to six paired devices on a
//! receiver, or exactly one device when connected directly. `Device` owns the
//! per-device feature-index cache; the transport stays shared and mutable, so
//! methods take `&mut Transport` rather than borrowing it for the device's
//! lifetime.

use std::collections::HashMap;

use tracing::debug;

use crate::error::{Error, Result};
use crate::ids;
use crate::protocol::{at, Packet, ROOT_INDEX};
use crate::transport::Transport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FeatureEntry {
    pub id: u16,
    pub index: u8,
    pub version: u8,
    /// Bit 7 of the type byte: the feature is obsolete.
    pub obsolete: bool,
    /// Bit 6: the feature is hidden from ordinary configuration software.
    pub hidden: bool,
    /// Bit 5: the feature is engineering-only.
    pub engineering: bool,
}

pub struct Device {
    index: u8,
    protocol: ProtocolVersion,
    by_id: HashMap<u16, FeatureEntry>,
}

impl Device {
    /// Probe a device index. Returns `Ok(None)` when nothing answers — an
    /// empty pairing slot or a device that is powered off.
    pub fn probe(transport: &mut Transport, device_index: u8) -> Result<Option<Self>> {
        // Root feature, function 1 (getProtocolVersion). The third parameter
        // is echoed back, which proves the reply belongs to this ping and not
        // to a stale report sitting in the kernel buffer.
        const PING_MARK: u8 = 0x5A;
        let req = Packet::short(device_index, ROOT_INDEX, 0x01, [0x00, 0x00, PING_MARK]);

        let reply = match transport.request(req) {
            Ok(r) => r,
            Err(Error::Timeout { .. }) => return Ok(None),
            Err(e) => return Err(e),
        };

        if reply.is_error_10() {
            // 0x09 = "resource error"/unreachable on most receivers.
            debug!(device_index, code = reply.error_code(), "ping refused");
            return Ok(None);
        }
        if reply.is_error_20() {
            return Ok(None);
        }
        if at(reply.params(), 2)? != PING_MARK {
            return Err(Error::Malformed("ping mark not echoed"));
        }

        Ok(Some(Self {
            index: device_index,
            protocol: ProtocolVersion {
                major: at(reply.params(), 0)?,
                minor: at(reply.params(), 1)?,
            },
            by_id: HashMap::new(),
        }))
    }

    pub fn index(&self) -> u8 {
        self.index
    }

    pub fn protocol(&self) -> ProtocolVersion {
        self.protocol
    }

    /// Resolve a feature id to its runtime index, caching the result.
    /// `Ok(None)` means the device does not implement the feature.
    pub fn feature(
        &mut self,
        transport: &mut Transport,
        feature_id: u16,
    ) -> Result<Option<FeatureEntry>> {
        if let Some(entry) = self.by_id.get(&feature_id) {
            return Ok(Some(*entry));
        }
        if feature_id == ids::ROOT {
            let entry = FeatureEntry {
                id: ids::ROOT,
                index: ROOT_INDEX,
                version: 0,
                obsolete: false,
                hidden: false,
                engineering: false,
            };
            self.by_id.insert(feature_id, entry);
            return Ok(Some(entry));
        }

        // Root function 0 (getFeature): params are the big-endian feature id.
        let [hi, lo] = feature_id.to_be_bytes();
        let reply =
            transport.request(Packet::short(self.index, ROOT_INDEX, 0x00, [hi, lo, 0x00]))?;
        if reply.is_error_10() || reply.is_error_20() {
            return Ok(None);
        }

        let index = at(reply.params(), 0)?;
        if index == 0 {
            // Index 0 is the root itself, so it means "not supported".
            return Ok(None);
        }
        let flags = at(reply.params(), 1).unwrap_or(0);
        let entry = FeatureEntry {
            id: feature_id,
            index,
            version: at(reply.params(), 2).unwrap_or(0),
            obsolete: flags & 0x80 != 0,
            hidden: flags & 0x40 != 0,
            engineering: flags & 0x20 != 0,
        };
        self.by_id.insert(feature_id, entry);
        Ok(Some(entry))
    }

    /// Resolve a feature id, erroring if it is absent.
    pub fn require(&mut self, transport: &mut Transport, feature_id: u16) -> Result<FeatureEntry> {
        self.feature(transport, feature_id)?
            .ok_or(Error::FeatureUnsupported { feature_id })
    }

    /// Invoke `function` on `feature_id` with up to 3 parameter bytes.
    pub fn call(
        &mut self,
        transport: &mut Transport,
        feature_id: u16,
        function: u8,
        params: [u8; 3],
    ) -> Result<Packet> {
        let entry = self.require(transport, feature_id)?;
        let reply = transport.request(Packet::short(self.index, entry.index, function, params))?;
        if reply.is_error_10() || reply.is_error_20() {
            return Err(reply.into_error(feature_id));
        }
        Ok(reply)
    }

    /// Invoke `function` with a 16-byte (long-report) parameter block. Some
    /// features — notably macro and profile writes — only accept long requests.
    pub fn call_long(
        &mut self,
        transport: &mut Transport,
        feature_id: u16,
        function: u8,
        params: [u8; 16],
    ) -> Result<Packet> {
        let entry = self.require(transport, feature_id)?;
        let reply = transport.request(Packet::long(self.index, entry.index, function, params))?;
        if reply.is_error_10() || reply.is_error_20() {
            return Err(reply.into_error(feature_id));
        }
        Ok(reply)
    }

    /// Enumerate every feature the device implements, via feature `0x0001`.
    ///
    /// This is what drives OpenLogi's capability model: rather than keeping a
    /// hard-coded table of "the M650 supports X", we ask the device and light
    /// up UI accordingly. New Logitech hardware then works without a release.
    pub fn enumerate_features(&mut self, transport: &mut Transport) -> Result<Vec<FeatureEntry>> {
        let set = self.require(transport, ids::FEATURE_SET)?;

        // Function 0: getCount. The root feature is not counted, so indices
        // run 0..=count with 0 being the root.
        let reply = transport.request(Packet::short(self.index, set.index, 0x00, [0; 3]))?;
        let count = at(reply.params(), 0)?;

        let mut out = Vec::with_capacity(count as usize + 1);
        out.push(FeatureEntry {
            id: ids::ROOT,
            index: 0,
            version: 0,
            obsolete: false,
            hidden: false,
            engineering: false,
        });

        for index in 1..=count {
            // Function 1: getFeatureId(index).
            let reply =
                transport.request(Packet::short(self.index, set.index, 0x01, [index, 0, 0]))?;
            if reply.is_error_10() || reply.is_error_20() {
                continue;
            }
            let p = reply.params();
            let id = crate::protocol::be16(p, 0)?;
            let flags = at(p, 2).unwrap_or(0);
            let entry = FeatureEntry {
                id,
                index,
                version: at(p, 3).unwrap_or(0),
                obsolete: flags & 0x80 != 0,
                hidden: flags & 0x40 != 0,
                engineering: flags & 0x20 != 0,
            };
            self.by_id.insert(id, entry);
            out.push(entry);
        }
        Ok(out)
    }
}
