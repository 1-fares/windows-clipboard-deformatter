use windows::Win32::Foundation::COLORREF;
use windows::Win32::Graphics::Dwm::DwmGetColorizationColor;

/// Get the Windows accent color as a COLORREF (0x00BBGGRR).
/// Falls back to a blue if DWM query fails.
pub fn get_accent_color() -> COLORREF {
    let mut colorization: u32 = 0;
    let mut opaque_blend = windows::core::BOOL::default();

    let result = unsafe { DwmGetColorizationColor(&mut colorization, &mut opaque_blend) };

    if result.is_ok() {
        // DWM returns AARRGGBB, COLORREF expects 0x00BBGGRR
        let r = (colorization >> 16) & 0xFF;
        let g = (colorization >> 8) & 0xFF;
        let b = colorization & 0xFF;
        COLORREF(b | (g << 8) | (r << 16))
    } else {
        // Fallback: Windows default blue
        COLORREF(0x00D77800)
    }
}
