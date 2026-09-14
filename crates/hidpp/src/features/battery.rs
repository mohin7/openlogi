//! Battery reporting.
//!
//! Logitech has shipped three incompatible battery features over the years and
//! a device implements exactly one of them:
//!
//! * `0x1000` **Battery Status** — older devices; percentage plus a coarse
//!   "next level" hint.
//! * `0x1001` **Battery Voltage** — reports millivolts; the percentage has to
//!   be approximated from a discharge curve.
//! * `0x1004` **Unified Battery** — current devices; percentage and/or a
//!   four-step level, plus charging state.
//!
//! [`read`] probes them in newest-first order so callers never care which.

use crate::device::Device;
use crate::error::{Error, Result};
use crate::ids;
use crate::protocol::{at, be16};
use crate::transport::Transport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargeState {
    Discharging,
    Charging,
    ChargingSlow,
    ChargeComplete,
    ChargeError,
    Unknown,
}

/// Coarse level, for devices that refuse to report a percentage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChargeLevel {
    Critical,
    Low,
    Good,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Battery {
    pub percentage: Option<u8>,
    pub level: Option<ChargeLevel>,
    pub state: ChargeState,
    pub voltage_mv: Option<u16>,
    /// Which feature produced this reading — surfaced in diagnostics.
    pub source: u16,
}

impl Battery {
    pub fn is_charging(&self) -> bool {
        matches!(
            self.state,
            ChargeState::Charging | ChargeState::ChargingSlow
        )
    }

    /// Percentage if reported, otherwise a representative value for the level
    /// bucket, so the UI always has something to draw.
    pub fn approx_percentage(&self) -> Option<u8> {
        self.percentage.or(self.level.map(|l| match l {
            ChargeLevel::Critical => 5,
            ChargeLevel::Low => 20,
            ChargeLevel::Good => 60,
            ChargeLevel::Full => 100,
        }))
    }
}

fn level_from_flags(flags: u8) -> Option<ChargeLevel> {
    // Bitfield, highest set bit wins.
    if flags & 0x08 != 0 {
        Some(ChargeLevel::Full)
    } else if flags & 0x04 != 0 {
        Some(ChargeLevel::Good)
    } else if flags & 0x02 != 0 {
        Some(ChargeLevel::Low)
    } else if flags & 0x01 != 0 {
        Some(ChargeLevel::Critical)
    } else {
        None
    }
}

/// Read the battery using whichever feature this device implements.
pub fn read(dev: &mut Device, t: &mut Transport) -> Result<Battery> {
    if dev.feature(t, ids::UNIFIED_BATTERY)?.is_some() {
        return read_unified(dev, t);
    }
    if dev.feature(t, ids::BATTERY_STATUS)?.is_some() {
        return read_status(dev, t);
    }
    if dev.feature(t, ids::BATTERY_VOLTAGE)?.is_some() {
        return read_voltage(dev, t);
    }
    Err(Error::FeatureUnsupported {
        feature_id: ids::UNIFIED_BATTERY,
    })
}

/// Feature `0x1004` function 1 — `getStatus`.
pub fn read_unified(dev: &mut Device, t: &mut Transport) -> Result<Battery> {
    let reply = dev.call(t, ids::UNIFIED_BATTERY, 0x01, [0; 3])?;
    let p = reply.params();
    let soc = at(p, 0)?;
    let state = match at(p, 2)? {
        0 => ChargeState::Discharging,
        1 => ChargeState::Charging,
        2 => ChargeState::ChargingSlow,
        3 => ChargeState::ChargeComplete,
        4 => ChargeState::ChargeError,
        _ => ChargeState::Unknown,
    };
    Ok(Battery {
        // A device that only supports levels reports 0 here.
        percentage: (soc > 0).then_some(soc),
        level: level_from_flags(at(p, 1)?),
        state,
        voltage_mv: None,
        source: ids::UNIFIED_BATTERY,
    })
}

/// Feature `0x1000` function 0 — `getBatteryLevelStatus`.
pub fn read_status(dev: &mut Device, t: &mut Transport) -> Result<Battery> {
    let reply = dev.call(t, ids::BATTERY_STATUS, 0x00, [0; 3])?;
    let p = reply.params();
    let state = match at(p, 2)? {
        0 => ChargeState::Discharging,
        1 => ChargeState::Charging,
        2 => ChargeState::Charging,
        3 => ChargeState::ChargeComplete,
        4 => ChargeState::ChargingSlow,
        5..=7 => ChargeState::ChargeError,
        _ => ChargeState::Unknown,
    };
    let pct = at(p, 0)?;
    Ok(Battery {
        percentage: (pct > 0).then_some(pct),
        level: None,
        state,
        voltage_mv: None,
        source: ids::BATTERY_STATUS,
    })
}

/// Feature `0x1001` function 0 — `getBatteryVoltage`.
pub fn read_voltage(dev: &mut Device, t: &mut Transport) -> Result<Battery> {
    let reply = dev.call(t, ids::BATTERY_VOLTAGE, 0x00, [0; 3])?;
    let p = reply.params();
    let mv = be16(p, 0)?;
    let flags = at(p, 2)?;
    Ok(Battery {
        percentage: Some(percentage_from_voltage(mv)),
        level: None,
        state: if flags & 0x80 != 0 {
            ChargeState::Charging
        } else {
            ChargeState::Discharging
        },
        voltage_mv: Some(mv),
        source: ids::BATTERY_VOLTAGE,
    })
}

/// Piecewise-linear discharge curve for a single alkaline/NiMH cell as used by
/// Logitech mice. Matches the table Logitech ships for `0x1001` devices
/// closely enough for a UI reading; it is never better than ±5%.
fn percentage_from_voltage(mv: u16) -> u8 {
    const CURVE: &[(u16, u8)] = &[
        (4186, 100),
        (4067, 90),
        (3989, 80),
        (3922, 70),
        (3859, 60),
        (3811, 50),
        (3778, 40),
        (3751, 30),
        (3717, 20),
        (3671, 10),
        (3579, 5),
        (3500, 0),
    ];
    if mv >= CURVE[0].0 {
        return 100;
    }
    for pair in CURVE.windows(2) {
        let (hi_mv, hi_pct) = pair[0];
        let (lo_mv, lo_pct) = pair[1];
        if mv <= hi_mv && mv > lo_mv {
            let span = (hi_mv - lo_mv) as u32;
            let into = (mv - lo_mv) as u32;
            let pct_span = (hi_pct - lo_pct) as u32;
            return (lo_pct as u32 + pct_span * into / span) as u8;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voltage_curve_is_monotonic_and_bounded() {
        assert_eq!(percentage_from_voltage(4300), 100);
        assert_eq!(percentage_from_voltage(3000), 0);
        let mut last = 0;
        for mv in (3500..=4200).step_by(10) {
            let p = percentage_from_voltage(mv);
            assert!(p >= last, "curve dipped at {mv}mV");
            last = p;
        }
    }

    #[test]
    fn level_flags_pick_highest_bit() {
        assert_eq!(level_from_flags(0b0000), None);
        assert_eq!(level_from_flags(0b0001), Some(ChargeLevel::Critical));
        assert_eq!(level_from_flags(0b0110), Some(ChargeLevel::Good));
        assert_eq!(level_from_flags(0b1111), Some(ChargeLevel::Full));
    }
}
