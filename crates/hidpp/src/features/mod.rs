//! Typed wrappers over individual HID++ features.
//!
//! Each module turns one feature's raw `(function, params) -> params` calls
//! into a Rust API. Keeping them separate means adding support for a new
//! Logitech capability is an additive change, never a rewrite.

pub mod battery;
pub mod controls;
pub mod dpi;
pub mod info;
