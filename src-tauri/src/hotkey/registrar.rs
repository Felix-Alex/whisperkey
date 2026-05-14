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

impl HotkeyHandle {
    pub fn thread_id(&self) -> u32 {
        self.thread_id
    }
}

impl Drop for HotkeyHandle {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        // SAFETY: self.thread_id was captured from GetCurrentThreadId in the hotkey pump thread.
        // The thread is guaranteed to have a message queue (it calls GetMessageW in a loop).
        // WM_QUIT causes the thread to break its message loop and exit cleanly.
        // wParam and lParam are 0 (None for WM_QUIT).
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
            // SAFETY: GetCurrentThreadId always returns the calling thread's ID and has no preconditions.
            let tid = unsafe { GetCurrentThreadId() };
            {
                let mut shared = tid_ref.lock().unwrap();
                *shared = tid;
            }

            let (_mods, vk) = cfg.to_winapi();
            let hotkey_mods = modifiers_to_hotkey(&cfg.modifiers);

            println!("=== PROBE-REG-1: RegisterHotKey cfg={cfg}, vk=0x{vk:02X}, mods={hotkey_mods:?} ===");

            // SAFETY: None = current thread's message queue (the pump thread we just created).
            // HOTKEY_ID is a unique application-defined constant. hotkey_mods and vk come from
            // validated HotkeyConfig. Registration happens once before entering the message loop.
            unsafe {
                match RegisterHotKey(None, HOTKEY_ID, hotkey_mods, vk) {
                    Ok(()) => {
                        println!("=== PROBE-REG-2: RegisterHotKey SUCCESS (vk=0x{vk:02X}, HOTKEY_ID=0x{HOTKEY_ID:04X}) ===");
                        tracing::info!("RegisterHotKey succeeded: mods={hotkey_mods:?}, vk={vk}");
                    }
                    Err(e) => {
                        println!("=== PROBE-REG-2: RegisterHotKey FAILED: {e:?} ===");
                        tracing::error!("RegisterHotKey failed: {e:?}, mods={hotkey_mods:?}, vk={vk}");
                        let _ = tx.send(HotkeyEvent::RegisterFailed);
                        return;
                    }
                }
            }

            println!("=== PROBE-REG-3: Entering GetMessageW loop (tid={tid}) ===");
            let mut msg = MSG::default();
            loop {
                if flag.load(Ordering::SeqCst) {
                    println!("=== PROBE-REG-4: Stop flag set, breaking loop ===");
                    break;
                }

                // SAFETY: msg is a valid MSG struct on the stack. None = receive all messages
                // for this thread (including WM_HOTKEY). GetMessageW blocks until a message arrives.
                let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
                if ret.0 == 0 {
                    println!("=== PROBE-REG-5: GetMessageW returned 0 (WM_QUIT?), breaking loop ===");
                    break;
                }
                if ret.0 == -1 {
                    println!("=== PROBE-REG-6: GetMessageW returned -1 (error), breaking loop ===");
                    break;
                }

                if msg.message == WM_HOTKEY {
                    println!("=== PROBE-REG-7: WM_HOTKEY received! Sending Triggered event ===");
                    let _ = tx.send(HotkeyEvent::Triggered);
                }
            }

            println!("=== PROBE-REG-8: Exiting message loop, unregistering hotkey ===");
            // SAFETY: HOTKEY_ID is the same constant used in RegisterHotKey above.
            // None = current thread, which is the one that registered the hotkey.
            // UnregisterHotKey failure is non-critical (thread is exiting anyway).
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
