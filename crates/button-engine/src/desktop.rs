//! Desktop-environment integration.
//!
//! Workspace switching is not a device capability — it is whatever key chord
//! the user's window manager happens to listen for. Hard-coding `Ctrl+Alt+Left`
//! would break for anyone who has rebound it, and differs between desktops.
//!
//! So on GNOME we *ask*: `org.gnome.desktop.wm.keybindings` holds the live
//! bindings, including any the user changed. Everything else falls back to the
//! near-universal `Ctrl+Alt+<arrow>` convention.

use std::collections::HashMap;
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::keys;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn as_str(self) -> &'static str {
        match self {
            Direction::Left => "left",
            Direction::Right => "right",
            Direction::Up => "up",
            Direction::Down => "down",
        }
    }
}

/// Which switcher a button should invoke.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum SwitchMode {
    /// Cycle applications — GNOME's Super+Tab. A quick tap returns to the
    /// previously used app, which is the useful behaviour from a mouse button.
    Applications,
    /// Cycle individual windows rather than applications (Alt+Tab).
    Windows,
    /// Cycle windows of the *current* application (Super+`).
    WindowsOfApp,
    /// The overview / "mission control" view.
    Overview,
    /// The application grid.
    AppGrid,
}

/// Resolve the shortcut for an application/window switcher.
pub fn switch_shortcut(mode: SwitchMode, backward: bool) -> String {
    static CACHE: OnceLock<Mutex<HashMap<(SwitchMode, bool), String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Ok(map) = cache.lock() {
        if let Some(hit) = map.get(&(mode, backward)) {
            return hit.clone();
        }
    }

    let resolved = resolve_switch(mode, backward);
    debug!(?mode, backward, %resolved, "switch shortcut resolved");
    if let Ok(mut map) = cache.lock() {
        map.insert((mode, backward), resolved.clone());
    }
    resolved
}

fn resolve_switch(mode: SwitchMode, backward: bool) -> String {
    let (schema, key, fallback) = match (mode, backward) {
        (SwitchMode::Applications, false) => (
            "org.gnome.desktop.wm.keybindings", "switch-applications", "alt+tab",
        ),
        (SwitchMode::Applications, true) => (
            "org.gnome.desktop.wm.keybindings", "switch-applications-backward", "shift+alt+tab",
        ),
        (SwitchMode::Windows, false) => (
            "org.gnome.desktop.wm.keybindings", "switch-windows", "alt+tab",
        ),
        (SwitchMode::Windows, true) => (
            "org.gnome.desktop.wm.keybindings", "switch-windows-backward", "shift+alt+tab",
        ),
        (SwitchMode::WindowsOfApp, false) => (
            "org.gnome.desktop.wm.keybindings", "switch-group", "alt+grave",
        ),
        (SwitchMode::WindowsOfApp, true) => (
            "org.gnome.desktop.wm.keybindings", "switch-group-backward", "shift+alt+grave",
        ),
        // GNOME ships toggle-overview unset because the overview opens by
        // tapping Super on its own; the fallback is the real behaviour here,
        // not a guess.
        (SwitchMode::Overview, _) => (
            "org.gnome.shell.keybindings", "toggle-overview", "super",
        ),
        (SwitchMode::AppGrid, _) => (
            "org.gnome.shell.keybindings", "toggle-application-view", "super+a",
        ),
    };

    first_binding(schema, key).unwrap_or_else(|| fallback.to_string())
}

/// First binding from a gsettings key that the emitter can actually produce.
fn first_binding(schema: &str, key: &str) -> Option<String> {
    let raw = gsettings(schema, key)?;
    raw.split(',')
        .filter_map(|entry| {
            parse_accelerator(entry.trim().trim_matches(['[', ']', '\'', ' '].as_ref()))
        })
        .next()
}

/// Resolve the shortcut for switching to, or moving a window to, the adjacent
/// workspace. Results are memoised — each lookup spawns `gsettings`.
pub fn workspace_shortcut(direction: Direction, move_window: bool) -> String {
    static CACHE: OnceLock<Mutex<HashMap<(Direction, bool), String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Ok(map) = cache.lock() {
        if let Some(hit) = map.get(&(direction, move_window)) {
            return hit.clone();
        }
    }

    let resolved = resolve(direction, move_window);
    debug!(?direction, move_window, %resolved, "workspace shortcut resolved");
    if let Ok(mut map) = cache.lock() {
        map.insert((direction, move_window), resolved.clone());
    }
    resolved
}

