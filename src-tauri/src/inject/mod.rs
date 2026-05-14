pub mod clipboard;
pub mod send_input;

use crate::error::AppResult;

#[derive(Debug)]
pub enum InjectResult {
    Injected,
    FallbackToUnicode,
    ClipboardOnly,
}

/// Main injection flow: try Ctrl+V first, fall back to Unicode SendInput
pub fn inject(text: &str) -> AppResult<InjectResult> {
    let _backup = clipboard::ClipboardBackup::backup();
    tracing::info!("Writing to clipboard: [{}]", text);
    clipboard::ClipboardBackup::set_text(text);

    if send_input::send_ctrl_v() {
        tracing::debug!("[inject] Ctrl+V succeeded");
        return Ok(InjectResult::Injected);
    }

    tracing::warn!("[inject] Ctrl+V failed, trying send_unicode");
    if send_input::send_unicode(text) {
        tracing::debug!("[inject] send_unicode succeeded");
        return Ok(InjectResult::FallbackToUnicode);
    }

    tracing::error!("[inject] Both Ctrl+V and send_unicode failed");
    Ok(InjectResult::ClipboardOnly)
}
