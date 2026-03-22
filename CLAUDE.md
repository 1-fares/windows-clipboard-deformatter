# Clipboard Deformatter

Windows 11 system tray app that strips clipboard formatting via a global hotkey (Win + Shift + C). Written in Rust using raw Win32 APIs via the `windows` crate.

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
- `src/hotkey.rs` - Global hotkey via RegisterHotKey (Win + Shift + C)
- `src/clipboard.rs` - CF_UNICODETEXT read/clear/write
- `src/overlay.rs` - Layered popup with GDI rendering, color-key transparency, fade animation
- `src/tray.rs` - System tray icon and context menu
- `src/startup.rs` - Auto-start via HKCU registry
- `src/error.rs` - Unified error type
- `examples/hook_test.rs` - Diagnostic tool: tests LL hook suppression of system hotkeys

## Key Details

- Overlay: 260x80px, light green rounded rect with color-key transparency, dark green text, 400ms fade
- Hotkey: Win + Shift + C via RegisterHotKey (MOD_WIN | MOD_SHIFT | MOD_NOREPEAT)
- Single instance enforced via named mutex `Global\ClipboardDeformatterMutex`
- No console window: `#![windows_subsystem = "windows"]`
- DPI-aware via embedded manifest (PerMonitorV2)
