//! The virtual input device.
//!
//! Remapped buttons are delivered by creating a `uinput` device and emitting
//! events into it. The kernel then presents it as an ordinary input device, so
//! the events are indistinguishable from a real keyboard's — which is why this
//! approach works identically on X11 and Wayland, where client-side injection
//! (XTEST and friends) does not.
//!
//! uinput requires a device to declare its entire capability set at creation
//! time; anything not declared is silently dropped when emitted. Mappings
//! change at runtime, so we declare every key we could ever send.

use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, EventType, InputEvent, Key};
use tracing::{debug, warn};

use crate::action::Action;
use crate::desktop;
use crate::keys;

#[derive(Debug, thiserror::Error)]
pub enum EmitError {
    #[error(
        "cannot open /dev/uinput: {0}.\n\
         OpenLogi needs membership of the 'openlogi' group. If it was just installed, \
         log out and back in — group changes only apply to new sessions."
    )]
    Open(#[source] std::io::Error),

    #[error("emitting input failed: {0}")]
    Emit(#[source] std::io::Error),

    #[error("{0}")]
    BadAction(String),
}

/// How long to wait after creating the device before it is usable.
const SETTLE: Duration = Duration::from_millis(250);

/// Gap between individual key transitions.
///
/// The kernel is happy to accept a press and release in the same microsecond,
/// but some toolkits sample input rather than reacting to every event and can
/// miss a chord that appears instantaneous. A few milliseconds is imperceptible
/// to a user and removes the whole class of problem.
const KEY_GAP: Duration = Duration::from_millis(4);

pub struct Emitter {
    device: VirtualDevice,
}

impl Emitter {
    pub fn new() -> Result<Self, EmitError> {
        let mut set = AttributeSet::<Key>::new();
        for k in keys::all_supported() {
            set.insert(k);
        }

        let device = VirtualDeviceBuilder::new()
            .map_err(EmitError::Open)?
            .name("OpenLogi Virtual Input")
            .with_keys(&set)
            .map_err(EmitError::Open)?
            .build()
            .map_err(EmitError::Open)?;

        // Give udev and the compositor's input stack time to notice the new
        // device. Events emitted before libinput has finished adding it are
        // accepted by the kernel and then dropped, which looks exactly like a
        // broken mapping.
        sleep(SETTLE);

        debug!("uinput virtual device created");
        Ok(Self { device })
    }

    fn key_event(&mut self, key: Key, pressed: bool) -> Result<(), EmitError> {
        self.device
            .emit(&[InputEvent::new(
                EventType::KEY,
                key.code(),
                if pressed { 1 } else { 0 },
            )])
            .map_err(EmitError::Emit)
    }

    /// Press modifiers, tap the key, then release in reverse order.
    ///
    /// Reverse order matters: releasing a modifier before the key it modifies
    /// can be seen by applications as a bare keypress.
    pub fn tap_shortcut(&mut self, spec: &str) -> Result<(), EmitError> {
        let (modifiers, main) = keys::parse_shortcut(spec).map_err(EmitError::BadAction)?;

        for m in &modifiers {
            self.key_event(*m, true)?;
            sleep(KEY_GAP);
        }
        self.key_event(main, true)?;
        sleep(KEY_GAP);
        self.key_event(main, false)?;
        sleep(KEY_GAP);
        for m in modifiers.iter().rev() {
            self.key_event(*m, false)?;
            sleep(KEY_GAP);
        }
        Ok(())
    }

    pub fn tap_key(&mut self, name: &str) -> Result<(), EmitError> {
        let k = keys::key(name)
            .ok_or_else(|| EmitError::BadAction(format!("unknown key '{name}'")))?;
        self.key_event(k, true)?;
        sleep(KEY_GAP);
        self.key_event(k, false)
    }

    pub fn click(&mut self, button: &str) -> Result<(), EmitError> {
        let b = match button.to_ascii_lowercase().as_str() {
            "left" => Key::BTN_LEFT,
            "right" => Key::BTN_RIGHT,
            "middle" => Key::BTN_MIDDLE,
            "back" => Key::BTN_SIDE,
            "forward" => Key::BTN_EXTRA,
            other => return Err(EmitError::BadAction(format!("unknown button '{other}'"))),
        };
        self.key_event(b, true)?;
        sleep(KEY_GAP);
        self.key_event(b, false)
    }

    /// Run an action. Process-spawning actions are detached so a long-running
    /// program can never block the device thread servicing the next button.
    pub fn perform(&mut self, action: &Action) -> Result<(), EmitError> {
        match action {
            Action::Default | Action::Disabled => Ok(()),
            Action::Keystroke { shortcut } => self.tap_shortcut(shortcut),
            Action::Media { key } => self.tap_key(key),
            Action::MouseButton { button } => self.click(button),
            Action::Launch { command } => {
                let mut parts = command.split_whitespace();
                let program = parts
                    .next()
                    .ok_or_else(|| EmitError::BadAction("empty command".into()))?;
                spawn_detached(Command::new(program).args(parts));
                Ok(())
            }
            Action::Command { script } => {
                spawn_detached(Command::new("sh").arg("-c").arg(script));
                Ok(())
            }
            Action::Url { url } => {
                spawn_detached(Command::new("xdg-open").arg(url));
                Ok(())
            }
            Action::Workspace { direction, move_window } => {
                let shortcut = desktop::workspace_shortcut(*direction, *move_window);
                self.tap_shortcut(&shortcut)
            }
            Action::AppSwitch { mode, backward } => {
                let shortcut = desktop::switch_shortcut(*mode, *backward);
                self.tap_shortcut(&shortcut)
            }
        }
    }
}

fn spawn_detached(cmd: &mut Command) {
    match cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        // The child is intentionally not awaited. Reaping is left to the
        // process' eventual exit; these are short-lived launchers.
        Ok(child) => debug!(pid = child.id(), "spawned"),
        Err(e) => warn!("spawn failed: {e}"),
    }
}
