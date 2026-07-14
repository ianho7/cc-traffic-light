use std::{
    env,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

use shared_core::{
    app_config::{AppConfig, changed_keys, config_file_path, default_widget_palette},
    settings_service::{SourceStatusView, StatusSnapshotView},
    tauri_ipc::{
        HookDiagnosticPathsDto, HookDiagnosticsDto, HookStatusDto, RuntimeLogDiagnosticsDto,
        SettingsAboutMetadataDto,
        SettingsBootstrapDto, SettingsIpcCommand, SettingsIpcEnvelope, SettingsIpcResponse,
        SettingsIpcResponseEnvelope, SettingsRefreshResultDto, SettingsSaveResultDto,
        SettingsTransportDto, TAURI_SETTINGS_PIPE_NAME, TAURI_SETTINGS_PROTOCOL_VERSION,
    },
};
use windows::{
    Win32::System::Pipes::{CallNamedPipeW, WaitNamedPipeW},
    core::PCWSTR,
};

#[derive(Clone)]
struct FakeBackendState {
    settings: AppConfig,
    snapshot: StatusSnapshotView,
}

static FAKE_STATE: OnceLock<Mutex<FakeBackendState>> = OnceLock::new();
static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);
const FAKE_BACKEND_ENV: &str = "CC_TRAFFIC_LIGHT_TAURI_FAKE_BACKEND";

fn state() -> &'static Mutex<FakeBackendState> {
    FAKE_STATE.get_or_init(|| {
        Mutex::new(FakeBackendState {
            settings: AppConfig::default_v1(),
            snapshot: fake_snapshot(),
        })
    })
}

fn fake_snapshot() -> StatusSnapshotView {
    let mut sources = std::collections::BTreeMap::new();
    sources.insert(
        "codex".to_string(),
        SourceStatusView {
            source_id: "codex".to_string(),
            state: "working".to_string(),
            confidence: "confirmed".to_string(),
            method: "hook_state".to_string(),
            updated_at: 1_783_066_000_000,
            message: Some("tauri_fake_codex_task".to_string()),
        },
    );
    sources.insert(
        "claude".to_string(),
        SourceStatusView {
            source_id: "claude".to_string(),
            state: "idle".to_string(),
            confidence: "degraded".to_string(),
            method: "process".to_string(),
            updated_at: 1_783_066_000_000,
            message: Some("process_present_only".to_string()),
        },
    );
    StatusSnapshotView {
        widget_mount_state: "attached".to_string(),
        overall_state: "working".to_string(),
        last_widget_attach_at: Some(1_783_066_000_000),
        last_detection_refresh_at: Some(1_783_066_000_000),
        last_error_summary: None,
        sources,
    }
}

fn fake_hook_diagnostics() -> HookDiagnosticsDto {
    HookDiagnosticsDto {
        codex: HookDiagnosticPathsDto {
            config_path: r"C:\Users\fake\.codex\hooks.json".to_string(),
            config_exists: true,
            backup_path: r"C:\Users\fake\.codex\hooks.json.cc-traffic-light-global-hooks.bak"
                .to_string(),
            backup_exists: true,
            hook_executable_path: r"C:\Program Files\CC Traffic Light\taskbar_widget_hook.exe"
                .to_string(),
            hook_executable_exists: true,
        },
        claude: HookDiagnosticPathsDto {
            config_path: r"C:\Users\fake\.claude\settings.json".to_string(),
            config_exists: true,
            backup_path: r"C:\Users\fake\.claude\settings.json.cc-traffic-light-hooks.bak"
                .to_string(),
            backup_exists: false,
            hook_executable_path: r"C:\Program Files\CC Traffic Light\taskbar_widget_hook.exe"
                .to_string(),
            hook_executable_exists: true,
        },
    }
}

fn fake_runtime_log_diagnostics() -> RuntimeLogDiagnosticsDto {
    RuntimeLogDiagnosticsDto {
        directory_path: r"C:\Users\fake\AppData\Local\CC Traffic Light\logs".to_string(),
        runtime_log_path: r"C:\Users\fake\AppData\Local\CC Traffic Light\logs\runtime.log".to_string(),
        runtime_log_exists: true,
    }
}

