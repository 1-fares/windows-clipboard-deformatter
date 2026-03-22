use std::ptr;

use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

use crate::error::AppError;

/// CF_UNICODETEXT = 13
const CF_UNICODETEXT: u32 = 13;

struct ClipboardGuard;

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

/// Read CF_UNICODETEXT from the clipboard, clear it, and write back only plain text.
/// Returns Ok(true) if text was present and stripped, Ok(false) if clipboard had no text.
pub fn strip_formatting(hwnd: HWND) -> Result<bool, AppError> {
    unsafe {
        // Try to open clipboard, retry once if locked
        if OpenClipboard(Some(hwnd)).is_err() {
            std::thread::sleep(std::time::Duration::from_millis(10));
            OpenClipboard(Some(hwnd))?;
        }
        let _guard = ClipboardGuard;

        // Get the plain text data
        let handle = GetClipboardData(CF_UNICODETEXT);
        let handle = match handle {
            Ok(h) if !h.is_invalid() => h,
            _ => return Ok(false),
        };

        // Lock and copy the text. HANDLE and HGLOBAL are both (*mut c_void) wrappers.
        let hglobal: HGLOBAL = std::mem::transmute(handle);
        let data_ptr = GlobalLock(hglobal);
        if data_ptr.is_null() {
            return Ok(false);
        }

        // Find the length of the null-terminated UTF-16 string
        let wide_ptr = data_ptr as *const u16;
        let mut len = 0usize;
        while *wide_ptr.add(len) != 0 {
            len += 1;
        }

        // Copy including null terminator
        let total_chars = len + 1;
        let mut text = vec![0u16; total_chars];
        ptr::copy_nonoverlapping(wide_ptr, text.as_mut_ptr(), total_chars);

        let _ = GlobalUnlock(hglobal);

        // Clear all clipboard formats
        EmptyClipboard()?;

        // Allocate new global memory and write plain text back
        let byte_size = total_chars * std::mem::size_of::<u16>();
        let new_hglobal = GlobalAlloc(GMEM_MOVEABLE, byte_size)?;
        let new_ptr = GlobalLock(new_hglobal);
        if new_ptr.is_null() {
            return Ok(false);
        }

        ptr::copy_nonoverlapping(text.as_ptr(), new_ptr as *mut u16, total_chars);
        let _ = GlobalUnlock(new_hglobal);

        // SetClipboardData takes ownership of the memory
        let new_handle: HANDLE = std::mem::transmute(new_hglobal);
        SetClipboardData(CF_UNICODETEXT, Some(new_handle))?;

        Ok(true)
    }
}
