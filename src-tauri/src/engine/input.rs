//! Passive low-level keyboard hook: only observes key presses for reactive effects.
//! Every event is passed on unchanged with CallNextHookEx; injected events are ignored.

use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::OnceLock;
use std::thread;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, MSG, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

type Callback = Box<dyn Fn(u16) + Send + Sync>;
static CALLBACK: OnceLock<Callback> = OnceLock::new();
static DOWN: Mutex<Option<HashSet<u16>>> = Mutex::new(None);

unsafe extern "system" fn hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        if kb.flags.0 & LLKHF_INJECTED.0 == 0 {
            let sc = (kb.scanCode as u16 & 0xFF) | if kb.flags.0 & LLKHF_EXTENDED.0 != 0 { 0x100 } else { 0 };
            let msg = wparam.0 as u32;
            let mut down = DOWN.lock();
            let set = down.get_or_insert_with(HashSet::new);
            if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                if set.insert(sc) {
                    if let Some(cb) = CALLBACK.get() {
                        cb(sc); // first press only, auto-repeat ignored
                    }
                }
            } else if msg == WM_KEYUP || msg == WM_SYSKEYUP {
                set.remove(&sc);
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

pub fn start(cb: impl Fn(u16) + Send + Sync + 'static) {
    if CALLBACK.set(Box::new(cb)).is_err() {
        return;
    }
    thread::spawn(|| unsafe {
        let module = GetModuleHandleW(None).ok();
        let Ok(_hook) = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook), module.map(|m| m.into()), 0) else {
            log::warn!("keyboard hook could not be installed");
            return;
        };
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {}
    });
}
