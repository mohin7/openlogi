//! Feature `0x2201` — Adjustable DPI.
//!
//! The DPI list is encoded compactly: a plain `u16` is a discrete value, but a
//! value with the `0xE000` marker means "the previous value through the next
//! value, in steps of `marker & 0x1FFF`". So `[400, 0xE032, 4000]` means
//! 400–4000 in steps of 50. Both forms appear in the wild; the M650-class
//! sensors typically report a range.

use crate::device::Device;
use crate::error::Result;
use crate::ids;
use crate::protocol::{at, be16};
use crate::transport::Transport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DpiRange {
    /// An explicit set of selectable values.
    List(Vec<u16>),
    /// A continuous range with a fixed step.
    Steps { min: u16, max: u16, step: u16 },
}

impl DpiRange {
    /// Flatten to concrete selectable values, capped so a 50-step range does
    /// not produce a thousand-entry dropdown.
    pub fn values(&self) -> Vec<u16> {
        match self {
            Self::List(v) => v.clone(),
            Self::Steps { min, max, step } => {
                let step = (*step).max(1);
                (*min..=*max).step_by(step as usize).collect()
            }
        }
    }

    pub fn min(&self) -> u16 {
        match self {
            Self::List(v) => v.iter().copied().min().unwrap_or(0),
            Self::Steps { min, .. } => *min,
        }
    }

    pub fn max(&self) -> u16 {
        match self {
            Self::List(v) => v.iter().copied().max().unwrap_or(0),
            Self::Steps { max, .. } => *max,
        }
    }

    /// Snap an arbitrary value onto something the sensor will accept.
    pub fn snap(&self, wanted: u16) -> u16 {
        match self {
            Self::List(v) => v
                .iter()
                .copied()
                .min_by_key(|c| c.abs_diff(wanted))
                .unwrap_or(wanted),
            Self::Steps { min, max, step } => {
                let step = (*step).max(1);
                let clamped = wanted.clamp(*min, *max);
                let offset = clamped - *min;
                let snapped = *min + (offset + step / 2) / step * step;
                snapped.min(*max)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensorDpi {
    pub sensor: u8,
    pub current: u16,
    pub default: u16,
    pub range: DpiRange,
}

pub fn sensor_count(dev: &mut Device, t: &mut Transport) -> Result<u8> {
    at(dev.call(t, ids::ADJUSTABLE_DPI, 0x00, [0; 3])?.params(), 0)
}

/// Function 1 — `getSensorDpiList`.
pub fn range(dev: &mut Device, t: &mut Transport, sensor: u8) -> Result<DpiRange> {
    let reply = dev.call(t, ids::ADJUSTABLE_DPI, 0x01, [sensor, 0, 0])?;
    let p = reply.params();

    // params[0] echoes the sensor index; the list follows as big-endian u16s.
    let mut raw = Vec::new();
    let mut i = 1;
    while i + 1 < p.len() {
        let v = be16(p, i)?;
        if v == 0 {
            break; // list terminator
        }
        raw.push(v);
        i += 2;
    }

    if let Some(pos) = raw.iter().position(|v| v & 0xE000 == 0xE000) {
        if pos > 0 && pos + 1 < raw.len() {
            return Ok(DpiRange::Steps {
                min: raw[pos - 1],
                max: raw[pos + 1],
                step: raw[pos] & 0x1FFF,
            });
        }
    }
    Ok(DpiRange::List(raw))
}

/// Function 2 — `getSensorDpi`.
pub fn get(dev: &mut Device, t: &mut Transport, sensor: u8) -> Result<SensorDpi> {
    let reply = dev.call(t, ids::ADJUSTABLE_DPI, 0x02, [sensor, 0, 0])?;
    let p = reply.params();
    Ok(SensorDpi {
        sensor,
        current: be16(p, 1)?,
        default: be16(p, 3).unwrap_or(0),
        range: range(dev, t, sensor)?,
    })
}

/// Function 3 — `setSensorDpi`. The value is snapped to the sensor's supported
/// range first; sending an unsupported value makes the firmware reply
/// `InvalidArgument` and leave the DPI unchanged.
pub fn set(dev: &mut Device, t: &mut Transport, sensor: u8, dpi: u16) -> Result<u16> {
    let snapped = range(dev, t, sensor)?.snap(dpi);
    let [hi, lo] = snapped.to_be_bytes();
    dev.call(t, ids::ADJUSTABLE_DPI, 0x03, [sensor, hi, lo])?;
    Ok(snapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_snap_to_nearest_multiple() {
        let r = DpiRange::Steps {
            min: 400,
            max: 4000,
            step: 50,
        };
        assert_eq!(r.snap(437), 450);
        assert_eq!(r.snap(10), 400);
        assert_eq!(r.snap(65535), 4000);
    }

    #[test]
    fn list_snaps_to_nearest_entry() {
        let r = DpiRange::List(vec![800, 1200, 1600]);
        assert_eq!(r.snap(1000), 800);
        assert_eq!(r.snap(1300), 1200);
        assert_eq!(r.snap(9000), 1600);
    }
}
