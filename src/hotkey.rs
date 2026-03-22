use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_WIN, VK_OEM_3,
};

use crate::error::AppError;

pub const HOTKEY_ID: i32 = 1;

pub fn register(hwnd: HWND) -> Result<(), AppError> {
    unsafe {
        RegisterHotKey(
            Some(hwnd),
            HOTKEY_ID,
            HOT_KEY_MODIFIERS(MOD_WIN.0),
            VK_OEM_3.0 as u32,
        )?;
    }
    Ok(())
}

pub fn unregister(hwnd: HWND) -> Result<(), AppError> {
    unsafe {
        UnregisterHotKey(Some(hwnd), HOTKEY_ID)?;
    }
    Ok(())
}
