use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontIndirectW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint,
    FillRect, GetStockObject, RoundRect, SelectObject, SetBkMode, SetTextColor, DT_CENTER,
    DT_SINGLELINE, DT_VCENTER, FW_SEMIBOLD, LOGFONTW, NULL_PEN, PAINTSTRUCT, TRANSPARENT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetClientRect, GetSystemMetrics, KillTimer,
    RegisterClassExW, SetLayeredWindowAttributes, SetTimer, ShowWindow, SetWindowPos,
    SM_CXSCREEN, SM_CYSCREEN, SW_HIDE, SW_SHOWNOACTIVATE,
    SWP_NOACTIVATE, SWP_NOZORDER, WNDCLASSEXW,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
    WS_POPUP, WM_ERASEBKGND, WM_PAINT, WM_TIMER,
};

use crate::app::app_ptr;

pub const OVERLAY_WIDTH: i32 = 260;
pub const OVERLAY_HEIGHT: i32 = 80;
pub const TIMER_ID: usize = 100;
pub const TIMER_INTERVAL_MS: u32 = 16; // ~60fps
pub const HOLD_STEPS: u8 = 12; // ~200ms at 16ms per step
pub const TOTAL_STEPS: u8 = 25; // ~400ms total

const CLASS_NAME: &str = "ClipDeformatOverlay";

/// Magenta color key — pixels painted this color become fully transparent.
const COLOR_KEY: COLORREF = COLORREF(0x00FF00FF);
/// Light green background.  RGB(200, 230, 201)
const BG_COLOR: COLORREF = COLORREF(0x00C9E6C8);
/// Dark green text/checkmark.  RGB(46, 125, 50)
const TEXT_COLOR: COLORREF = COLORREF(0x00327D2E);

const LWA_FLAGS: u32 = 0x03; // LWA_COLORKEY | LWA_ALPHA

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn register_class(hinstance: HINSTANCE) -> Result<(), windows::core::Error> {
    let class_name = to_wide(CLASS_NAME);
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(overlay_wndproc),
        hInstance: hinstance,
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };
    let atom = unsafe { RegisterClassExW(&wc) };
    if atom == 0 {
        return Err(windows::core::Error::from_thread());
    }
    Ok(())
}

pub fn create_window(hinstance: HINSTANCE) -> Result<HWND, windows::core::Error> {
    let class_name = to_wide(CLASS_NAME);

    let ex_style = WS_EX_LAYERED
        | WS_EX_TOPMOST
        | WS_EX_TOOLWINDOW
        | WS_EX_NOACTIVATE
        | WS_EX_TRANSPARENT;

    let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    let x = (screen_w - OVERLAY_WIDTH) / 2;
    let y = (screen_h - OVERLAY_HEIGHT) / 2;

    let hwnd = unsafe {
        CreateWindowExW(
            ex_style,
            PCWSTR(class_name.as_ptr()),
            PCWSTR::null(),
            WS_POPUP,
            x,
            y,
            OVERLAY_WIDTH,
            OVERLAY_HEIGHT,
            None,
            None,
            Some(hinstance),
            None,
        )?
    };

    // Start fully transparent, with color key active
    unsafe {
        SetLayeredWindowAttributes(
            hwnd,
            COLOR_KEY,
            0,
            windows::Win32::UI::WindowsAndMessaging::LAYERED_WINDOW_ATTRIBUTES_FLAGS(LWA_FLAGS),
        )?;
    }

    Ok(hwnd)
}

