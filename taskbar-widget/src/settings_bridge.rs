use std::sync::{Mutex, OnceLock};

use shared_core::{
    app_config::changed_keys,
    settings_service::{SettingsService, SettingsServiceError},
    tauri_ipc::{HookStatus, HookStatusDto, SettingsSaveResultDto},
};
use taskbar_widget::{
    agent_state,
    app_config::{self, AppConfig},
    ui_state::{AppStatusSnapshot, SourceId},
};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    Graphics::Gdi::InvalidateRect,
    UI::WindowsAndMessaging::PostMessageW,
};

use crate::win32;
use crate::{autostart, tray_icon};

static SETTINGS_SNAPSHOT: OnceLock<Mutex<AppStatusSnapshot>> = OnceLock::new();
static SETTINGS_CONFIG: OnceLock<Mutex<AppConfig>> = OnceLock::new();
static MAIN_HWND: OnceLock<Mutex<isize>> = OnceLock::new();
static SETTINGS_WINDOW_HWND: OnceLock<Mutex<isize>> = OnceLock::new();
const HOOK_ACTIVITY_WINDOW_MS: u64 = 5 * 60 * 1_000;

pub struct HostSettingsBridge;

impl SettingsService for HostSettingsBridge {
    fn load_settings(&self) -> Result<AppConfig, SettingsServiceError> {
        Ok(refresh_config_from_disk())
    }

    fn save_settings(&self, config: &AppConfig) -> Result<(), SettingsServiceError> {
        app_config::save_config(config)
            .map_err(|error| SettingsServiceError::Persistence(error.to_string()))
    }

    fn read_status_snapshot(&self) -> Result<AppStatusSnapshot, SettingsServiceError> {
        Ok(current_snapshot())
    }
}

pub fn initialize(snapshot: AppStatusSnapshot, config: AppConfig) {
    let _ = SETTINGS_SNAPSHOT.set(Mutex::new(snapshot));
    let _ = SETTINGS_CONFIG.set(Mutex::new(config));
}

pub fn bind_main_window(hwnd: HWND) {
    let lock = MAIN_HWND.get_or_init(|| Mutex::new(0));
    if let Ok(mut current) = lock.lock() {
        *current = hwnd.0 as isize;
    }
}

pub fn register_settings_window(hwnd: HWND) {
    let lock = SETTINGS_WINDOW_HWND.get_or_init(|| Mutex::new(0));
    if let Ok(mut current) = lock.lock() {
        *current = hwnd.0 as isize;
    }
}

pub fn update_snapshot(snapshot: AppStatusSnapshot) {
    if let Some(lock) = SETTINGS_SNAPSHOT.get()
        && let Ok(mut current) = lock.lock()
    {
        *current = snapshot;
    }

    invalidate_main_window();
    invalidate_fallback_window();
}

pub fn current_snapshot() -> AppStatusSnapshot {
    SETTINGS_SNAPSHOT
        .get()
        .and_then(|snapshot| snapshot.lock().ok().map(|snapshot| snapshot.clone()))
        .unwrap_or_else(AppStatusSnapshot::empty)
}

pub fn current_config() -> AppConfig {
    SETTINGS_CONFIG
        .get()
        .and_then(|config| config.lock().ok().map(|config| config.clone()))
        .unwrap_or_else(AppConfig::default_v1)
}

pub fn refresh_config_from_disk() -> AppConfig {
    let next = app_config::load_config_diagnostic().config;
    let lock = SETTINGS_CONFIG.get_or_init(|| Mutex::new(AppConfig::default_v1()));
    if let Ok(mut current) = lock.lock() {
        *current = next.clone();
    }
    next
}

pub fn update_config<F>(mutate: F) -> Result<AppConfig, String>
where
    F: FnOnce(&mut AppConfig),
{
    let Some(lock) = SETTINGS_CONFIG.get() else {
        return Err("settings config store unavailable".to_string());
    };
    let Ok(mut config) = lock.lock() else {
        return Err("settings config lock poisoned".to_string());
    };
    mutate(&mut config);
    HostSettingsBridge
        .save_settings(&config)
        .map_err(service_error_to_string)?;
    invalidate_main_window();
    invalidate_fallback_window();
    Ok(config.clone())
}

