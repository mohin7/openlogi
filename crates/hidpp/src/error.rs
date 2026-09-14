use std::fmt;

/// Error codes defined by HID++ 2.0 (returned in an `0xFF` error report).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hidpp20Error {
    NoError,
    Unknown,
    InvalidArgument,
    OutOfRange,
    HardwareError,
    LogitechInternal,
    InvalidFeatureIndex,
    InvalidFunctionId,
    Busy,
    Unsupported,
    Other(u8),
}

impl From<u8> for Hidpp20Error {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::NoError,
            1 => Self::Unknown,
            2 => Self::InvalidArgument,
            3 => Self::OutOfRange,
            4 => Self::HardwareError,
            5 => Self::LogitechInternal,
            6 => Self::InvalidFeatureIndex,
            7 => Self::InvalidFunctionId,
            8 => Self::Busy,
            9 => Self::Unsupported,
            other => Self::Other(other),
        }
    }
}

impl fmt::Display for Hidpp20Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::NoError => "no error",
            Self::Unknown => "unknown",
            Self::InvalidArgument => "invalid argument",
            Self::OutOfRange => "out of range",
            Self::HardwareError => "hardware error",
            Self::LogitechInternal => "logitech internal",
            Self::InvalidFeatureIndex => "invalid feature index",
            Self::InvalidFunctionId => "invalid function id",
            Self::Busy => "busy",
            Self::Unsupported => "unsupported",
            Self::Other(c) => return write!(f, "error code 0x{c:02x}"),
        };
        f.write_str(s)
    }
}

/// Error codes defined by HID++ 1.0 (returned in an `0x8F` error report).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hidpp10Error(pub u8);

impl fmt::Display for Hidpp10Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.0 {
            1 => "invalid sub id",
            2 => "invalid address",
            3 => "invalid value",
            4 => "connection failed",
            5 => "too many devices",
            6 => "already exists",
            7 => "busy",
            8 => "unknown device",
            9 => "resource error",
            10 => "request unavailable",
            11 => "invalid parameter value",
            12 => "wrong pin code",
            c => return write!(f, "hid++1.0 error 0x{c:02x}"),
        };
        f.write_str(s)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("hidapi: {0}")]
    Hid(#[from] hidapi::HidError),

    #[error("no Logitech HID++ interface found")]
    NoInterface,

    #[error("timed out waiting for a reply from device {device_index}")]
    Timeout { device_index: u8 },

    #[error("device {device_index} is not responding (unreachable or unpaired)")]
    Unreachable { device_index: u8 },

    #[error("feature 0x{feature_id:04x} is not supported by this device")]
    FeatureUnsupported { feature_id: u16 },

    #[error("HID++ 2.0 error on feature 0x{feature_id:04x} fn {function}: {source}")]
    Protocol20 {
        feature_id: u16,
        function: u8,
        #[source]
        source: Hidpp20ErrorWrapper,
    },

    #[error("HID++ 1.0 error: {0}")]
    Protocol10(Hidpp10Error),

    #[error("malformed reply: {0}")]
    Malformed(&'static str),

    #[error("permission denied opening {path} — run scripts/install-udev-rules.sh")]
    Permission { path: String },
}

/// Newtype so `Hidpp20Error` can be used as a `#[source]`.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Hidpp20ErrorWrapper(pub Hidpp20Error);

impl Hidpp10Error {
    pub fn code(self) -> u8 {
        self.0
    }
}

pub type Result<T> = std::result::Result<T, Error>;
