//! # button-engine
//!
//! The half of remapping that HID++ cannot do.
//!
//! `hidpp` can tell a device to stop reporting a button (diversion) and can
//! deliver the resulting notifications. That makes the button *silent* — it
//! does not make it do anything. This crate supplies the other half: a
//! `uinput` virtual device that synthesises whatever the user asked for.
//!
//! ```text
//!   physical press
//!        │
//!   firmware sees the control is diverted
//!        │
//!   0x1B04 notification ──► hidpp ──► ButtonEngine::on_press(cid)
//!                                          │
//!                                     Action lookup
//!                                          │
//!                                   Emitter ──► /dev/uinput ──► desktop
//! ```
//!
//! The engine holds no hardware handle of its own; the caller owns the HID++
//! transport and feeds it events. That keeps this crate testable without a
//! mouse attached.

pub mod action;
pub mod desktop;
pub mod emitter;
pub mod keys;

use std::collections::HashMap;

use tracing::{debug, warn};

pub use action::Action;
pub use desktop::{Direction, SwitchMode};
pub use emitter::{EmitError, Emitter};

/// Maps control ids to actions and performs them.
pub struct ButtonEngine {
    mappings: HashMap<u16, Action>,
    emitter: Option<Emitter>,
    /// Why the emitter is unavailable, if it is. Surfaced to the UI so a
    /// missing group membership reads as an explanation, not as silence.
    unavailable: Option<String>,
}

impl ButtonEngine {
    /// Build an engine, attempting to open uinput.
    ///
    /// A failure here is deliberately **not** fatal: reading battery and
    /// setting DPI still work without input synthesis, so the app stays useful
    /// and reports what is degraded rather than refusing to start.
    pub fn new() -> Self {
        match Emitter::new() {
            Ok(emitter) => Self {
                mappings: HashMap::new(),
                emitter: Some(emitter),
                unavailable: None,
            },
            Err(e) => {
                warn!("input synthesis unavailable: {e}");
                Self {
                    mappings: HashMap::new(),
                    emitter: None,
                    unavailable: Some(e.to_string()),
                }
            }
        }
    }

    pub fn is_available(&self) -> bool {
        self.emitter.is_some()
    }

    pub fn unavailable_reason(&self) -> Option<&str> {
        self.unavailable.as_deref()
    }

    pub fn set(&mut self, cid: u16, action: Action) {
        if matches!(action, Action::Default) {
            self.mappings.remove(&cid);
        } else {
            self.mappings.insert(cid, action);
        }
    }

    pub fn get(&self, cid: u16) -> Option<&Action> {
        self.mappings.get(&cid)
    }

    pub fn mappings(&self) -> &HashMap<u16, Action> {
        &self.mappings
    }

    /// Control ids that must be diverted for the current mapping set.
    pub fn diverted_cids(&self) -> Vec<u16> {
        self.mappings
            .iter()
            .filter(|(_, a)| a.needs_diversion())
            .map(|(cid, _)| *cid)
            .collect()
    }

    /// Handle a press of a diverted control.
    pub fn on_press(&mut self, cid: u16) {
        let Some(action) = self.mappings.get(&cid).cloned() else {
            return;
        };
        debug!(cid = format!("0x{cid:04x}"), ?action, "button press");

        let Some(emitter) = self.emitter.as_mut() else {
            warn!("dropping press: input synthesis unavailable");
            return;
        };
        if let Err(e) = emitter.perform(&action) {
            warn!("action failed: {e}");
        }
    }
}

impl Default for ButtonEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_action_clears_the_mapping() {
        let mut e = ButtonEngine {
            mappings: HashMap::new(),
            emitter: None,
            unavailable: Some("test".into()),
        };
        e.set(0x53, Action::Keystroke { shortcut: "ctrl+z".into() });
        assert!(e.get(0x53).is_some());
        e.set(0x53, Action::Default);
        assert!(e.get(0x53).is_none(), "Default must un-map, not store a no-op");
    }

    #[test]
    fn only_non_default_actions_need_diversion() {
        let mut e = ButtonEngine {
            mappings: HashMap::new(),
            emitter: None,
            unavailable: None,
        };
        e.set(0x52, Action::Disabled);
        e.set(0x53, Action::Keystroke { shortcut: "ctrl+z".into() });
        let mut cids = e.diverted_cids();
        cids.sort_unstable();
        assert_eq!(cids, vec![0x52, 0x53]);
    }

    #[test]
    fn a_press_without_an_emitter_is_dropped_quietly() {
        let mut e = ButtonEngine {
            mappings: HashMap::new(),
            emitter: None,
            unavailable: Some("no uinput".into()),
        };
        e.set(0x53, Action::Keystroke { shortcut: "ctrl+z".into() });
        e.on_press(0x53); // must not panic
        assert!(!e.is_available());
    }
}
