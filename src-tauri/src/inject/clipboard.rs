use std::ptr;

// ── Raw Win32 FFI (bypasses windows-crate version conflicts) ──

#[allow(non_snake_case)]
pub(crate) mod ffi {
    #[link(name = "user32")]
    extern "system" {
        pub fn OpenClipboard(hWnd: isize) -> i32;
        pub fn EmptyClipboard() -> i32;
        pub fn SetClipboardData(uFormat: u32, hMem: isize) -> isize;
        pub fn GetClipboardData(uFormat: u32) -> isize;
        pub fn CloseClipboard() -> i32;
        pub fn SendInput(cInputs: u32, pInputs: *const INPUT, cbSize: i32) -> u32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> isize;
        pub fn GlobalLock(hMem: isize) -> *mut u8;
        pub fn GlobalUnlock(hMem: isize) -> i32;
        pub fn GlobalFree(hMem: isize) -> isize;
    }

    pub const CF_UNICODETEXT: u32 = 13;
    pub const GMEM_MOVEABLE: u32 = 0x0002;

    // SendInput types
    pub const INPUT_KEYBOARD: u32 = 1;
    pub const KEYEVENTF_KEYUP: u32 = 0x0002;
    pub const KEYEVENTF_UNICODE: u32 = 0x0004;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct KEYBDINPUT {
        pub wVk: u16,
        pub wScan: u16,
        pub dwFlags: u32,
        pub time: u32,
        pub dwExtraInfo: usize,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct INPUT {
        pub r#type: u32,
        pub ki: KEYBDINPUT,
    }
}

use ffi::*;

pub struct ClipboardBackup {
    handle: isize,
}

impl ClipboardBackup {
    pub fn backup() -> Self {
        // SAFETY: Win32 clipboard API calls. hWnd=0 for current process.
        // OpenClipboard must succeed before GetClipboardData/GlobalLock.
        // All global handles are properly unlocked/freed. CloseClipboard is called on all paths.
        let handle = unsafe {
            if OpenClipboard(0) == 0 {
                tracing::warn!("[clipboard] OpenClipboard failed during backup — original content will not be restored");
                return Self { handle: 0 };
            }
            let data = GetClipboardData(CF_UNICODETEXT);
            let result = if data == 0 {
                0
            } else {
                let src = GlobalLock(data);
                if src.is_null() {
                    0
                } else {
                    let len = {
                        let mut n = 0usize;
                        let p = src as *const u16;
                        while *p.add(n) != 0 {
                            n += 1;
                        }
                        n + 1
                    };
                    let backup = GlobalAlloc(GMEM_MOVEABLE, len * 2);
                    if backup != 0 {
                        let dst = GlobalLock(backup) as *mut u16;
                        ptr::copy_nonoverlapping(src as *const u16, dst, len);
                        GlobalUnlock(backup);
                    }
                    GlobalUnlock(data);
                    backup
                }
            };
            CloseClipboard();
            result
        };
        Self { handle }
    }

    pub fn set_text(text: &str) {
        let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let size = utf16.len() * 2;
        // SAFETY: Win32 clipboard write sequence. utf16 is a valid null-terminated UTF-16 string.
        // OpenClipboard/EmptyClipboard/GlobalAlloc/GlobalLock/SetClipboardData/CloseClipboard
        // all operate on well-defined memory. GlobalUnlock pairs each GlobalLock.
        unsafe {
            if OpenClipboard(0) == 0 {
                tracing::error!("[clipboard] OpenClipboard failed in set_text");
                return;
            }
            if EmptyClipboard() == 0 {
                tracing::error!("[clipboard] EmptyClipboard failed");
                CloseClipboard();
                return;
            }
            let h = GlobalAlloc(GMEM_MOVEABLE, size);
            if h == 0 {
                tracing::error!("[clipboard] GlobalAlloc failed (size={size})");
                CloseClipboard();
                return;
            }
            let dst = GlobalLock(h) as *mut u16;
            ptr::copy_nonoverlapping(utf16.as_ptr(), dst, utf16.len());
            GlobalUnlock(h);
            let result = SetClipboardData(CF_UNICODETEXT, h);
            if result == 0 {
                tracing::error!("[clipboard] SetClipboardData failed (CF_UNICODETEXT)");
            }
            CloseClipboard();
        }
    }
}

impl Drop for ClipboardBackup {
    fn drop(&mut self) {
        if self.handle != 0 {
            // SAFETY: self.handle is a valid global memory handle from a prior successful
            // GetClipboardData+GlobalAlloc+GlobalLock+copy+GlobalUnlock sequence in backup().
            // The handle contains a null-terminated UTF-16 string allocated with GMEM_MOVEABLE.
            // All GlobalLock calls are paired with GlobalUnlock.
            unsafe {
                if OpenClipboard(0) != 0 {
                    EmptyClipboard();
                    let h = GlobalAlloc(GMEM_MOVEABLE, {
                        let src = GlobalLock(self.handle) as *const u16;
                        let mut n = 0usize;
                        while *src.add(n) != 0 { n += 1; }
                        GlobalUnlock(self.handle);
                        (n + 1) * 2
                    });
                    if h != 0 {
                        let src = GlobalLock(self.handle) as *const u16;
                        let dst = GlobalLock(h) as *mut u16;
                        let mut n = 0usize;
                        while *src.add(n) != 0 { n += 1; }
                        ptr::copy_nonoverlapping(src, dst, n + 1);
                        GlobalUnlock(h);
                        GlobalUnlock(self.handle);
                        SetClipboardData(CF_UNICODETEXT, h);
                    }
                    CloseClipboard();
                }
            }
        }
    }
}