/// Update the in-memory config cache **without** persisting to disk.
/// Use this for runtime-only state (e.g. `last_manual_refresh_at`) that
/// does not need to survive a restart.
fn update_runtime_config<F>(mutate: F) -> Result<AppConfig, String>
where
    F: FnOnce(&mut AppConfig),
{
    let Some(lock) = SETTINGS_CONFIG.get() else {
        return Err("settings config store unavailable".to_string());
    };
    let Ok(mut config) = lock.lock() else {
        return Err("settings config lock poisoned".to_string());
    };
    mutate(&mut config);
    invalidate_fallback_window();
    Ok(config.clone())
}

pub fn toggle_autostart_setting() -> Result<AppConfig, String> {
    let Some(lock) = SETTINGS_CONFIG.get() else {
        return Err("settings config store unavailable".to_string());
    };
    let Ok(mut config) = lock.lock() else {
        return Err("settings config lock poisoned".to_string());
    };

    let next_enabled = !config.general.autostart_enabled;
    autostart::set_enabled(next_enabled).map_err(|error| error.to_string())?;
    config.general.autostart_enabled = next_enabled;
    HostSettingsBridge
        .save_settings(&config)
        .map_err(service_error_to_string)?;
    invalidate_main_window();
    invalidate_fallback_window();
    Ok(config.clone())
}

pub fn request_manual_refresh_command() -> Result<AppConfig, String> {
    let Some(main_hwnd) = current_main_hwnd() else {
        return Err("main window unavailable for refresh command".to_string());
    };
    let post_result = unsafe {
        PostMessageW(
            main_hwnd,
            windows::Win32::UI::WindowsAndMessaging::WM_COMMAND,
            WPARAM(usize::from(tray_icon::TRAY_CMD_REFRESH)),
            LPARAM(0),
        )
    };
    if post_result.is_err() {
        return Err(format!(
            "PostMessageW refresh failed last_error={}",
            win32::last_error_code()
        ));
    }

    update_runtime_config(|config| {
        config.diagnostics.last_manual_refresh_at = Some(agent_state::now_ms());
    })
}

pub fn apply_full_settings(next: AppConfig) -> Result<SettingsSaveResultDto, String> {
    let previous = current_config();
    if previous.general.autostart_enabled != next.general.autostart_enabled {
        autostart::set_enabled(next.general.autostart_enabled)
            .map_err(|error| error.to_string())?;
    }

    let Some(lock) = SETTINGS_CONFIG.get() else {
        return Err("settings config store unavailable".to_string());
    };
    let Ok(mut config) = lock.lock() else {
        return Err("settings config lock poisoned".to_string());
    };

    let applied_keys = changed_keys(&previous, &next);
    *config = next.clone();
    HostSettingsBridge
        .save_settings(&config)
        .map_err(service_error_to_string)?;
    invalidate_main_window();
    invalidate_fallback_window();

    Ok(SettingsSaveResultDto {
        settings: next,
        applied_keys,
    })
}

pub fn notify_settings_applied(applied_keys: &[String]) {
    win32::runtime_debug_log(&format!(
        "[tauri-ipc] settings applied keys={}",
        applied_keys.join(",")
    ));
    if should_request_runtime_refresh(applied_keys) {
        let _ = post_refresh_command();
    }
}

fn current_main_hwnd() -> Option<HWND> {
    MAIN_HWND
        .get()
        .and_then(|lock| {
            lock.lock()
                .ok()
                .map(|value| HWND(*value as *mut std::ffi::c_void))
        })
        .filter(|hwnd| hwnd.0 != std::ptr::null_mut())
}

fn current_settings_window_hwnd() -> Option<HWND> {
    SETTINGS_WINDOW_HWND
        .get()
        .and_then(|lock| {
            lock.lock()
                .ok()
                .map(|value| HWND(*value as *mut std::ffi::c_void))
        })
        .filter(|hwnd| hwnd.0 != std::ptr::null_mut())
}

fn invalidate_fallback_window() {
    if let Some(hwnd) = current_settings_window_hwnd() {
        unsafe {
            let _ = InvalidateRect(hwnd, None, true);
        }
    }
}

fn invalidate_main_window() {
    if let Some(hwnd) = current_main_hwnd() {
        unsafe {
            let _ = InvalidateRect(hwnd, None, true);
        }
    }
}

fn post_refresh_command() -> Result<(), String> {
    let Some(main_hwnd) = current_main_hwnd() else {
        return Err("main window unavailable for refresh command".to_string());
    };
    unsafe {
        let _ = PostMessageW(
            main_hwnd,
            windows::Win32::UI::WindowsAndMessaging::WM_COMMAND,
            WPARAM(usize::from(tray_icon::TRAY_CMD_REFRESH)),
            LPARAM(0),
        );
    }
    Ok(())
}

