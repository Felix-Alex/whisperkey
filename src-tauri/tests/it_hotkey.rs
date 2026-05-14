/// IT-01: Hotkey parsing, validation, and display (logic-layer, no Windows API)
use whisperkey_lib::hotkey::{HotkeyConfig, Modifier};

#[test]
fn it01_parse_ctrl_shift_space() {
    let hk = HotkeyConfig::from_string("Ctrl+Shift+Space").unwrap();
    assert_eq!(hk.modifiers.len(), 2);
    assert!(hk.modifiers.contains(&Modifier::Ctrl));
    assert!(hk.modifiers.contains(&Modifier::Shift));
    assert_eq!(hk.key, "Space");
}

#[test]
fn it01_parse_alt_j() {
    let hk = HotkeyConfig::from_string("Alt+J").unwrap();
    assert_eq!(hk.modifiers, vec![Modifier::Alt]);
    assert_eq!(hk.key, "J");
}

#[test]
fn it01_parse_win_a() {
    let hk = HotkeyConfig::from_string("Win+A").unwrap();
    assert_eq!(hk.modifiers, vec![Modifier::Win]);
    assert_eq!(hk.key, "A");
}

#[test]
fn it01_parse_ctrl_shift_alt_fails_too_many() {
    // 4-key combo (3 modifiers + 1 key) fails validate but parses OK
    let hk = HotkeyConfig::from_string("Ctrl+Shift+Alt+Space").unwrap();
    assert_eq!(hk.modifiers.len(), 3);
    assert!(hk.validate().is_err());
}

#[test]
fn it01_parse_single_key_fails() {
    assert!(HotkeyConfig::from_string("Space").is_none());
    assert!(HotkeyConfig::from_string("J").is_none());
}

#[test]
fn it01_parse_empty_fails() {
    assert!(HotkeyConfig::from_string("").is_none());
}

#[test]
fn it01_parse_invalid_modifier_fails() {
    assert!(HotkeyConfig::from_string("Foo+Bar").is_none());
    assert!(HotkeyConfig::from_string("A+B").is_none());
}

#[test]
fn it01_validate_two_key_ok() {
    let hk = HotkeyConfig::new(vec![Modifier::Ctrl], "Space");
    assert!(hk.validate().is_ok());
}

#[test]
fn it01_validate_three_key_ok() {
    let hk = HotkeyConfig::new(vec![Modifier::Ctrl, Modifier::Shift], "A");
    assert!(hk.validate().is_ok());
}

#[test]
fn it01_validate_no_modifier_fails() {
    let hk = HotkeyConfig::new(vec![], "J");
    assert!(hk.validate().is_err());
}

#[test]
fn it01_validate_four_key_fails() {
    let hk = HotkeyConfig::new(vec![Modifier::Ctrl, Modifier::Shift, Modifier::Alt], "A");
    assert!(hk.validate().is_err());
}

#[test]
fn it01_display_roundtrip() {
    let hk = HotkeyConfig::from_string("Ctrl+Shift+Space").unwrap();
    assert_eq!(hk.to_string(), "Ctrl+Shift+Space");
}

#[test]
fn it01_to_winapi() {
    let hk = HotkeyConfig::from_string("Alt+F2").unwrap();
    let (mods, vk) = hk.to_winapi();
    assert_eq!(mods, Modifier::Alt.to_winapi());
    assert_eq!(vk, 0x71); // VK_F2
}

#[test]
fn it01_special_keys_parse() {
    assert!(HotkeyConfig::from_string("Ctrl+Enter").is_some());
    assert!(HotkeyConfig::from_string("Ctrl+Tab").is_some());
    assert!(HotkeyConfig::from_string("Ctrl+Escape").is_some());
    assert!(HotkeyConfig::from_string("Ctrl+Backspace").is_some());
    assert!(HotkeyConfig::from_string("Alt+F12").is_some());
}

#[test]
fn it01_vk_codes_correct() {
    use whisperkey_lib::hotkey::key_to_vk;

    assert_eq!(key_to_vk("Space"), 0x20);
    assert_eq!(key_to_vk("F1"), 0x70);
    assert_eq!(key_to_vk("F12"), 0x7B);
    assert_eq!(key_to_vk("A"), 0x41);
    assert_eq!(key_to_vk("Z"), 0x5A);
    assert_eq!(key_to_vk("0"), 0x30);
    assert_eq!(key_to_vk("Enter"), 0x0D);
    assert_eq!(key_to_vk("Escape"), 0x1B);
    assert_eq!(key_to_vk("Tab"), 0x09);
}
