use std::sync::{Mutex, OnceLock};

use shared_core::{
    settings_service::{SettingsService, SettingsServiceError},
    tauri_ipc::SettingsSaveResultDto,
};
use taskbar_widget::{
    agent_state,
    app_config::{self, AppConfig},
    ui_state::AppStatusSnapshot,
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

pub struct HostSettingsBridge;

impl HostSettingsBridge {
    pub fn new() -> Self {
        Self
    }
}

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
    HostSettingsBridge::new()
        .save_settings(&config)
        .map_err(service_error_to_string)?;
    invalidate_main_window();
    invalidate_fallback_window();
    Ok(config.clone())
}

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
    HostSettingsBridge::new()
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
    HostSettingsBridge::new()
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

fn changed_keys(previous: &AppConfig, next: &AppConfig) -> Vec<String> {
    let mut keys = Vec::new();

    if previous.localization.language != next.localization.language {
        keys.push("localization.language".to_string());
    }
    if previous.general.autostart_enabled != next.general.autostart_enabled {
        keys.push("general.autostart_enabled".to_string());
    }
    if previous.general.start_minimized_to_tray != next.general.start_minimized_to_tray {
        keys.push("general.start_minimized_to_tray".to_string());
    }
    if previous.general.close_to_tray != next.general.close_to_tray {
        keys.push("general.close_to_tray".to_string());
    }
    if previous.monitoring.codex_enabled != next.monitoring.codex_enabled {
        keys.push("monitoring.codex_enabled".to_string());
    }
    if previous.monitoring.claude_enabled != next.monitoring.claude_enabled {
        keys.push("monitoring.claude_enabled".to_string());
    }
    if previous.appearance.ui_theme != next.appearance.ui_theme {
        keys.push("appearance.ui_theme".to_string());
    }
    if previous.appearance.indicator_style != next.appearance.indicator_style {
        keys.push("appearance.indicator_style".to_string());
    }
    if previous.appearance.widget_size != next.appearance.widget_size {
        keys.push("appearance.widget_size".to_string());
    }
    if previous.appearance.show_labels != next.appearance.show_labels {
        keys.push("appearance.show_labels".to_string());
    }
    if previous.appearance.reduced_motion != next.appearance.reduced_motion {
        keys.push("appearance.reduced_motion".to_string());
    }
    if previous.widget_visual.palette.green != next.widget_visual.palette.green {
        keys.push("widget_visual.palette.green".to_string());
    }
    if previous.widget_visual.placement != next.widget_visual.placement {
        keys.push("widget_visual.placement".to_string());
    }
    if previous.widget_visual.palette.yellow != next.widget_visual.palette.yellow {
        keys.push("widget_visual.palette.yellow".to_string());
    }
    if previous.widget_visual.palette.red != next.widget_visual.palette.red {
        keys.push("widget_visual.palette.red".to_string());
    }
    if previous.widget_visual.palette.inactive_brightness_percent
        != next.widget_visual.palette.inactive_brightness_percent
    {
        keys.push("widget_visual.palette.inactive_brightness_percent".to_string());
    }
    if previous.diagnostics.last_opened_page != next.diagnostics.last_opened_page {
        keys.push("diagnostics.last_opened_page".to_string());
    }
    if previous.diagnostics.last_manual_refresh_at != next.diagnostics.last_manual_refresh_at {
        keys.push("diagnostics.last_manual_refresh_at".to_string());
    }

    keys
}
