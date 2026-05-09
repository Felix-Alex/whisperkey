pub mod registrar;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HotkeyEvent {
    Triggered,
    RegisterFailed,
}

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    Ctrl,
    Shift,
    Alt,
    Win,
}

impl Modifier {
    pub fn to_winapi(self) -> u32 {
        match self {
            Modifier::Alt => 0x0001,   // MOD_ALT
            Modifier::Ctrl => 0x0002,  // MOD_CONTROL
            Modifier::Shift => 0x0004, // MOD_SHIFT
            Modifier::Win => 0x0008,   // MOD_WIN
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ctrl" | "control" => Some(Modifier::Ctrl),
            "shift" => Some(Modifier::Shift),
            "alt" => Some(Modifier::Alt),
            "win" | "windows" | "super" => Some(Modifier::Win),
            _ => None,
        }
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modifier::Ctrl => write!(f, "Ctrl"),
            Modifier::Shift => write!(f, "Shift"),
            Modifier::Alt => write!(f, "Alt"),
            Modifier::Win => write!(f, "Win"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyConfig {
    pub modifiers: Vec<Modifier>,
    pub key: String,
}

impl HotkeyConfig {
    pub fn new(modifiers: Vec<Modifier>, key: &str) -> Self {
        Self {
            modifiers,
            key: key.to_string(),
        }
    }

    /// Convert to Windows API (modifier flags, virtual key code)
    pub fn to_winapi(&self) -> (u32, u32) {
        let mod_flags: u32 = self.modifiers.iter().map(|m| m.to_winapi()).sum();
        let vk = key_to_vk(&self.key);
        (mod_flags, vk)
    }

    /// Parse from string like "Ctrl+Shift+Space" or "Alt+F2"
    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
        if parts.len() < 2 {
            return None;
        }

        let key_raw = parts.last().unwrap().to_string();
        let mut modifiers = Vec::new();

        for part in &parts[..parts.len() - 1] {
            match Modifier::from_string(part) {
                Some(m) => modifiers.push(m),
                None => return None,
            }
        }

        if key_to_vk(&key_raw) == 0 {
            return None;
        }

        let key = normalize_key(&key_raw);

        Some(Self { modifiers, key })
    }
}

impl fmt::Display for HotkeyConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for m in &self.modifiers {
            write!(f, "{m}+")?;
        }
        write!(f, "{}", self.key)
    }
}

fn normalize_key(key: &str) -> String {
    let upper = key.to_uppercase();
    match upper.as_str() {
        "SPACE" | "ENTER" | "RETURN" | "TAB" | "ESCAPE" | "ESC" | "BACKSPACE" | "BACK"
        | "DELETE" | "DEL" | "INSERT" | "INS" | "HOME" | "END" | "PAGEUP" | "PGUP"
        | "PAGEDOWN" | "PGDN" | "UP" | "DOWN" | "LEFT" | "RIGHT" | "PRINTSCREEN" | "PRTSC"
        | "PAUSE" | "CAPSLOCK" | "NUMLOCK" | "SCROLLLOCK" => {
            // Title-case: first char upper, rest lower
            let mut chars = key.chars();
            match chars.next() {
                None => key.to_string(),
                Some(first) => {
                    let rest: String = chars.collect();
                    format!("{}{}", first.to_uppercase(), rest.to_lowercase())
                }
            }
        }
        _ if upper.starts_with('F') => upper,
        _ if key.len() == 1 => key.to_uppercase(),
        _ => key.to_string(),
    }
}