fn should_request_runtime_refresh(applied_keys: &[String]) -> bool {
    applied_keys
        .iter()
        .any(|key| key != "diagnostics.last_opened_page")
}

fn service_error_to_string(error: SettingsServiceError) -> String {
    match error {
        SettingsServiceError::StoreUnavailable(message)
        | SettingsServiceError::Persistence(message)
        | SettingsServiceError::SnapshotUnavailable(message)
        | SettingsServiceError::Refresh(message) => message,
    }
}

/// Detect whether Codex and Claude Code hooks are installed and active.
pub fn detect_hook_status() -> HookStatusDto {
    HookStatusDto {
        codex: detect_agent_hook_status("codex"),
        claude: detect_agent_hook_status("claude"),
    }
}

pub fn should_show_hook_notification(status: &HookStatusDto) -> bool {
    let all_ready =
        matches!(status.codex, HookStatus::Active) && matches!(status.claude, HookStatus::Active);
    let next_key = hook_notification_key(status);
    let current_key = current_config().diagnostics.last_hook_notification_key;

    if all_ready {
        if current_key.is_some() {
            if let Err(error) = update_config(|config| {
                config.diagnostics.last_hook_notification_key = None;
            }) {
                win32::runtime_debug_log(&format!(
                    "[hook-state] failed to clear startup notification key: {error}"
                ));
            }
        }
        return false;
    }

    let Some(next_key) = next_key else {
        return false;
    };
    if current_key.as_deref() == Some(next_key.as_str()) {
        return false;
    }

    if let Err(error) = update_config(|config| {
        config.diagnostics.last_hook_notification_key = Some(next_key.clone());
    }) {
        win32::runtime_debug_log(&format!(
            "[hook-state] failed to persist startup notification key: {error}"
        ));
    }
    true
}

fn hook_notification_key(status: &HookStatusDto) -> Option<String> {
    Some(format!(
        "codex:{};claude:{}",
        status.codex.as_str(),
        status.claude.as_str()
    ))
}

fn detect_agent_hook_status(agent: &str) -> HookStatus {
    let hooks_path = std::env::var_os("USERPROFILE")
        .map(|p| {
            let mut path = std::path::PathBuf::from(p);
            match agent {
                "claude" => path.extend([".claude", "settings.json"]),
                _ => path.extend([".codex", "hooks.json"]),
            }
            path
        })
        .unwrap_or_else(|| match agent {
            "claude" => std::path::PathBuf::from(".claude/settings.json"),
            _ => std::path::PathBuf::from(".codex/hooks.json"),
        });

    let hooks_text = std::fs::read_to_string(&hooks_path).ok();
    let hooks_invalid = hooks_text
        .as_deref()
        .is_some_and(|text| serde_json::from_str::<serde_json::Value>(text).is_err());
    let hooks_installed = hooks_text
        .as_deref()
        .is_some_and(|text| text.contains("CcTrafficLight") && text.contains(agent));

    let state_path = agent_state::state_file_path();
    let state_text = std::fs::read_to_string(&state_path).ok();
    let state_invalid = state_text
        .as_deref()
        .is_some_and(|text| serde_json::from_str::<serde_json::Value>(text).is_err());
    let state_recent = state_text
        .as_deref()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
        .is_some_and(|value| state_has_recent_activity(&value, agent, agent_state::now_ms()));

    classify_hook_status(
        agent,
        hooks_installed,
        state_recent,
        hooks_invalid || state_invalid,
    )
}

fn classify_hook_status(
    _agent: &str,
    hooks_installed: bool,
    state_recent: bool,
    has_error: bool,
) -> HookStatus {
    if has_error {
        return HookStatus::Error;
    }

    match (hooks_installed, state_recent) {
        (true, true) => HookStatus::Active,
        (true, false) => HookStatus::ConfiguredUnverified,
        (false, false) => HookStatus::NotInstalled,
        _ => HookStatus::NotInstalled,
    }
}

fn state_has_recent_activity(value: &serde_json::Value, agent: &str, now_ms: u64) -> bool {
    value
        .get("agents")
        .and_then(|agents| agents.get(agent))
        .and_then(|summary| summary.get("updated_at"))
        .and_then(|updated_at| updated_at.as_u64())
        .is_some_and(|updated_at| now_ms.saturating_sub(updated_at) <= HOOK_ACTIVITY_WINDOW_MS)
}

/// Run install-codex-hooks.ps1 to deploy global Codex hooks.
pub fn install_codex_hooks() -> (bool, String) {
    install_hooks("codex")
}

