use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT, MOD_WIN,
};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, PostThreadMessageW, MSG, WM_HOTKEY, WM_QUIT};
use windows::Win32::System::Threading::GetCurrentThreadId;

use crate::hotkey::{HotkeyConfig, HotkeyEvent, Modifier};

const HOTKEY_ID: i32 = 0xC001;

pub struct HotkeyHandle {
    thread_id: u32,
    stop_flag: Arc<AtomicBool>,
}

impl Drop for HotkeyHandle {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        let _ = unsafe { PostThreadMessageW(self.thread_id, WM_QUIT, None, None) };
    }
}

fn modifiers_to_hotkey(mods: &[Modifier]) -> HOT_KEY_MODIFIERS {
    let mut flags = HOT_KEY_MODIFIERS(0);
    for m in mods {
        flags |= match m {
                Modifier::Ctrl => MOD_CONTROL,
                Modifier::Shift => MOD_SHIFT,
                Modifier::Alt => MOD_ALT,
                Modifier::Win => MOD_WIN,
            };
    }
    flags | MOD_NOREPEAT
}

pub fn start(cfg: HotkeyConfig, tx: broadcast::Sender<HotkeyEvent>) -> HotkeyHandle {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let flag = stop_flag.clone();
    let shared_tid = Arc::new(Mutex::new(0u32));
    let tid_ref = shared_tid.clone();

    let _handle = std::thread::Builder::new()
        .name("hotkey-pump".into())
        .spawn(move || {
            let tid = unsafe { GetCurrentThreadId() };
            {
                let mut shared = tid_ref.lock().unwrap();
                *shared = tid;
            }

            let (_mods, vk) = cfg.to_winapi();
            let hotkey_mods = modifiers_to_hotkey(&cfg.modifiers);

            unsafe {
                if let Err(_e) = RegisterHotKey(None, HOTKEY_ID, hotkey_mods, vk) {
                    let _ = tx.send(HotkeyEvent::RegisterFailed);
                    return;
                }
            }

            let mut msg = MSG::default();
            loop {
                if flag.load(Ordering::SeqCst) {
                    break;
                }

                let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
                if ret.0 == 0 || ret.0 == -1 {
                    break;
                }

                if msg.message == WM_HOTKEY {
                    let _ = tx.send(HotkeyEvent::Triggered);
                }
            }

            unsafe {
                let _ = UnregisterHotKey(None, HOTKEY_ID);
            }
        })
        .expect("failed to spawn hotkey thread");

    // Spin-wait briefly for the thread to set its ID
    let tid = loop {
        let t = *shared_tid.lock().unwrap();
        if t != 0 {
            break t;
        }
        std::thread::yield_now();
    };

    HotkeyHandle {
        thread_id: tid,
        stop_flag,
    }
}
