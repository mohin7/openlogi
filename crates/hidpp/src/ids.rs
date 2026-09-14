//! HID++ 2.0 feature identifiers.
//!
//! A feature *id* is a stable, Logitech-assigned constant. A feature *index*
//! is a slot number that differs per device and per firmware, and must be
//! resolved at runtime through the root feature. Never hard-code an index.

macro_rules! features {
    ($($konst:ident = $id:expr, $name:expr;)*) => {
        $(pub const $konst: u16 = $id;)*

        /// Human-readable name for a feature id, for diagnostics and the
        /// device-capabilities view in the UI.
        pub fn feature_name(id: u16) -> &'static str {
            match id {
                $($id => $name,)*
                _ => classify(id),
            }
        }
    };
}

features! {
    ROOT                      = 0x0000, "Root";
    FEATURE_SET               = 0x0001, "Feature Set";
    FEATURE_INFO              = 0x0002, "Feature Info";
    DEVICE_FW_VERSION         = 0x0003, "Device FW Version";
    DEVICE_UNIT_ID            = 0x0004, "Device Unit ID";
    DEVICE_NAME               = 0x0005, "Device Name & Type";
    DEVICE_GROUPS             = 0x0006, "Device Groups";
    DEVICE_FRIENDLY_NAME      = 0x0007, "Device Friendly Name";
    KEEP_ALIVE                = 0x0008, "Keep Alive";
    CONFIG_CHANGE             = 0x0020, "Config Change";
    DFU_CONTROL               = 0x00C3, "DFU Control";
    BATTERY_STATUS            = 0x1000, "Battery Status";
    BATTERY_VOLTAGE           = 0x1001, "Battery Voltage";
    UNIFIED_BATTERY           = 0x1004, "Unified Battery";
    CHARGING_CONTROL          = 0x1010, "Charging Control";
    LED_CONTROL               = 0x1300, "LED Control";
    HOST_INFO                 = 0x1815, "Host Info";
    WIRELESS_DEVICE_STATUS    = 0x1D4B, "Wireless Device Status";
    RESET                     = 0x1802, "Reset";
    CHANGE_HOST               = 0x1814, "Change Host";
    BACKLIGHT2                = 0x1982, "Backlight v2";
    REPROG_CONTROLS_V4        = 0x1B04, "Reprogrammable Controls v4";
    PERSISTENT_REMAPPABLE_ACTION = 0x1C00, "Persistent Remappable Action";
    VERTICAL_SCROLLING        = 0x2100, "Vertical Scrolling";
    SMART_SHIFT               = 0x2110, "Smart Shift";
    SMART_SHIFT_ENHANCED      = 0x2111, "Smart Shift Enhanced";
    HI_RES_SCROLLING          = 0x2120, "Hi-Res Scrolling";
    LOWRES_WHEEL              = 0x2130, "Low-Res Wheel";
    HIRES_WHEEL               = 0x2121, "Hi-Res Wheel";
    THUMB_WHEEL               = 0x2150, "Thumb Wheel";
    MOUSE_POINTER             = 0x2200, "Mouse Pointer";
    ADJUSTABLE_DPI            = 0x2201, "Adjustable DPI";
    EXTENDED_ADJUSTABLE_DPI   = 0x2202, "Extended Adjustable DPI";
    POINTER_SPEED             = 0x2205, "Pointer Speed";
    ANGLE_SNAPPING            = 0x2230, "Angle Snapping";
    REPORT_RATE               = 0x8060, "Report Rate";
    EXTENDED_REPORT_RATE      = 0x8061, "Extended Report Rate";
    ONBOARD_PROFILES          = 0x8100, "Onboard Profiles";
    MOUSE_BUTTON_SPY          = 0x8110, "Mouse Button Spy";
    GESTURE_2                 = 0x6501, "Gesture v2";
    TOUCHPAD_RAW_XY           = 0x6100, "Touchpad Raw XY";
    ANALYTICS_DATA            = 0x2250, "Analytics Data";
    PASSWORD                  = 0x1602, "Password";
    GPIO_ACCESS               = 0x1803, "GPIO Access";
    OOB_STATE                 = 0x1805, "Out-of-Box State";
    CONFIGURABLE_PROPERTIES   = 0x1806, "Configurable Device Properties";
    BLE_PRO_PREPAIRING        = 0x1816, "BLE Pro Prepairing";
    LED_TEST                  = 0x18A1, "LED Test";
    MONITOR_MODE              = 0x18B1, "Monitor Mode";
    ENABLE_HIDDEN_FEATURES    = 0x1E00, "Enable Hidden Features";
    MANAGE_DEACTIVATABLE_AUTH = 0x1E02, "Manage Deactivatable Features";
    SPI_DIRECT_ACCESS         = 0x1E22, "SPI Direct Access";
    TDE_ACCESS                = 0x1EB0, "TDE Access";
}

/// Fallback classification for ids we have no specific name for.
///
/// Logitech groups features into bands, and the band alone is useful: anything
/// in the manufacturing/test range is guaranteed not to be user-configurable,
/// so diagnostics can say so confidently without us having reversed each one.
fn classify(id: u16) -> &'static str {
    match id {
        0x1800..=0x19FF | 0x1E00..=0x1EFF => "manufacturing/test",
        0x9000..=0x9FFF => "test",
        0x0000..=0x00FF => "common",
        0x1000..=0x1FFF => "device capability",
        0x2000..=0x2FFF => "mouse capability",
        0x4000..=0x4FFF => "keyboard capability",
        0x6000..=0x6FFF => "touchpad capability",
        0x8000..=0x8FFF => "gaming capability",
        _ => "unknown",
    }
}

/// Device type reported by feature `0x0005` function 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    Keyboard,
    RemoteControl,
    Numpad,
    Mouse,
    Touchpad,
    Trackball,
    Presenter,
    Receiver,
    Headset,
    Unknown(u8),
}

impl From<u8> for DeviceKind {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Keyboard,
            1 => Self::RemoteControl,
            2 => Self::Numpad,
            3 => Self::Mouse,
            4 => Self::Touchpad,
            5 => Self::Trackball,
            6 => Self::Presenter,
            7 => Self::Receiver,
            8 => Self::Headset,
            other => Self::Unknown(other),
        }
    }
}

impl std::fmt::Display for DeviceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(v) => write!(f, "unknown({v})"),
            other => write!(f, "{}", format!("{other:?}").to_lowercase()),
        }
    }
}