fn about_metadata() -> SettingsAboutMetadataDto {
    SettingsAboutMetadataDto {
        product_name: "CC Traffic Light".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        runtime_description: "Win32 host + Tauri settings".to_string(),
        config_path: config_file_path().display().to_string(),
    }
}

fn next_request_id() -> String {
    format!("req-{}", REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn fake_backend_enabled() -> bool {
    env::var(FAKE_BACKEND_ENV)
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "True"))
        .unwrap_or(false)
}

fn wide_null(value: &str) -> Vec<u16> {
    let mut wide: Vec<u16> = value.encode_utf16().collect();
    wide.push(0);
    wide
}

fn call_pipe(command: SettingsIpcCommand) -> Result<SettingsIpcResponse, String> {
    let envelope = SettingsIpcEnvelope {
        protocol_version: TAURI_SETTINGS_PROTOCOL_VERSION.to_string(),
        request_id: next_request_id(),
        command,
    };
    let request_id = envelope.request_id.clone();
    let payload = serde_json::to_vec(&envelope).map_err(|e| e.to_string())?;
    let pipe_name = wide_null(TAURI_SETTINGS_PIPE_NAME);
    let _ = unsafe { WaitNamedPipeW(PCWSTR(pipe_name.as_ptr()), 150) };

    let mut output = vec![0u8; 64 * 1024];
    let mut read = 0u32;
    unsafe {
        CallNamedPipeW(
            PCWSTR(pipe_name.as_ptr()),
            Some(payload.as_ptr().cast()),
            payload.len() as u32,
            Some(output.as_mut_ptr().cast()),
            output.len() as u32,
            &mut read,
            250,
        )
    }
    .ok()
    .map_err(|_| "named pipe request failed".to_string())?;

    let response = serde_json::from_slice::<SettingsIpcResponseEnvelope>(&output[..read as usize])
        .map_err(|e| e.to_string())?;

    if response.protocol_version != TAURI_SETTINGS_PROTOCOL_VERSION {
        return Err("named pipe protocol mismatch".to_string());
    }
    if response.request_id != request_id {
        return Err("named pipe request id mismatch".to_string());
    }
    Ok(response.response)
}

/// Try the host pipe; on failure (and fake mode enabled) fall back to fake state.
fn call_or_fake<T>(
    command: SettingsIpcCommand,
    extract: impl FnOnce(SettingsIpcResponse) -> Result<T, String>,
    fake_fallback: impl FnOnce(&mut FakeBackendState) -> Result<T, String>,
) -> Result<T, String> {
    match call_pipe(command) {
        Ok(response) => extract(response),
        Err(pipe_error) if !fake_backend_enabled() => Err(format!(
            "{}; fake backend disabled unless {}=1",
            pipe_error, FAKE_BACKEND_ENV
        )),
        Err(_) => {
            let mut guard = state()
                .lock()
                .map_err(|_| "fake state lock poisoned".to_string())?;
            fake_fallback(&mut guard)
        }
    }
}

#[tauri::command]
fn bootstrap_window() -> Result<SettingsBootstrapDto, String> {
    if let (
        Ok(SettingsIpcResponse::GetSnapshot { snapshot }),
        Ok(SettingsIpcResponse::GetSettings { settings }),
    ) = (
        call_pipe(SettingsIpcCommand::GetSnapshot),
        call_pipe(SettingsIpcCommand::GetSettings),
    ) {
        return Ok(SettingsBootstrapDto {
            protocol_version: TAURI_SETTINGS_PROTOCOL_VERSION.to_string(),
            transport: SettingsTransportDto {
                kind: "named_pipe".to_string(),
                endpoint: TAURI_SETTINGS_PIPE_NAME.to_string(),
            },
            fake_mode: false,
            pages: vec![
                "overview".to_string(),
                "general".to_string(),
                "monitoring".to_string(),
                "appearance".to_string(),
                "diagnostics".to_string(),
                "about".to_string(),
            ],
            about: about_metadata(),
            default_widget_palette: default_widget_palette(),
            snapshot,
            settings,
        });
    }

    if !fake_backend_enabled() {
        return Err(format!(
            "host settings pipe unavailable; fake backend disabled unless {}=1",
            FAKE_BACKEND_ENV
        ));
    }

    let guard = state()
        .lock()
        .map_err(|_| "fake state lock poisoned".to_string())?;
    Ok(SettingsBootstrapDto {
        protocol_version: TAURI_SETTINGS_PROTOCOL_VERSION.to_string(),
        transport: SettingsTransportDto {
            kind: "named_pipe".to_string(),
            endpoint: TAURI_SETTINGS_PIPE_NAME.to_string(),
        },
        fake_mode: true,
        pages: vec![
            "overview".to_string(),
            "general".to_string(),
            "monitoring".to_string(),
            "appearance".to_string(),
            "diagnostics".to_string(),
            "about".to_string(),
        ],
        about: about_metadata(),
        default_widget_palette: default_widget_palette(),
        snapshot: guard.snapshot.clone(),
        settings: guard.settings.clone(),
    })
}

