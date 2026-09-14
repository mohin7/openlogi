//! Serialisable mirrors of the `hidpp` types.
//!
//! These are the contract with the frontend: field names match
//! `src/types/device.ts` exactly, so the browser preview and the live app
//! render from the same shapes. Keeping the DTOs separate from the protocol
//! types means the wire format for the UI can evolve without touching the
//! protocol crate.

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BatteryDto {
    pub percentage: Option<u8>,
    pub state: &'static str,
    pub source_feature: u16,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareDto {
    pub kind: &'static str,
    pub version: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ControlDto {
    pub cid: u16,
    pub task_id: u16,
    pub name: String,
    pub reprogrammable: bool,
    pub divertable: bool,
    pub supports_gestures: bool,
    /// `virtual` is a reserved word in Rust, so the field is renamed on the
    /// wire to match `Control.virtual` in src/types/device.ts.
    #[serde(rename = "virtual")]
    pub virtual_control: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DpiRangeDto {
    pub min: u16,
    pub max: u16,
    pub step: u16,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitiesDto {
    pub features: Vec<u16>,
    pub dpi: Option<DpiRangeDto>,
    pub has_adjustable_dpi: bool,
    pub has_pointer_speed: bool,
    pub has_smart_shift: bool,
    pub has_hi_res_wheel: bool,
    pub has_thumb_wheel: bool,
    pub can_persist_onboard: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDto {
    pub id: String,
    pub unit_id: String,
    pub name: String,
    pub kind: &'static str,
    pub connection: &'static str,
    pub connected: bool,
    pub protocol: String,
    pub model_id: String,
    pub battery: BatteryDto,
    pub firmware: Vec<FirmwareDto>,
    pub controls: Vec<ControlDto>,
    pub capabilities: CapabilitiesDto,
    pub current_dpi: u16,
    /// The sensor's factory default, reported by 0x2201. Reset restores this
    /// rather than a number we invented.
    pub default_dpi: u16,
    pub last_seen: Option<String>,
}
