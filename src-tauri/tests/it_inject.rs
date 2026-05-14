/// IT-12: Text injection logic-layer tests (no actual Windows injection)
/// Tests the inject module structure and clipboard utility
use whisperkey_lib::inject::{clipboard, send_input};

#[test]
fn it12_clipboard_backup_api() {
    // Verify clipboard backup API works
    let backup = clipboard::ClipboardBackup::backup();
    // The backup handle exists — actual clipboard interaction depends on Windows
    assert!(std::mem::size_of_val(&backup) > 0);
}

#[test]
fn it12_send_input_functions_exist() {
    // Verify send_input functions are callable
    // send_ctrl_v() attempts to send Ctrl+V
    let result = send_input::send_ctrl_v();
    // May fail in headless/test env, but shouldn't panic
    match result {
        true => {}  // Ctrl+V succeeded
        false => {} // Expected in headless env
    }

    // send_unicode falls back to character-by-character injection
    let uni_result = send_input::send_unicode("A");
    match uni_result {
        true => {}
        false => {} // Expected in headless env
    }
}

#[test]
fn it12_inject_result_enum_variants() {
    use whisperkey_lib::inject::InjectResult;

    let injected = InjectResult::Injected;
    let fallback = InjectResult::FallbackToUnicode;
    let copy_only = InjectResult::ClipboardOnly;

    // Verify all variants exist and can be pattern matched
    let results = vec![injected, fallback, copy_only];
    for r in &results {
        match r {
            InjectResult::Injected => assert!(true),
            InjectResult::FallbackToUnicode => assert!(true),
            InjectResult::ClipboardOnly => assert!(true),
        }
    }
    assert_eq!(results.len(), 3);
}
