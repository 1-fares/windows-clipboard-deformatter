use windows::core::PCWSTR;
use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_SET_VALUE, REG_SZ,
};

use crate::error::AppError;

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn win32_err(e: WIN32_ERROR) -> AppError {
    AppError::Windows(windows::core::Error::from(e))
}

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "ClipboardDeformatter";

pub fn register() -> Result<(), AppError> {
    let exe_path = std::env::current_exe()
        .map_err(|_| AppError::Windows(windows::core::Error::from_thread()))?;
    let exe_path_wide = to_wide(&exe_path.to_string_lossy());
    let key_path = to_wide(RUN_KEY);
    let value_name = to_wide(VALUE_NAME);

    unsafe {
        let mut hkey = HKEY::default();
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_path.as_ptr()),
            None,
            KEY_SET_VALUE,
            &mut hkey,
        );
        if result != ERROR_SUCCESS {
            return Err(win32_err(result));
        }

        let byte_len = exe_path_wide.len() * std::mem::size_of::<u16>();
        let bytes = std::slice::from_raw_parts(exe_path_wide.as_ptr() as *const u8, byte_len);

        let result = RegSetValueExW(
            hkey,
            PCWSTR(value_name.as_ptr()),
            None,
            REG_SZ,
            Some(bytes),
        );

        let _ = RegCloseKey(hkey);

        if result != ERROR_SUCCESS {
            return Err(win32_err(result));
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn unregister() -> Result<(), AppError> {
    let key_path = to_wide(RUN_KEY);
    let value_name = to_wide(VALUE_NAME);

    unsafe {
        let mut hkey = HKEY::default();
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_path.as_ptr()),
            None,
            KEY_SET_VALUE,
            &mut hkey,
        );
        if result != ERROR_SUCCESS {
            return Err(win32_err(result));
        }

        let result = RegDeleteValueW(hkey, PCWSTR(value_name.as_ptr()));

        let _ = RegCloseKey(hkey);

        if result != ERROR_SUCCESS {
            return Err(win32_err(result));
        }
    }

    Ok(())
}
