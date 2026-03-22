# Clipboard Deformatter

Windows 11 system tray app that strips clipboard formatting via a global hotkey (Win + `). Written in Rust using raw Win32 APIs via the `windows` crate.

## Build

```bash
cargo build --release --target x86_64-pc-windows-gnu
# or with MSVC
cargo build --release --target x86_64-pc-windows-msvc
```

## Architecture

Single binary, no GUI framework. All UI via raw Win32 (GDI for overlay, Shell_NotifyIcon for tray).

- `src/main.rs` - Entry point, single-instance mutex, message loop, WndProc
- `src/app.rs` - App state struct coordinating all modules
- `src/hotkey.rs` - Global hotkey (RegisterHotKey with MOD_WIN + VK_OEM_3)
- `src/clipboard.rs` - CF_UNICODETEXT read/clear/write
- `src/overlay.rs` - Layered popup window with GDI rendering and fade animation
- `src/tray.rs` - System tray icon and context menu
- `src/startup.rs` - Auto-start via HKCU registry
- `src/accent.rs` - Windows accent color via DwmGetColorizationColor
- `src/error.rs` - Unified error type

## Key Details

- Overlay: 260x80px, accent-colored rounded rect, white checkmark + text, 400ms fade
- Hotkey: Win + ` (backtick). VK_OEM_3 with MOD_WIN.
- Single instance enforced via named mutex `Global\ClipboardDeformatterMutex`
- No console window: `#![windows_subsystem = "windows"]`
- DPI-aware via embedded manifest (PerMonitorV2)
