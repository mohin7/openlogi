//! Profile storage.
//!
//! Profiles are host-side for every device we support: the Signature M650 has
//! neither `0x1C00` (persistent remappable action) nor `0x8100` (onboard
//! profiles), so there is nowhere on the device to put them.
//!
//! This is the in-memory shape only. Persistence to SQLite lands with the
//! daemon; until then the defaults are returned so the UI has something
//! truthful to render rather than failing.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionDto {
    pub kind: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MappingDto {
    pub cid: u16,
    pub action: ActionDto,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDto {
    pub id: String,
    pub name: String,
    /// Application that activates this profile; `None` is the fallback.
    pub app_match: Option<String>,
    pub color: String,
    pub mappings: Vec<MappingDto>,
    pub is_default: bool,
}

fn action(kind: &str, label: &str, detail: Option<&str>) -> ActionDto {
    ActionDto {
        kind: kind.to_string(),
        label: label.to_string(),
        detail: detail.map(str::to_string),
    }
}

pub fn defaults() -> Vec<ProfileDto> {
    vec![ProfileDto {
        id: "default".into(),
        name: "Default".into(),
        app_match: None,
        color: "#00a651".into(),
        is_default: true,
        mappings: vec![
            MappingDto { cid: 0x0052, action: action("default", "Middle Click", None) },
            MappingDto { cid: 0x0053, action: action("default", "Back", None) },
            MappingDto { cid: 0x0056, action: action("default", "Forward", None) },
        ],
    }]
}
