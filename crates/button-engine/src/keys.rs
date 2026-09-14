//! Key-name parsing.
//!
//! Shortcuts arrive from the UI as human strings like `Ctrl+Shift+P`. They are
//! resolved to Linux `KEY_*` codes here, once, at assignment time — never in
//! the hot path where a button press is being serviced.
//!
//! Note these are *scancode-level* keys, not characters. `KEY_P` is "the key
//! engraved P on a US layout"; what it types depends on the user's active
//! keyboard layout. That is the same model Logitech's own software uses, and
//! it is why the UI records physical keypresses rather than asking for text.

use evdev::Key;

/// Parse a shortcut such as `ctrl+shift+p` or `super+Left`.
///
/// Returns the modifier keys and the single non-modifier key, so the emitter
/// can press modifiers first and release them last.
pub fn parse_shortcut(spec: &str) -> Result<(Vec<Key>, Key), String> {
    let mut modifiers = Vec::new();
    let mut main = None;

    for part in spec.split('+').map(str::trim).filter(|s| !s.is_empty()) {
        match modifier(part) {
            Some(m) => {
                if !modifiers.contains(&m) {
                    modifiers.push(m);
                }
            }
            None => {
                if main.is_some() {
                    return Err(format!("shortcut '{spec}' has more than one non-modifier key"));
                }
                main = Some(key(part).ok_or_else(|| format!("unknown key '{part}'"))?);
            }
        }
    }

    if let Some(m) = main {
        return Ok((modifiers, m));
    }
    // A lone modifier is a legitimate chord: tapping Super alone opens the
    // GNOME overview, and several desktops bind bare modifiers this way. Only
    // a single one is meaningful — "ctrl+shift" with nothing else is not a
    // shortcut anyone can trigger.
    match modifiers.len() {
        1 => Ok((Vec::new(), modifiers[0])),
        _ => Err(format!("shortcut '{spec}' has no non-modifier key")),
    }
}

fn modifier(name: &str) -> Option<Key> {
    Some(match name.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => Key::KEY_LEFTCTRL,
        "rctrl" => Key::KEY_RIGHTCTRL,
        "shift" => Key::KEY_LEFTSHIFT,
        "rshift" => Key::KEY_RIGHTSHIFT,
        "alt" => Key::KEY_LEFTALT,
        "altgr" | "ralt" => Key::KEY_RIGHTALT,
        "super" | "meta" | "cmd" | "win" => Key::KEY_LEFTMETA,
        _ => return None,
    })
}

/// Resolve a single key name. Case-insensitive.
pub fn key(name: &str) -> Option<Key> {
    let lower = name.to_ascii_lowercase();

    // Letters must be looked up, not computed. Linux key codes follow the
    // physical QWERTY layout, not the alphabet: KEY_Q is 16 and KEY_A is 30,
    // so `KEY_A + 15` lands on KEY_X rather than KEY_P.
    if lower.len() == 1 {
        let c = lower.as_bytes()[0];
        if c.is_ascii_lowercase() {
            return Some(match c {
                b'a' => Key::KEY_A, b'b' => Key::KEY_B, b'c' => Key::KEY_C,
                b'd' => Key::KEY_D, b'e' => Key::KEY_E, b'f' => Key::KEY_F,
                b'g' => Key::KEY_G, b'h' => Key::KEY_H, b'i' => Key::KEY_I,
                b'j' => Key::KEY_J, b'k' => Key::KEY_K, b'l' => Key::KEY_L,
                b'm' => Key::KEY_M, b'n' => Key::KEY_N, b'o' => Key::KEY_O,
                b'p' => Key::KEY_P, b'q' => Key::KEY_Q, b'r' => Key::KEY_R,
                b's' => Key::KEY_S, b't' => Key::KEY_T, b'u' => Key::KEY_U,
                b'v' => Key::KEY_V, b'w' => Key::KEY_W, b'x' => Key::KEY_X,
                b'y' => Key::KEY_Y, _ => Key::KEY_Z,
            });
        }
        // Digits *are* contiguous: KEY_1=2 .. KEY_9=10, with KEY_0=11 after.
        if c.is_ascii_digit() {
            return Some(match c {
                b'0' => Key::KEY_0,
                _ => Key::new(Key::KEY_1.code() + (c - b'1') as u16),
            });
        }
    }

    // Function keys.
    if let Some(n) = lower.strip_prefix('f').and_then(|d| d.parse::<u8>().ok()) {
        return match n {
            1..=10 => Some(Key::new(Key::KEY_F1.code() + (n - 1) as u16)),
            11 => Some(Key::KEY_F11),
            12 => Some(Key::KEY_F12),
            _ => None,
        };
    }

    Some(match lower.as_str() {
        "escape" | "esc" => Key::KEY_ESC,
        "tab" => Key::KEY_TAB,
        "enter" | "return" => Key::KEY_ENTER,
        "space" => Key::KEY_SPACE,
        "backspace" => Key::KEY_BACKSPACE,
        "delete" | "del" => Key::KEY_DELETE,
        "insert" => Key::KEY_INSERT,
        "home" => Key::KEY_HOME,
        "end" => Key::KEY_END,
        "pageup" | "pgup" => Key::KEY_PAGEUP,
        "pagedown" | "pgdn" => Key::KEY_PAGEDOWN,
        "up" => Key::KEY_UP,
        "down" => Key::KEY_DOWN,
        "left" => Key::KEY_LEFT,
        "right" => Key::KEY_RIGHT,
        "minus" | "-" => Key::KEY_MINUS,
        "equal" | "=" => Key::KEY_EQUAL,
        "comma" | "," => Key::KEY_COMMA,
        "dot" | "period" | "." => Key::KEY_DOT,
        "slash" | "/" => Key::KEY_SLASH,
        "semicolon" | ";" => Key::KEY_SEMICOLON,
        "apostrophe" | "'" => Key::KEY_APOSTROPHE,
        // GTK writes the key above Tab as "Above_Tab"; it is physically grave.
        "grave" | "`" | "abovetab" => Key::KEY_GRAVE,
        "backslash" | "\\" => Key::KEY_BACKSLASH,
        "leftbrace" | "[" => Key::KEY_LEFTBRACE,
        "rightbrace" | "]" => Key::KEY_RIGHTBRACE,
        // Media and system keys.
        "playpause" => Key::KEY_PLAYPAUSE,
        "nexttrack" => Key::KEY_NEXTSONG,
        "prevtrack" => Key::KEY_PREVIOUSSONG,
        "stop" => Key::KEY_STOPCD,
        "volumeup" => Key::KEY_VOLUMEUP,
        "volumedown" => Key::KEY_VOLUMEDOWN,
        "mute" => Key::KEY_MUTE,
        "brightnessup" => Key::KEY_BRIGHTNESSUP,
        "brightnessdown" => Key::KEY_BRIGHTNESSDOWN,
        "screenshot" => Key::KEY_SYSRQ,
        _ => return None,
    })
}

