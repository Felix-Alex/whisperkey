use super::clipboard::ffi::*;

const VK_CONTROL: u16 = 0x11;
const VK_V: u16 = 0x56;

fn keybd(vk: u16, flags: u32) -> bool {
    // INPUT must match the Windows layout exactly:
    //   typedef struct { DWORD type; union { MOUSEINPUT mi; KEYBDINPUT ki; HARDWAREINPUT hi; }; } INPUT;
    // sizeof(INPUT) = 40 on x64, 28 on x86
    // We pad to the x64 size (40 bytes) to keep the struct correct.
    #[repr(C, align(8))]
    struct PaddedInput {
        r#type: u32,
        _pad: u32,
        ki: KEYBDINPUT,
        _tail: u64,
    }
    let input = PaddedInput {
        r#type: INPUT_KEYBOARD,
        _pad: 0,
        ki: KEYBDINPUT {
            wVk: vk,
            wScan: 0,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
        },
        _tail: 0,
    };
    // SAFETY: PaddedInput is repr(C, align(8)) matching the Windows INPUT struct layout.
    // The pointer is valid and points to a correctly initialized KEYBDINPUT event.
    // cbSize matches the actual struct size per Win32 API requirements.
    let sent = unsafe { SendInput(1, &input as *const PaddedInput as *const INPUT, std::mem::size_of::<PaddedInput>() as i32) };
    if sent != 1 {
        tracing::error!("[send_input] SendInput returned {sent} (vk={vk}, flags={flags})");
    }
    sent == 1
}

/// Send Ctrl+V via Windows SendInput
pub fn send_ctrl_v() -> bool {
    if !keybd(VK_CONTROL, 0) {
        tracing::error!("[send_input] Ctrl down failed");
        return false;
    }
    std::thread::sleep(std::time::Duration::from_millis(15));
    if !keybd(VK_V, 0) {
        tracing::error!("[send_input] V down failed");
        keybd(VK_CONTROL, KEYEVENTF_KEYUP);
        return false;
    }
    std::thread::sleep(std::time::Duration::from_millis(15));
    keybd(VK_V, KEYEVENTF_KEYUP);
    std::thread::sleep(std::time::Duration::from_millis(15));
    keybd(VK_CONTROL, KEYEVENTF_KEYUP);
    tracing::debug!("[send_input] Ctrl+V sent successfully");
    true
}

/// Send Unicode text character by character (fallback)
pub fn send_unicode(text: &str) -> bool {
    #[repr(C, align(8))]
    struct PaddedInput {
        r#type: u32,
        _pad: u32,
        ki: KEYBDINPUT,
        _tail: u64,
    }
    let mut inputs: Vec<PaddedInput> = Vec::with_capacity(text.chars().count() * 2);
    for ch in text.chars() {
        let code = ch as u16;
        inputs.push(PaddedInput {
            r#type: INPUT_KEYBOARD,
            _pad: 0,
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: code,
                dwFlags: KEYEVENTF_UNICODE,
                time: 0,
                dwExtraInfo: 0,
            },
            _tail: 0,
        });
        inputs.push(PaddedInput {
            r#type: INPUT_KEYBOARD,
            _pad: 0,
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: code,
                dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            },
            _tail: 0,
        });
    }
    if inputs.is_empty() {
        return true;
    }
    // SAFETY: inputs is a Vec<PaddedInput> with correct repr(C) layout matching INPUT.
    // Each element is a valid KEYEVENTF_UNICODE key event with proper wScan and flags.
    // as_ptr() points to a contiguous array of correctly sized structs.
    let sent = unsafe { SendInput(inputs.len() as u32, inputs.as_ptr() as *const INPUT, std::mem::size_of::<PaddedInput>() as i32) };
    if sent != inputs.len() as u32 {
        tracing::error!("[send_input] send_unicode SendInput returned {sent} (expected {})", inputs.len());
    }
    sent == inputs.len() as u32
}
