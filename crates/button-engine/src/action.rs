//! What a remapped button does.

use serde::{Deserialize, Serialize};

use crate::desktop::{Direction, SwitchMode};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Action {
    /// Leave the button alone — the firmware handles it. Un-diverts.
    Default,
    /// Divert the button and do nothing, making it inert.
    Disabled,
    /// Send a keyboard shortcut, e.g. `ctrl+shift+p`.
    Keystroke { shortcut: String },
    /// A single media/system key, e.g. `playpause`.
    Media { key: String },
    /// Launch a program. Argument-split on spaces; no shell involved.
    Launch { command: String },
    /// Run a command through the user's shell.
    Command { script: String },
    /// Open a URL with the desktop's default handler.
    Url { url: String },
    /// Emit a standard mouse button.
    MouseButton { button: String },
    /// Switch to the adjacent workspace, optionally taking the focused window.
    ///
    /// Stored as intent rather than as a key chord, so the shortcut is
    /// resolved from the live desktop settings at press time — a user who
    /// rebinds their workspace keys does not have to redo their mouse mapping.
    Workspace {
        direction: Direction,
        #[serde(rename = "moveWindow", default)]
        move_window: bool,
    },
    /// Invoke an application or window switcher. Like `Workspace`, this is
    /// stored as intent and resolved from the desktop's live settings.
    AppSwitch {
        mode: SwitchMode,
        #[serde(default)]
        backward: bool,
    },
}

impl Action {
    /// Does this action require the firmware to stop sending the button's
    /// normal report?
    pub fn needs_diversion(&self) -> bool {
        !matches!(self, Action::Default)
    }

    /// Short human label, used when the UI has no better string.
    pub fn label(&self) -> String {
        match self {
            Action::Default => "Default".into(),
            Action::Disabled => "Disabled".into(),
            Action::Keystroke { shortcut } => shortcut.clone(),
            Action::Media { key } => key.clone(),
            Action::Launch { command } => command.clone(),
            Action::Command { script } => script.clone(),
            Action::Url { url } => url.clone(),
            Action::MouseButton { button } => button.clone(),
            Action::Workspace { direction, move_window } => {
                let what = if *move_window { "Move window" } else { "Workspace" };
                format!("{what} {:?}", direction).to_lowercase()
            }
            Action::AppSwitch { mode, backward } => {
                let base = match mode {
                    SwitchMode::Applications => "Switch apps",
                    SwitchMode::Windows => "Switch windows",
                    SwitchMode::WindowsOfApp => "Windows of app",
                    SwitchMode::Overview => "Overview",
                    SwitchMode::AppGrid => "App grid",
                };
                if *backward && !matches!(mode, SwitchMode::Overview | SwitchMode::AppGrid) {
                    format!("{base} (back)")
                } else {
                    base.to_string()
                }
            }
        }
    }
}
