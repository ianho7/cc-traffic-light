# Settings process console-window diagnosis (2026-07-13)

- Diagnosis: the Win32 tray host launched `taskbar-settings-tauri.exe` with a plain `Command::spawn()`. If that executable is built with the console subsystem, Windows creates a visible console window in addition to the Tauri settings window.
- Change: the host now launches the managed settings process with `CREATE_NO_WINDOW`. This is scoped to the tray-host launch path and does not alter Tauri window creation or the host/settings IPC boundary.
- Validation: run `cargo check -p taskbar-widget --offline`; manually confirm that opening Settings from the tray shows only the Tauri window using the root `target\\debug\\taskbar-widget.exe` or root `target\\release\\taskbar-widget.exe` being tested.
