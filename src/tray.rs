use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, LoadImageW, SetForegroundWindow,
    TrackPopupMenu, HICON, IMAGE_ICON, LR_DEFAULTSIZE, LR_SHARED, MF_CHECKED,
    MF_SEPARATOR, MF_STRING, MF_UNCHECKED, TPM_BOTTOMALIGN, TPM_RETURNCMD, TPM_RIGHTALIGN,
    WM_APP,
};

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub const WM_TRAY_CALLBACK: u32 = WM_APP + 1;
pub const ID_TOGGLE: u16 = 1001;
pub const ID_EXIT: u16 = 1002;

pub struct TrayIcon {
    nid: NOTIFYICONDATAW,
}

impl TrayIcon {
    pub fn create(hwnd: HWND) -> Result<Self, crate::error::AppError> {
        let mut nid = NOTIFYICONDATAW::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = WM_TRAY_CALLBACK;

        // Use IDI_APPLICATION (32512) system icon as placeholder
        let icon = unsafe {
            LoadImageW(
                None,
                PCWSTR(32512 as *const u16), // IDI_APPLICATION
                IMAGE_ICON,
                0,
                0,
                LR_SHARED | LR_DEFAULTSIZE,
            )
        };
        if let Ok(handle) = icon {
            nid.hIcon = HICON(handle.0);
        }

        let tip = to_wide("Clipboard Deformatter");
        let tip_len = tip.len().min(128);
        nid.szTip[..tip_len].copy_from_slice(&tip[..tip_len]);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_ADD, &nid);
        }

        Ok(TrayIcon { nid })
    }

    pub fn remove(&self) {
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &self.nid);
        }
    }

    pub fn show_context_menu(&self, hwnd: HWND, enabled: bool) -> Option<u16> {
        unsafe {
            let hmenu = CreatePopupMenu().ok()?;

            let toggle_label = if enabled {
                to_wide("Enabled")
            } else {
                to_wide("Disabled")
            };
            let toggle_flags = MF_STRING | if enabled { MF_CHECKED } else { MF_UNCHECKED };
            let _ = AppendMenuW(
                hmenu,
                toggle_flags,
                ID_TOGGLE as usize,
                PCWSTR(toggle_label.as_ptr()),
            );

            let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());

            let exit_label = to_wide("Exit");
            let _ = AppendMenuW(
                hmenu,
                MF_STRING,
                ID_EXIT as usize,
                PCWSTR(exit_label.as_ptr()),
            );

            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);

            // Required per MSDN before TrackPopupMenu
            let _ = SetForegroundWindow(hwnd);

            let cmd = TrackPopupMenu(
                hmenu,
                TPM_RIGHTALIGN | TPM_BOTTOMALIGN | TPM_RETURNCMD,
                pt.x,
                pt.y,
                None,
                hwnd,
                None,
            );

            let _ = DestroyMenu(hmenu);

            if cmd.as_bool() {
                Some(cmd.0 as u16)
            } else {
                None
            }
        }
    }
}