/// Map a key name string to Windows virtual key code
fn key_to_vk(key: &str) -> u32 {
    match key.to_uppercase().as_str() {
        "SPACE" => 0x20,
        "ENTER" | "RETURN" => 0x0D,
        "TAB" => 0x09,
        "ESCAPE" | "ESC" => 0x1B,
        "BACKSPACE" | "BACK" => 0x08,
        "DELETE" | "DEL" => 0x2E,
        "INSERT" | "INS" => 0x2D,
        "HOME" => 0x24,
        "END" => 0x23,
        "PAGEUP" | "PGUP" => 0x21,
        "PAGEDOWN" | "PGDN" => 0x22,
        "UP" => 0x26,
        "DOWN" => 0x28,
        "LEFT" => 0x25,
        "RIGHT" => 0x27,
        "PRINTSCREEN" | "PRTSC" => 0x2C,
        "PAUSE" => 0x13,
        "CAPSLOCK" => 0x14,
        "NUMLOCK" => 0x90,
        "SCROLLLOCK" => 0x91,
        s if s.starts_with('F') => {
            if let Ok(n) = s[1..].parse::<u32>() {
                if (1..=24).contains(&n) {
                    return 0x6F + n; // VK_F1 = 0x70
                }
            }
            0
        }
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap();
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase() as u32
            } else {
                0
            }
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ctrl_shift_space() {
        let hk = HotkeyConfig::from_string("Ctrl+Shift+Space").unwrap();
        assert_eq!(hk.modifiers.len(), 2);
        assert!(hk.modifiers.contains(&Modifier::Ctrl));
        assert!(hk.modifiers.contains(&Modifier::Shift));
        assert_eq!(hk.key, "Space");
    }

    #[test]
    fn test_parse_alt_f2() {
        let hk = HotkeyConfig::from_string("Alt+F2").unwrap();
        assert_eq!(hk.modifiers, vec![Modifier::Alt]);
        assert_eq!(hk.key, "F2");
    }

    #[test]
    fn test_parse_win_a() {
        let hk = HotkeyConfig::from_string("Win+A").unwrap();
        assert_eq!(hk.modifiers, vec![Modifier::Win]);
        assert_eq!(hk.key, "A");
    }

    #[test]
    fn test_display_roundtrip() {
        let hk = HotkeyConfig::from_string("Ctrl+Shift+Space").unwrap();
        assert_eq!(hk.to_string(), "Ctrl+Shift+Space");
    }

    #[test]
    fn test_to_winapi() {
        let hk = HotkeyConfig::from_string("Ctrl+Shift+Space").unwrap();
        let (mods, vk) = hk.to_winapi();
        assert_eq!(mods, Modifier::Ctrl.to_winapi() + Modifier::Shift.to_winapi());
        assert_eq!(vk, 0x20); // VK_SPACE
    }

    #[test]
    fn test_to_winapi_f2() {
        let hk = HotkeyConfig::from_string("Alt+F2").unwrap();
        let (mods, vk) = hk.to_winapi();
        assert_eq!(mods, Modifier::Alt.to_winapi());
        assert_eq!(vk, 0x71); // VK_F2
    }

    #[test]
    fn test_invalid_string() {
        assert!(HotkeyConfig::from_string("").is_none());
        assert!(HotkeyConfig::from_string("Space").is_none()); // No modifier
        assert!(HotkeyConfig::from_string("Foo+Bar").is_none()); // Invalid
    }

    #[test]
    fn test_parse_lowercase_modifiers() {
        let hk = HotkeyConfig::from_string("ctrl+shift+a").unwrap();
        assert_eq!(hk.modifiers.len(), 2);
        assert_eq!(hk.key, "A");
    }

    #[test]
    fn test_special_keys() {
        assert!(HotkeyConfig::from_string("Ctrl+Enter").is_some());
        assert!(HotkeyConfig::from_string("Ctrl+Tab").is_some());
        assert!(HotkeyConfig::from_string("Ctrl+Escape").is_some());
        assert!(HotkeyConfig::from_string("Ctrl+Backspace").is_some());
    }

    #[test]
    fn test_key_to_vk() {
        assert_eq!(key_to_vk("Space"), 0x20);
        assert_eq!(key_to_vk("F1"), 0x70);
        assert_eq!(key_to_vk("F12"), 0x7B);
        assert_eq!(key_to_vk("F24"), 0x87);
        assert_eq!(key_to_vk("A"), 0x41);
        assert_eq!(key_to_vk("Z"), 0x5A);
        assert_eq!(key_to_vk("0"), 0x30);
        assert_eq!(key_to_vk("Enter"), 0x0D);
        assert_eq!(key_to_vk("Escape"), 0x1B);
    }
}