/// Every key the virtual device must declare up front.
///
/// uinput requires a device to advertise its full capability set at creation:
/// a key that was not declared is silently dropped at emit time. Since a
/// mapping can change at runtime, we declare everything we can ever send.
pub fn all_supported() -> Vec<Key> {
    let mut keys = Vec::new();
    // KEY_ESC (1) through KEY_MEDIA (226) covers letters, digits, function
    // keys, navigation, and the media/brightness block.
    for code in Key::KEY_ESC.code()..=Key::KEY_MEDIA.code() {
        keys.push(Key::new(code));
    }
    // Mouse buttons live in a separate range.
    for code in Key::BTN_LEFT.code()..=Key::BTN_TASK.code() {
        keys.push(Key::new(code));
    }
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modifier_combinations() {
        let (mods, k) = parse_shortcut("ctrl+shift+p").unwrap();
        assert_eq!(mods, vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTSHIFT]);
        assert_eq!(k, Key::KEY_P);
    }

    #[test]
    fn every_letter_maps_to_its_own_distinct_key() {
        // Guards the QWERTY-order trap: an arithmetic mapping silently
        // produces the wrong key rather than failing.
        let mut seen = std::collections::HashSet::new();
        for c in b'a'..=b'z' {
            let k = key(&(c as char).to_string()).expect("letter must resolve");
            assert!(seen.insert(k.code()), "duplicate code for '{}'", c as char);
        }
        assert_eq!(key("p"), Some(Key::KEY_P));
        assert_eq!(key("q"), Some(Key::KEY_Q));
        assert_eq!(key("m"), Some(Key::KEY_M));
    }

    #[test]
    fn digits_resolve_arithmetically() {
        assert_eq!(key("a"), Some(Key::KEY_A));
        assert_eq!(key("z"), Some(Key::KEY_Z));
        assert_eq!(key("1"), Some(Key::KEY_1));
        assert_eq!(key("9"), Some(Key::KEY_9));
        assert_eq!(key("0"), Some(Key::KEY_0));
    }

    #[test]
    fn function_keys_span_the_split_range() {
        assert_eq!(key("f1"), Some(Key::KEY_F1));
        assert_eq!(key("f10"), Some(Key::KEY_F10));
        // F11/F12 are not contiguous with F1..F10 in the input event codes.
        assert_eq!(key("f11"), Some(Key::KEY_F11));
        assert_eq!(key("f12"), Some(Key::KEY_F12));
        assert_eq!(key("f13"), None);
    }

    #[test]
    fn rejects_ambiguous_or_empty_shortcuts() {
        assert!(parse_shortcut("a+b").is_err());
        assert!(parse_shortcut("ctrl+nonsense").is_err());
        // Two bare modifiers are not a chord anyone can trigger.
        assert!(parse_shortcut("ctrl+shift").is_err());
    }

    #[test]
    fn a_lone_modifier_is_a_valid_chord() {
        // Tapping Super alone opens the GNOME overview.
        let (mods, k) = parse_shortcut("super").unwrap();
        assert!(mods.is_empty());
        assert_eq!(k, Key::KEY_LEFTMETA);
    }

    #[test]
    fn gtk_above_tab_is_grave() {
        assert_eq!(key("abovetab"), Some(Key::KEY_GRAVE));
    }

    #[test]
    fn case_and_whitespace_are_forgiving() {
        let (mods, k) = parse_shortcut(" Ctrl + Shift + P ").unwrap();
        assert_eq!(mods.len(), 2);
        assert_eq!(k, Key::KEY_P);
    }
}
