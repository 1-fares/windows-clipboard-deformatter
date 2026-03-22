use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, VK_C,
};

use crate::error::AppError;

const HOTKEY_ID: i32 = 1;
/// MOD_WIN | MOD_SHIFT | MOD_NOREPEAT
const MODIFIERS: HOT_KEY_MODIFIERS = HOT_KEY_MODIFIERS(0x0008 | 0x0004 | 0x4000);

pub fn register(hwnd: HWND) -> Result<(), AppError> {
    unsafe {
        RegisterHotKey(Some(hwnd), HOTKEY_ID, MODIFIERS, VK_C.0 as u32)?;
    }
    Ok(())
}

pub fn unregister(hwnd: HWND) -> Result<(), AppError> {
    unsafe {
        UnregisterHotKey(Some(hwnd), HOTKEY_ID)?;
    }
    Ok(())
}
