use std::cell::RefCell;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, PostQuitMessage, RegisterClassExW,
    WINDOW_EX_STYLE, WM_DESTROY, WM_HOTKEY, WM_RBUTTONUP, WNDCLASSEXW, WS_OVERLAPPED,
};

use crate::accent;
use crate::clipboard;
use crate::error::AppError;
use crate::hotkey;
use crate::overlay;
use crate::startup;
use crate::tray::{self, TrayIcon};

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

const MAIN_CLASS_NAME: &str = "ClipDeformatMain";

thread_local! {
    static APP: RefCell<Option<*mut App>> = const { RefCell::new(None) };
}

/// Get a mutable reference to the App from the thread-local pointer.
pub fn app_ptr() -> Option<&'static mut App> {
    APP.with(|cell| {
        let borrow = cell.borrow();
        borrow.map(|ptr| unsafe { &mut *ptr })
    })
}

pub struct App {
    pub hwnd: HWND,
    pub overlay_hwnd: HWND,
    pub hinstance: HINSTANCE,
    pub tray: Option<TrayIcon>,
    pub enabled: bool,
    pub accent_color: COLORREF,
    pub fade_step: u8,
}

impl App {
    pub fn new(hinstance: HINSTANCE) -> Self {
        App {
            hwnd: HWND::default(),
            overlay_hwnd: HWND::default(),
            hinstance,
            tray: None,
            enabled: true,
            accent_color: COLORREF(0x00D77800),
            fade_step: 0,
        }
    }

    pub fn init(&mut self) -> Result<(), AppError> {
        // Store self pointer in thread-local for WndProc access
        let self_ptr = self as *mut App;
        APP.with(|cell| {
            *cell.borrow_mut() = Some(self_ptr);
        });

        // Register main window class
        let class_name = to_wide(MAIN_CLASS_NAME);
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(main_wndproc),
            hInstance: self.hinstance,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        let atom = unsafe { RegisterClassExW(&wc) };
        if atom == 0 {
            return Err(AppError::Windows(windows::core::Error::from_thread()));
        }

        // Create hidden message-only window
        let class_name = to_wide(MAIN_CLASS_NAME);
        self.hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR::null(),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(self.hinstance),
                None,
            )?
        };

        // Register overlay window class and create overlay
        overlay::register_class(self.hinstance)?;
        self.overlay_hwnd = overlay::create_window(self.hinstance)?;

        // Register global hotkey
        hotkey::register(self.hwnd)?;

        // Get accent color
        self.accent_color = accent::get_accent_color();

        // Create system tray icon
        self.tray = Some(TrayIcon::create(self.hwnd)?);

        // Register auto-start (silently ignore errors)
        let _ = startup::register();

        Ok(())
    }

    pub fn shutdown(&mut self) {
        let _ = hotkey::unregister(self.hwnd);
        if let Some(ref tray) = self.tray {
            tray.remove();
        }
        overlay::hide(self.overlay_hwnd);
    }

    pub fn on_hotkey(&mut self) {
        if !self.enabled {
            return;
        }

        match clipboard::strip_formatting(self.hwnd) {
            Ok(true) => {
                self.accent_color = accent::get_accent_color();
                self.fade_step = 0;
                overlay::show(self.overlay_hwnd);
            }
            Ok(false) | Err(_) => {}
        }
    }

    pub fn on_timer(&mut self) {
        self.fade_step += 1;
        let done = overlay::tick_fade(self.overlay_hwnd, self.fade_step);
        if done {
            self.fade_step = 0;
        }
    }

    pub fn on_tray_callback(&mut self, lparam: LPARAM) {
        let msg = (lparam.0 & 0xFFFF) as u32;
        if msg == WM_RBUTTONUP {
            if let Some(ref tray) = self.tray {
                if let Some(cmd) = tray.show_context_menu(self.hwnd, self.enabled) {
                    match cmd {
                        tray::ID_TOGGLE => {
                            self.enabled = !self.enabled;
                        }
                        tray::ID_EXIT => unsafe {
                            let _ = DestroyWindow(self.hwnd);
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}

unsafe extern "system" fn main_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            if let Some(app) = app_ptr() {
                app.on_hotkey();
            }
            LRESULT(0)
        }
        msg if msg == tray::WM_TRAY_CALLBACK => {
            if let Some(app) = app_ptr() {
                app.on_tray_callback(lparam);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            if let Some(app) = app_ptr() {
                app.shutdown();
            }
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
