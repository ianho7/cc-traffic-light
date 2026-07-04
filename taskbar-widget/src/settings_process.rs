use std::{
    env,
    path::PathBuf,
    process::{Child, Command},
    sync::{Mutex, OnceLock},
};

use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM},
    UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible, SW_RESTORE, SetForegroundWindow,
        ShowWindow,
    },
};

use crate::win32;

const SETTINGS_HOST_ENV: &str = "CC_TRAFFIC_LIGHT_SETTINGS_HOST";
const SETTINGS_HOST_TAURI: &str = "tauri";
const SETTINGS_HOST_SLINT: &str = "slint";
const SETTINGS_EXE_ENV: &str = "CC_TRAFFIC_LIGHT_TAURI_SETTINGS_EXE";

static SETTINGS_PROCESS: OnceLock<Mutex<SettingsProcessState>> = OnceLock::new();

struct SettingsProcessState {
    child: Option<Child>,
}

impl SettingsProcessState {
    fn shared() -> &'static Mutex<Self> {
        SETTINGS_PROCESS.get_or_init(|| Mutex::new(Self { child: None }))
    }
}

pub fn tauri_settings_enabled() -> bool {
    tauri_settings_enabled_from_host_env(env::var(SETTINGS_HOST_ENV).ok().as_deref())
}

fn tauri_settings_enabled_from_host_env(value: Option<&str>) -> bool {
    value
        .map(|value| {
            if value.eq_ignore_ascii_case(SETTINGS_HOST_SLINT) {
                return false;
            }
            value.eq_ignore_ascii_case(SETTINGS_HOST_TAURI) || !value.is_empty()
        })
        .unwrap_or(true)
}

pub fn open_or_focus_tauri_settings() -> Result<bool, String> {
    if !tauri_settings_enabled() {
        return Ok(false);
    }

    let state = SettingsProcessState::shared();
    let Ok(mut state) = state.lock() else {
        return Err("settings process lock poisoned".to_string());
    };

    if let Some(child) = state.child.as_mut() {
        match child.try_wait() {
            Ok(None) => {
                if focus_child_window(child.id()) {
                    win32::runtime_debug_log(&format!(
                        "[settings-process] reused existing tauri settings pid={}",
                        child.id()
                    ));
                    return Ok(true);
                }

                win32::runtime_debug_log(&format!(
                    "[settings-process] live tauri settings pid={} had no visible top-level window; respawning",
                    child.id()
                ));
                terminate_child(child, "stale_window");
                state.child = None;
            }
            Ok(Some(status)) => {
                win32::runtime_debug_log(&format!(
                    "[settings-process] previous tauri settings exited status={status}"
                ));
                state.child = None;
            }
            Err(error) => {
                win32::runtime_debug_log(&format!(
                    "[settings-process] child status check failed error={error}"
                ));
                state.child = None;
            }
        }
    }

    let exe_path = resolve_tauri_settings_exe()?;
    let child = Command::new(&exe_path)
        .spawn()
        .map_err(|error| format!("failed to spawn {}: {error}", exe_path.display()))?;
    let pid = child.id();
    state.child = Some(child);
    win32::runtime_debug_log(&format!(
        "[settings-process] spawned tauri settings pid={} path={}",
        pid,
        exe_path.display()
    ));
    Ok(true)
}

pub fn shutdown_managed_tauri_settings() {
    let state = SettingsProcessState::shared();
    let Ok(mut state) = state.lock() else {
        return;
    };
    let Some(child) = state.child.as_mut() else {
        return;
    };

    match child.try_wait() {
        Ok(Some(status)) => {
            win32::runtime_debug_log(&format!(
                "[settings-process] cleared exited tauri settings pid={} status={status}",
                child.id()
            ));
        }
        Ok(None) => {
            terminate_child(child, "host_shutdown");
        }
        Err(error) => {
            win32::runtime_debug_log(&format!(
                "[settings-process] child status check during shutdown failed pid={} error={error}",
                child.id()
            ));
            terminate_child(child, "host_shutdown_after_status_error");
        }
    }
    state.child = None;
}

fn resolve_tauri_settings_exe() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os(SETTINGS_EXE_ENV) {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Ok(candidate);
        }
        return Err(format!(
            "env override {} does not point to a file: {}",
            SETTINGS_EXE_ENV,
            candidate.display()
        ));
    }

    let current_exe = env::current_exe().map_err(|error| error.to_string())?;
    let current_dir = current_exe
        .parent()
        .ok_or_else(|| "current exe has no parent directory".to_string())?;

    let candidates = [
        current_dir.join("taskbar-settings-tauri.exe"),
        current_dir
            .join("taskbar-settings-tauri")
            .with_extension("exe"),
        current_dir
            .parent()
            .map(|dir| dir.join("debug").join("taskbar-settings-tauri.exe"))
            .unwrap_or_default(),
        current_dir
            .parent()
            .map(|dir| dir.join("release").join("taskbar-settings-tauri.exe"))
            .unwrap_or_default(),
    ];

    candidates
        .into_iter()
        .find(|candidate| !candidate.as_os_str().is_empty() && candidate.is_file())
        .ok_or_else(|| {
            format!(
                "tauri settings executable not found; checked near {} and env {}",
                current_exe.display(),
                SETTINGS_EXE_ENV
            )
        })
}

fn focus_child_window(target_pid: u32) -> bool {
    let mut context = WindowSearchContext {
        target_pid,
        hwnd: None,
    };

    let _ = unsafe {
        EnumWindows(
            Some(enum_windows_for_process),
            LPARAM((&mut context as *mut WindowSearchContext) as isize),
        )
    };

    if let Some(hwnd) = context.hwnd {
        unsafe {
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(hwnd);
        }
        true
    } else {
        false
    }
}

fn terminate_child(child: &mut Child, reason: &str) {
    let pid = child.id();
    let _ = child.kill();
    let _ = child.wait();
    win32::runtime_debug_log(&format!(
        "[settings-process] terminated managed tauri settings pid={} reason={}",
        pid, reason
    ));
}

struct WindowSearchContext {
    target_pid: u32,
    hwnd: Option<HWND>,
}

unsafe extern "system" fn enum_windows_for_process(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let context = unsafe { &mut *(lparam.0 as *mut WindowSearchContext) };
    let mut process_id = 0u32;
    unsafe {
        let _ = GetWindowThreadProcessId(hwnd, Some(&mut process_id));
    }

    if process_id == context.target_pid && unsafe { IsWindowVisible(hwnd) }.as_bool() {
        context.hwnd = Some(hwnd);
        BOOL(0)
    } else {
        BOOL(1)
    }
}

#[cfg(test)]
mod tests {
    use super::tauri_settings_enabled_from_host_env;

    #[test]
    fn host_env_defaults_to_tauri_when_missing() {
        assert!(tauri_settings_enabled_from_host_env(None));
    }

    #[test]
    fn host_env_allows_explicit_slint_fallback() {
        assert!(!tauri_settings_enabled_from_host_env(Some("slint")));
        assert!(!tauri_settings_enabled_from_host_env(Some("SLINT")));
    }

    #[test]
    fn host_env_keeps_tauri_enabled_for_non_empty_values() {
        assert!(tauri_settings_enabled_from_host_env(Some("tauri")));
        assert!(tauri_settings_enabled_from_host_env(Some("custom")));
    }
}