fn fallback(direction: Direction, move_window: bool) -> String {
    if move_window {
        format!("ctrl+alt+shift+{}", direction.as_str())
    } else {
        format!("ctrl+alt+{}", direction.as_str())
    }
}

fn resolve(direction: Direction, move_window: bool) -> String {
    let key = format!(
        "{}-to-workspace-{}",
        if move_window { "move" } else { "switch" },
        direction.as_str()
    );

    // The value is a GVariant array: ['<Super>Page_Up', '<Control><Alt>Left'],
    // ordered by the desktop's own preference.
    first_binding("org.gnome.desktop.wm.keybindings", &key)
        .unwrap_or_else(|| fallback(direction, move_window))
}

fn gsettings(schema: &str, key: &str) -> Option<String> {
    let out = Command::new("gsettings").arg("get").arg(schema).arg(key).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let text = text.trim();
    // An unset or empty binding reads as @as [] or [''].
    if text.is_empty() || text == "@as []" || text == "[]" {
        return None;
    }
    Some(text.to_string())
}

/// Convert a GTK accelerator such as `<Control><Alt>Left` into our shortcut
/// syntax (`ctrl+alt+left`). Returns `None` if any part is not representable,
/// so the caller can try the next binding.
fn parse_accelerator(accel: &str) -> Option<String> {
    if accel.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();
    let mut rest = accel;

    while let Some(open) = rest.find('<') {
        let close = rest[open..].find('>')? + open;
        let modifier = &rest[open + 1..close];
        let mapped = match modifier.to_ascii_lowercase().as_str() {
            // GTK's "Primary" means Control on Linux.
            "control" | "ctrl" | "primary" => "ctrl",
            "alt" | "mod1" => "alt",
            "shift" => "shift",
            "super" | "meta" | "mod4" => "super",
            _ => return None,
        };
        if !parts.iter().any(|p| p == mapped) {
            parts.push(mapped.to_string());
        }
        rest = &rest[close + 1..];
    }

    let key_name = rest.trim();
    if key_name.is_empty() {
        return None;
    }

    // GTK key names to ours: Page_Up -> pageup, Left -> left.
    let normalised = key_name.to_ascii_lowercase().replace('_', "");
    // Only accept it if the emitter can actually produce this key.
    keys::key(&normalised)?;

    parts.push(normalised);
    Some(parts.join("+"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gtk_accelerators() {
        assert_eq!(parse_accelerator("<Control><Alt>Left").as_deref(), Some("ctrl+alt+left"));
        assert_eq!(parse_accelerator("<Super>Page_Up").as_deref(), Some("super+pageup"));
        assert_eq!(
            parse_accelerator("<Control><Shift><Alt>Right").as_deref(),
            Some("ctrl+shift+alt+right")
        );
    }

    #[test]
    fn primary_is_control_on_linux() {
        assert_eq!(parse_accelerator("<Primary>Left").as_deref(), Some("ctrl+left"));
    }

    #[test]
    fn rejects_keys_the_emitter_cannot_produce() {
        // Not in our key table, so the caller should try the next binding.
        assert_eq!(parse_accelerator("<Super>Launch5"), None);
        assert_eq!(parse_accelerator(""), None);
    }

    #[test]
    fn every_switch_fallback_is_emittable() {
        for mode in [
            SwitchMode::Applications,
            SwitchMode::Windows,
            SwitchMode::WindowsOfApp,
            SwitchMode::Overview,
            SwitchMode::AppGrid,
        ] {
            for backward in [false, true] {
                let s = resolve_switch(mode, backward);
                assert!(keys::parse_shortcut(&s).is_ok(), "{mode:?}/{backward} -> {s}");
            }
        }
    }

    #[test]
    fn fallback_is_a_valid_shortcut() {
        for d in [Direction::Left, Direction::Right] {
            for mv in [false, true] {
                let s = fallback(d, mv);
                assert!(keys::parse_shortcut(&s).is_ok(), "{s} must parse");
            }
        }
    }
}