pub fn install_claude_hooks() -> (bool, String) {
    install_hooks("claude")
}

pub fn uninstall_codex_hooks() -> (bool, String) {
    uninstall_hooks("codex", SourceId::Codex)
}

pub fn uninstall_claude_hooks() -> (bool, String) {
    uninstall_hooks("claude", SourceId::Claude)
}

fn install_hooks(agent: &str) -> (bool, String) {
    let script_path = get_install_script_path(agent);
    if !script_path.exists() {
        return (
            false,
            format!("install script not found: {}", script_path.display()),
        );
    }

    match std::process::Command::new("powershell.exe")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_path.to_string_lossy(),
            "-Apply",
        ])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                (true, "hooks deployed successfully".to_string())
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let detail = if stderr.trim().is_empty() {
                    stdout.trim()
                } else {
                    stderr.trim()
                };
                (
                    false,
                    format!(
                        "hooks deployment failed (exit={}): {}",
                        output.status,
                        if detail.is_empty() {
                            "no diagnostic output"
                        } else {
                            detail
                        }
                    ),
                )
            }
        }
        Err(error) => (
            false,
            format!("failed to start {agent} hook installer: {error}"),
        ),
    }
}

fn uninstall_hooks(agent: &str, source_id: SourceId) -> (bool, String) {
    let script_path = get_install_script_path(agent);
    if !script_path.exists() {
        return (
            false,
            format!("install script not found: {}", script_path.display()),
        );
    }

    match std::process::Command::new("powershell.exe")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_path.to_string_lossy(),
            "-Uninstall",
            "-Apply",
        ])
        .output()
    {
        Ok(output) if output.status.success() => match agent_state::clear_agent_tasks(source_id) {
            Ok(_) => (true, format!("{agent} hooks removed and historical state cleared")),
            Err(error) => (
                false,
                format!("{agent} hooks removed, but state cleanup failed: {error}"),
            ),
        },
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if stderr.trim().is_empty() {
                stdout.trim()
            } else {
                stderr.trim()
            };
            (false, format!("{agent} hook removal failed (exit={}): {detail}", output.status))
        }
        Err(error) => (false, format!("failed to start {agent} hook uninstaller: {error}")),
    }
}

fn get_install_script_path(agent: &str) -> std::path::PathBuf {
    let script_name = format!("install-{agent}-hooks.ps1");
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let local = exe_dir.join("scripts").join(&script_name);
            if local.exists() {
                return local;
            }
        }
    }
    std::env::var_os("LOCALAPPDATA")
        .map(|p| {
            let mut path = std::path::PathBuf::from(p);
            path.extend([
                "Programs",
                "CC Traffic Light",
                "scripts",
            ]);
            path.push(&script_name);
            path
        })
        .unwrap_or_else(|| std::path::PathBuf::from(script_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn recent_activity_remains_active_after_task_completion() {
        let value = json!({
            "agents": {
                "codex": {
                    "updated_at": 9_000,
                    "active_task_count": 0,
                    "state": "idle"
                }
            }
        });

        assert!(state_has_recent_activity(&value, "codex", 9_000 + 1_000));
    }

    #[test]
    fn old_activity_is_not_active() {
        let value = json!({
            "agents": {
                "codex": {
                    "updated_at": 1_000
                }
            }
        });

        assert!(!state_has_recent_activity(
            &value,
            "codex",
            1_000 + HOOK_ACTIVITY_WINDOW_MS + 1
        ));
    }

    #[test]
    fn classifies_configured_without_recent_event_as_unverified() {
        assert_eq!(
            classify_hook_status("codex", true, false, false),
            HookStatus::ConfiguredUnverified
        );
    }

    #[test]
    fn classifies_recent_configured_event_as_active() {
        assert_eq!(
            classify_hook_status("codex", true, true, false),
            HookStatus::Active
        );
    }

    #[test]
    fn classifies_unconfigured_claude_as_not_installed() {
        assert_eq!(
            classify_hook_status("claude", false, false, false),
            HookStatus::NotInstalled
        );
    }

    #[test]
    fn classifies_invalid_configuration_as_error() {
        assert_eq!(
            classify_hook_status("codex", false, false, true),
            HookStatus::Error
        );
    }

    #[test]
    fn hook_notification_key_is_stable_for_same_status() {
        let status = HookStatusDto {
            codex: HookStatus::ConfiguredUnverified,
            claude: HookStatus::ProcessOnly,
        };

        assert_eq!(
            hook_notification_key(&status).as_deref(),
            Some("codex:configured_unverified;claude:process_only")
        );
    }
}
