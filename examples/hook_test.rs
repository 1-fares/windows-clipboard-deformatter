//! Diagnostic test: verifies whether a low-level keyboard hook can suppress
//! a Win+C hotkey registered via RegisterHotKey on this system.
//!
//! Build:  cargo build --example hook_test --target x86_64-pc-windows-gnu
//! Run the resulting .exe on Windows from a normal (non-elevated) terminal.

use std::cell::Cell;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, RegisterHotKey, SendInput, UnregisterHotKey, HOT_KEY_MODIFIERS, INPUT,
    INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_C, VK_LWIN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, PeekMessageW, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, MSG, PM_REMOVE, WH_KEYBOARD_LL,
    WM_HOTKEY, WM_KEYDOWN, WM_SYSKEYDOWN,
};

const MOD_WIN: HOT_KEY_MODIFIERS = HOT_KEY_MODIFIERS(0x0008);
const HOTKEY_ID: i32 = 9999;

thread_local! {
    static HOOK: Cell<Option<HHOOK>> = const { Cell::new(None) };
    static HOOK_SAW_WINC: Cell<bool> = const { Cell::new(false) };
}

unsafe extern "system" fn test_hook(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code as u32 == HC_ACTION {
        let kb = &*(lp.0 as *const KBDLLHOOKSTRUCT);
        let msg = wp.0 as u32;
        if kb.vkCode == VK_C.0 as u32 && (msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN) {
            let win = (GetAsyncKeyState(VK_LWIN.0 as i32) as u16 & 0x8000) != 0;
            if win {
                HOOK_SAW_WINC.with(|c| c.set(true));
                return LRESULT(1); // suppress
            }
        }
    }
    let h = HOOK.with(|c| c.get());
    CallNextHookEx(h, code, wp, lp)
}

fn main() {
    println!("=== Clipboard Deformatter — LL Hook Diagnostic ===\n");

    unsafe {
        // 1. Try to register Win+C ourselves
        let reg_ok = RegisterHotKey(None, HOTKEY_ID, MOD_WIN, VK_C.0 as u32).is_ok();
        if reg_ok {
            println!("[OK]   RegisterHotKey(Win+C) succeeded — no other app holds it");
        } else {
            println!("[WARN] RegisterHotKey(Win+C) FAILED — another app (Copilot?) owns Win+C");
        }

        // 2. Install LL keyboard hook
        let hmod = GetModuleHandleW(PCWSTR::null()).unwrap();
        let hook =
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(test_hook), Some(hmod.into()), 0).unwrap();
        HOOK.with(|c| c.set(Some(hook)));
        println!("[OK]   LL keyboard hook installed");

        // 3. Simulate Win+C via SendInput
        println!("\n       Simulating Win+C via SendInput ...");
        let events = [
            key_input(VK_LWIN.0, false),
            key_input(VK_C.0, false),
            key_input(VK_C.0, true),
            key_input(VK_LWIN.0, true),
        ];
        SendInput(&events, std::mem::size_of::<INPUT>() as i32);

        // 4. Pump messages
        let mut hotkey_msg = false;
        let mut msg = MSG::default();
        // Pump a few times to make sure all messages arrive
        for _ in 0..50 {
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_HOTKEY {
                    hotkey_msg = true;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // 5. Results
        let hook_ok = HOOK_SAW_WINC.with(|c| c.get());
        println!("\n=== Results ===");
        if hook_ok {
            println!("[OK]   LL hook intercepted Win+C");
        } else {
            println!("[FAIL] LL hook did NOT see Win+C");
        }
        if reg_ok {
            if hotkey_msg {
                println!("[FAIL] RegisterHotKey WM_HOTKEY was received DESPITE hook suppression");
                println!("       -> LL hooks CANNOT block system hotkeys on this build of Windows");
                println!("       -> DisabledHotkeys registry fix is required");
            } else {
                println!("[OK]   RegisterHotKey WM_HOTKEY was NOT received (hook suppression works)");
            }
        } else {
            if hotkey_msg {
                println!("[INFO] Got a WM_HOTKEY even though we didn't register — unexpected");
            } else {
                println!("[INFO] No WM_HOTKEY (expected, since RegisterHotKey failed)");
            }
        }

        // Cleanup
        if reg_ok {
            let _ = UnregisterHotKey(None, HOTKEY_ID);
        }
        let _ = UnhookWindowsHookEx(hook);

        println!("\nDone. Press Enter to exit.");
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).ok();
    }
}

fn key_input(vk: u16, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(vk),
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
                ..Default::default()
            },
        },
    }
}