#[tauri::command]
fn get_snapshot() -> Result<StatusSnapshotView, String> {
    call_or_fake(
        SettingsIpcCommand::GetSnapshot,
        |response| match response {
            SettingsIpcResponse::GetSnapshot { snapshot } => Ok(snapshot),
            SettingsIpcResponse::Error { message } => Err(message),
            _ => Err("unexpected get_snapshot response".to_string()),
        },
        |guard| Ok(guard.snapshot.clone()),
    )
}

#[tauri::command]
fn get_settings() -> Result<AppConfig, String> {
    call_or_fake(
        SettingsIpcCommand::GetSettings,
        |response| match response {
            SettingsIpcResponse::GetSettings { settings } => Ok(settings),
            SettingsIpcResponse::Error { message } => Err(message),
            _ => Err("unexpected get_settings response".to_string()),
        },
        |guard| Ok(guard.settings.clone()),
    )
}

#[tauri::command]
fn save_settings(settings: AppConfig) -> Result<SettingsSaveResultDto, String> {
    call_or_fake(
        SettingsIpcCommand::SaveSettings {
            settings: settings.clone(),
        },
        |response| match response {
            SettingsIpcResponse::SaveSettings { result } => Ok(result),
            SettingsIpcResponse::Error { message } => Err(message),
            _ => Err("unexpected save_settings response".to_string()),
        },
        |guard| {
            let previous = std::mem::replace(&mut guard.settings, settings);
            let applied_keys = changed_keys(&previous, &guard.settings);
            Ok(SettingsSaveResultDto {
                settings: guard.settings.clone(),
                applied_keys,
            })
        },
    )
}

#[tauri::command]
fn request_refresh() -> Result<SettingsRefreshResultDto, String> {
    call_or_fake(
        SettingsIpcCommand::RequestRefresh,
        |response| match response {
            SettingsIpcResponse::RequestRefresh { result } => Ok(result),
            SettingsIpcResponse::Error { message } => Err(message),
            _ => Err("unexpected request_refresh response".to_string()),
        },
        |guard| {
            guard.snapshot.last_detection_refresh_at = Some(1_783_066_111_000);
            Ok(SettingsRefreshResultDto { accepted: true })
        },
    )
}

#[tauri::command]
fn notify_settings_applied(applied_keys: Vec<String>) -> Result<(), String> {
    call_or_fake(
        SettingsIpcCommand::NotifySettingsApplied {
            applied_keys: applied_keys.clone(),
        },
        |response| match response {
            SettingsIpcResponse::NotifySettingsApplied { acknowledged } if acknowledged => Ok(()),
            SettingsIpcResponse::Error { message } => Err(message),
            _ => Err("unexpected notify_settings_applied response".to_string()),
        },
        |_guard| Ok(()),
    )
}

#[tauri::command]
fn get_hook_status() -> Result<HookStatusDto, String> {
    call_or_fake(
        SettingsIpcCommand::GetHookStatus,
        |response| match response {
            SettingsIpcResponse::GetHookStatus { status } => Ok(status),
            SettingsIpcResponse::Error { message } => Err(message),
            _ => Err("unexpected get_hook_status response".to_string()),
        },
        |_guard| {
            Ok(HookStatusDto {
                codex: shared_core::tauri_ipc::HookStatus::NotInstalled,
                claude: shared_core::tauri_ipc::HookStatus::NotInstalled,
            })
        },
    )
}

