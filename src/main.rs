#![windows_subsystem = "windows"]

mod app;
mod clipboard;
mod error;
mod hotkey;
mod overlay;
mod startup;
mod tray;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG,
};

use app::App;
use error::AppError;

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn run() -> Result<(), AppError> {
    // Single instance check
    let mutex_name = to_wide("Global\\ClipboardDeformatterMutex");
    unsafe {
        let _mutex = CreateMutexW(None, true, PCWSTR(mutex_name.as_ptr()))?;
        if GetLastError() == ERROR_ALREADY_EXISTS {
            return Err(AppError::AlreadyRunning);
        }
    }

    let hinstance = unsafe { GetModuleHandleW(PCWSTR::null())? };

    let mut app = App::new(hinstance.into());
    app.init()?;

    // Message loop
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}

fn show_error(msg: &str) {
    let wide = to_wide(msg);
    let title = to_wide("Clipboard Deformatter");
    unsafe {
        windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
            None,
            PCWSTR(wide.as_ptr()),
            PCWSTR(title.as_ptr()),
            windows::Win32::UI::WindowsAndMessaging::MB_OK
                | windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR,
        );
    }
}

fn main() {
    if let Err(e) = run() {
        match e {
            AppError::AlreadyRunning => {}
            _ => show_error(&format!("{}", e)),
        }
    }
}
