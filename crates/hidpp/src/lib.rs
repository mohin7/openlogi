//! # hidpp
//!
//! A Linux implementation of Logitech's HID++ 1.0/2.0 protocol, written for
//! OpenLogi but usable on its own.
//!
//! ## Layering
//!
//! ```text
//!   features::{battery, dpi, controls, info}   typed, per-feature APIs
//!            │
//!   Device                                     feature-index cache, call()
//!            │
//!   Transport                                  hidraw I/O, reply matching
//!            │
//!   Packet                                     the 7/20-byte wire format
//! ```
//!
//! ## Minimal use
//!
//! ```no_run
//! use hidpp::{discover, Device, Transport};
//!
//! let api = hidapi::HidApi::new()?;
//! let endpoint = discover(&api).into_iter().next().expect("no receiver");
//! let mut transport = Transport::open(&api, endpoint)?;
//!
//! for index in 1..=6 {
//!     if let Some(mut dev) = Device::probe(&mut transport, index)? {
//!         let name = hidpp::features::info::name(&mut dev, &mut transport)?;
//!         let battery = hidpp::features::battery::read(&mut dev, &mut transport)?;
//!         println!("{name}: {:?}%", battery.approx_percentage());
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#[macro_use]
mod bitflags_lite;

pub mod device;
pub mod error;
pub mod features;
pub mod hidpp1;
pub mod ids;
pub mod protocol;
pub mod transport;

pub use device::{Device, FeatureEntry, ProtocolVersion};
pub use error::{Error, Result};
pub use protocol::Packet;
pub use transport::{discover, Endpoint, Transport, LOGITECH_VID};

/// Maximum number of devices a Logitech receiver can pair.
pub const MAX_PAIRED_DEVICES: u8 = 6;
