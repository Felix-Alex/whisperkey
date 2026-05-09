pub mod clipboard;
pub mod send_input;

use crate::error::AppResult;

pub enum InjectResult {
    Injected,
    FallbackToUnicode,
    ClipboardOnly,
}

/// Main injection flow: try Ctrl+V first, fall back to Unicode SendInput
pub fn inject(text: &str) -> AppResult<InjectResult> {
    let _backup = clipboard::ClipboardBackup::backup();
    clipboard::ClipboardBackup::set_text(text);

    if send_input::send_ctrl_v() {
        return Ok(InjectResult::Injected);
    }

    if send_input::send_unicode(text) {
        return Ok(InjectResult::FallbackToUnicode);
    }

    Ok(InjectResult::ClipboardOnly)
}
