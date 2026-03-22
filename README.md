# Clipboard Deformatter

A lightweight Windows 11 system tray app that strips clipboard formatting with a single hotkey.

Press **Win + Shift + C** and whatever is on your clipboard gets replaced with its plain-text version. A brief overlay confirms the action, then fades out. Like pasting through Notepad, without the Notepad.

## Features

- **Global hotkey** (Win + Shift + C) strips rich formatting from the clipboard
- **Visual confirmation**: centered overlay (light green, rounded rectangle) with checkmark and "Clipboard cleaned" text, 400ms fade
- **System tray** icon with right-click menu (Enable/Disable, Exit)
- **Auto-start** with Windows via registry
- **Single instance** enforcement (only one copy runs at a time)
- **Tiny footprint**: ~250KB standalone .exe, no runtime dependencies

## How it works

Windows stores clipboard data in multiple formats simultaneously (CF_UNICODETEXT, CF_HTML, CF_RTF, etc.). When you paste, the receiving app picks the richest format it supports. This tool reads the plain text (CF_UNICODETEXT), clears the clipboard, and writes back only the plain text.

## Build

Requires Rust and a Windows target toolchain.

```bash
# With GNU toolchain (cross-compile from Linux)
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

# With MSVC toolchain (native Windows)
cargo build --release --target x86_64-pc-windows-msvc
```

The output is a single `.exe` with no external dependencies.

## Install

1. Copy `clipboard-deformatter.exe` anywhere on your system
2. Run it
3. The app registers itself to start with Windows automatically

To uninstall: right-click the tray icon, click Exit, delete the exe, and optionally remove the `ClipboardDeformatter` value from `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.

## Technical details

- Written in Rust using the [`windows`](https://crates.io/crates/windows) crate (official Microsoft Win32 bindings)
- Raw Win32 APIs throughout: no GUI framework
- GDI rendering for the overlay with color-key transparency for rounded corners
- Shell_NotifyIcon for the tray
- DPI-aware via embedded PerMonitorV2 manifest

## License

GPL-3.0