#[tauri::command]
fn get_hook_diagnostics() -> Result<HookDiagnosticsDto, String> {
    call_or_fake(
        SettingsIpcCommand::GetHookDiagnostics,
        |response| match response {
            SettingsIpcResponse::GetHookDiagnostics { diagnostics } => Ok(diagnostics),
            SettingsIpcResponse::Error { message } => Err(message),
            _ => Err("unexpected get_hook_diagnostics response".to_string()),
        },
        |_guard| Ok(fake_hook_diagnostics()),
    )
}

#[tauri::command]
fn get_runtime_log_diagnostics() -> Result<RuntimeLogDiagnosticsDto, String> {
    call_or_fake(
        SettingsIpcCommand::GetRuntimeLogDiagnostics,
        |response| match response {
            SettingsIpcResponse::GetRuntimeLogDiagnostics { diagnostics } => Ok(diagnostics),
            SettingsIpcResponse::Error { message } => Err(message),
            _ => Err("unexpected get_runtime_log_diagnostics response".to_string()),
        },
        |_guard| Ok(fake_runtime_log_diagnostics()),
    )
}

#[tauri::command]
fn open_runtime_log_directory() -> Result<String, String> {
    call_or_fake(
        SettingsIpcCommand::OpenRuntimeLogDirectory,
        |response| match response {
            SettingsIpcResponse::OpenRuntimeLogDirectory { directory_path } => Ok(directory_path),
            SettingsIpcResponse::Error { message } => Err(message),
            _ => Err("unexpected open_runtime_log_directory response".to_string()),
        },
        |_guard| Ok(fake_runtime_log_diagnostics().directory_path),
    )
}

#[tauri::command]
fn install_codex_hooks() -> Result<String, String> {
    call_or_fake(
        SettingsIpcCommand::InstallCodexHooks,
        |response| match response {
            SettingsIpcResponse::InstallCodexHooks { success, message } => {
                if success {
                    Ok(message)
                } else {
                    Err(message)
                }
            }
            SettingsIpcResponse::Error { message } => Err(message),
            _ => Err("unexpected install_codex_hooks response".to_string()),
        },
        |_guard| Err("cannot install hooks in fake backend mode".to_string()),
    )
}

#[tauri::command]
fn install_claude_hooks() -> Result<String, String> {
    call_or_fake(
        SettingsIpcCommand::InstallClaudeHooks,
        |response| match response {
            SettingsIpcResponse::InstallClaudeHooks { success, message } => {
                if success {
                    Ok(message)
                } else {
                    Err(message)
                }
            }
            SettingsIpcResponse::Error { message } => Err(message),
            _ => Err("unexpected install_claude_hooks response".to_string()),
        },
        |_guard| Ok("fake Claude hooks deployed".to_string()),
    )
}

#[tauri::command]
fn uninstall_codex_hooks() -> Result<String, String> {
    call_or_fake(
        SettingsIpcCommand::UninstallCodexHooks,
        |response| match response {
            SettingsIpcResponse::UninstallCodexHooks { success, message } => {
                if success {
                    Ok(message)
                } else {
                    Err(message)
                }
            }
            SettingsIpcResponse::Error { message } => Err(message),
            _ => Err("unexpected uninstall_codex_hooks response".to_string()),
        },
        |_guard| Err("cannot uninstall hooks in fake backend mode".to_string()),
    )
}

#[tauri::command]
fn uninstall_claude_hooks() -> Result<String, String> {
    call_or_fake(
        SettingsIpcCommand::UninstallClaudeHooks,
        |response| match response {
            SettingsIpcResponse::UninstallClaudeHooks { success, message } => {
                if success {
                    Ok(message)
                } else {
                    Err(message)
                }
            }
            SettingsIpcResponse::Error { message } => Err(message),
            _ => Err("unexpected uninstall_claude_hooks response".to_string()),
        },
        |_guard| Err("cannot uninstall hooks in fake backend mode".to_string()),
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            bootstrap_window,
            get_snapshot,
            get_settings,
            save_settings,
            request_refresh,
            notify_settings_applied,
            get_hook_status,
            get_hook_diagnostics,
            get_runtime_log_diagnostics,
            open_runtime_log_directory,
            install_codex_hooks,
            install_claude_hooks,
            uninstall_codex_hooks,
            uninstall_claude_hooks
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri settings");
}