pub fn show(hwnd: HWND) {
    let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    let x = (screen_w - OVERLAY_WIDTH) / 2;
    let y = (screen_h - OVERLAY_HEIGHT) / 2;

    unsafe {
        let _ = SetWindowPos(
            hwnd,
            None,
            x,
            y,
            OVERLAY_WIDTH,
            OVERLAY_HEIGHT,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
        let _ = SetLayeredWindowAttributes(
            hwnd,
            COLOR_KEY,
            255,
            windows::Win32::UI::WindowsAndMessaging::LAYERED_WINDOW_ATTRIBUTES_FLAGS(LWA_FLAGS),
        );
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        let _ = SetTimer(Some(hwnd), TIMER_ID, TIMER_INTERVAL_MS, None);
    }
}

pub fn hide(hwnd: HWND) {
    unsafe {
        let _ = KillTimer(Some(hwnd), TIMER_ID);
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

/// Advance the fade animation. Returns true when complete.
pub fn tick_fade(hwnd: HWND, step: u8) -> bool {
    if step < HOLD_STEPS {
        // Hold phase: full opacity
        false
    } else if step >= TOTAL_STEPS {
        // Done
        hide(hwnd);
        true
    } else {
        // Fade phase: linear fade from 255 to 0
        let fade_progress = step - HOLD_STEPS;
        let fade_total = TOTAL_STEPS - HOLD_STEPS;
        let alpha = 255 - ((fade_progress as u16 * 255) / fade_total as u16) as u8;
        unsafe {
            let _ = SetLayeredWindowAttributes(
                hwnd,
                COLOR_KEY,
                alpha,
                windows::Win32::UI::WindowsAndMessaging::LAYERED_WINDOW_ATTRIBUTES_FLAGS(LWA_FLAGS),
            );
        }
        false
    }
}

pub fn paint(hwnd: HWND) {
    unsafe {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);
        if hdc.is_invalid() {
            return;
        }

        let mut rect = RECT::default();
        let _ = GetClientRect(hwnd, &mut rect);

        // Fill entire client area with color-key (these pixels become transparent)
        let key_brush = CreateSolidBrush(COLOR_KEY);
        FillRect(hdc, &rect, key_brush);
        let _ = DeleteObject(key_brush.into());

        // Draw rounded rectangle background in light green
        let bg_brush = CreateSolidBrush(BG_COLOR);
        let null_pen = GetStockObject(NULL_PEN);
        let old_brush = SelectObject(hdc, bg_brush.into());
        let old_pen = SelectObject(hdc, null_pen);
        let _ = RoundRect(hdc, 0, 0, rect.right + 1, rect.bottom + 1, 24, 24);
        SelectObject(hdc, old_pen);
        SelectObject(hdc, old_brush);

        // Set text drawing mode
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, TEXT_COLOR);

        // Draw checkmark
        let mut check_font_lf = LOGFONTW::default();
        check_font_lf.lfHeight = -28;
        let face = to_wide("Segoe UI Symbol");
        let face_len = face.len().min(32);
        check_font_lf.lfFaceName[..face_len].copy_from_slice(&face[..face_len]);
        let check_font = CreateFontIndirectW(&check_font_lf);
        let old_font = SelectObject(hdc, check_font.into());

        let mut check_rect = RECT {
            left: 0,
            top: 2,
            right: rect.right,
            bottom: 48,
        };
        let mut checkmark: Vec<u16> = vec![0x2714]; // Heavy checkmark
        DrawTextW(
            hdc,
            &mut checkmark,
            &mut check_rect,
            DT_CENTER | DT_SINGLELINE | DT_VCENTER,
        );

        // Draw "Clipboard cleaned" text
        let mut text_font_lf = LOGFONTW::default();
        text_font_lf.lfHeight = -15;
        text_font_lf.lfWeight = FW_SEMIBOLD.0 as i32;
        let face = to_wide("Segoe UI");
        let face_len = face.len().min(32);
        text_font_lf.lfFaceName[..face_len].copy_from_slice(&face[..face_len]);
        let text_font = CreateFontIndirectW(&text_font_lf);
        SelectObject(hdc, text_font.into());

        let mut text_rect = RECT {
            left: 0,
            top: 44,
            right: rect.right,
            bottom: rect.bottom,
        };
        let mut label: Vec<u16> = "Clipboard cleaned".encode_utf16().collect();
        DrawTextW(
            hdc,
            &mut label,
            &mut text_rect,
            DT_CENTER | DT_SINGLELINE | DT_VCENTER,
        );

        // Cleanup
        SelectObject(hdc, old_font);
        let _ = DeleteObject(check_font.into());
        let _ = DeleteObject(text_font.into());
        let _ = DeleteObject(bg_brush.into());

        let _ = EndPaint(hwnd, &ps);
    }
}

unsafe extern "system" fn overlay_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            paint(hwnd);
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == TIMER_ID => {
            if let Some(app) = app_ptr() {
                app.on_timer();
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
