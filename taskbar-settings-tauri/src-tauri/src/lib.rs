use std::{
    env,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex, OnceLock,
    },
};

use serde::Serialize;
use shared_core::{
    app_config::{config_file_path, default_widget_palette, AppConfig},
    settings_service::{SourceStatusView, StatusSnapshotView},
    tauri_ipc::{
        SettingsAboutMetadataDto, SettingsBootstrapDto, SettingsIpcCommand,
        SettingsIpcEnvelope, SettingsIpcResponse, SettingsIpcResponseEnvelope,
        SettingsRefreshResultDto, SettingsSaveResultDto, SettingsTransportDto,
        TAURI_SETTINGS_PIPE_NAME, TAURI_SETTINGS_PROTOCOL_VERSION,
    },
};
use windows::{core::PCWSTR, Win32::System::Pipes::{CallNamedPipeW, WaitNamedPipeW}};

#[derive(Clone)]
struct FakeBackendState {
    settings: AppConfig,
    snapshot: StatusSnapshotView,
}

static FAKE_STATE: OnceLock<Mutex<FakeBackendState>> = OnceLock::new();
static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);
const FAKE_BACKEND_ENV: &str = "CC_TRAFFIC_LIGHT_TAURI_FAKE_BACKEND";

#[derive(Serialize)]
struct TauriCommandError {
    message: String,
}

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

fn error(message: impl Into<String>) -> TauriCommandError {
    TauriCommandError {
        message: message.into(),
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

fn call_pipe(command: SettingsIpcCommand) -> Result<SettingsIpcResponse, TauriCommandError> {
    let envelope = SettingsIpcEnvelope {
        protocol_version: TAURI_SETTINGS_PROTOCOL_VERSION.to_string(),
        request_id: next_request_id(),
        command,
    };
    let request_id = envelope.request_id.clone();
    let payload = serde_json::to_vec(&envelope)
        .map_err(|serialize_error| error(serialize_error.to_string()))?;
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
    .map_err(|_| error("named pipe request failed"))?;

    let response = serde_json::from_slice::<SettingsIpcResponseEnvelope>(&output[..read as usize])
        .map_err(|parse_error| error(parse_error.to_string()))?;

    if response.protocol_version != TAURI_SETTINGS_PROTOCOL_VERSION {
        return Err(error("named pipe protocol mismatch"));
    }
    if response.request_id != request_id {
        return Err(error("named pipe request id mismatch"));
    }
    Ok(response.response)
}

fn pipe_or_fake_error(pipe_error: TauriCommandError) -> TauriCommandError {
    if fake_backend_enabled() {
        return pipe_error;
    }

    error(format!(
        "{}; fake backend disabled unless {}=1",
        pipe_error.message, FAKE_BACKEND_ENV
    ))
}

#[tauri::command]
fn bootstrap_window() -> Result<SettingsBootstrapDto, TauriCommandError> {
    if let (Ok(SettingsIpcResponse::GetSnapshot { snapshot }), Ok(SettingsIpcResponse::GetSettings { settings })) =
        (call_pipe(SettingsIpcCommand::GetSnapshot), call_pipe(SettingsIpcCommand::GetSettings))
    {
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
        return Err(error(format!(
            "host settings pipe unavailable; fake backend disabled unless {}=1",
            FAKE_BACKEND_ENV
        )));
    }

    let guard = state().lock().map_err(|_| error("fake state lock poisoned"))?;
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
fn get_snapshot() -> Result<StatusSnapshotView, TauriCommandError> {
    match call_pipe(SettingsIpcCommand::GetSnapshot) {
        Ok(SettingsIpcResponse::GetSnapshot { snapshot }) => return Ok(snapshot),
        Ok(SettingsIpcResponse::Error { message }) => return Err(error(message)),
        Ok(_) => return Err(error("unexpected get_snapshot response")),
        Err(pipe_error) if !fake_backend_enabled() => return Err(pipe_or_fake_error(pipe_error)),
        Err(_) => {}
    }

    let guard = state().lock().map_err(|_| error("fake state lock poisoned"))?;
    Ok(guard.snapshot.clone())
}

#[tauri::command]
fn get_settings() -> Result<AppConfig, TauriCommandError> {
    match call_pipe(SettingsIpcCommand::GetSettings) {
        Ok(SettingsIpcResponse::GetSettings { settings }) => return Ok(settings),
        Ok(SettingsIpcResponse::Error { message }) => return Err(error(message)),
        Ok(_) => return Err(error("unexpected get_settings response")),
        Err(pipe_error) if !fake_backend_enabled() => return Err(pipe_or_fake_error(pipe_error)),
        Err(_) => {}
    }

    let guard = state().lock().map_err(|_| error("fake state lock poisoned"))?;
    Ok(guard.settings.clone())
}

#[tauri::command]
fn save_settings(settings: AppConfig) -> Result<SettingsSaveResultDto, TauriCommandError> {
    match call_pipe(SettingsIpcCommand::SaveSettings {
        settings: settings.clone(),
    }) {
        Ok(SettingsIpcResponse::SaveSettings { result }) => return Ok(result),
        Ok(SettingsIpcResponse::Error { message }) => return Err(error(message)),
        Ok(_) => return Err(error("unexpected save_settings response")),
        Err(pipe_error) if !fake_backend_enabled() => return Err(pipe_or_fake_error(pipe_error)),
        Err(_) => {}
    }

    let mut guard = state().lock().map_err(|_| error("fake state lock poisoned"))?;
    let applied_keys = vec![
        "general.autostart_enabled".to_string(),
        "general.start_minimized_to_tray".to_string(),
        "general.close_to_tray".to_string(),
        "monitoring.codex_enabled".to_string(),
        "monitoring.claude_enabled".to_string(),
        "appearance.ui_theme".to_string(),
        "appearance.indicator_style".to_string(),
        "appearance.widget_size".to_string(),
        "appearance.show_labels".to_string(),
        "appearance.reduced_motion".to_string(),
        "widget_visual.placement".to_string(),
        "widget_visual.palette.green".to_string(),
        "widget_visual.palette.yellow".to_string(),
        "widget_visual.palette.red".to_string(),
        "widget_visual.palette.inactive_brightness_percent".to_string(),
        "localization.language".to_string(),
        "diagnostics.last_opened_page".to_string()
    ];
    guard.settings = settings.clone();
    Ok(SettingsSaveResultDto {
        settings,
        applied_keys,
    })
}

#[tauri::command]
fn request_refresh() -> Result<SettingsRefreshResultDto, TauriCommandError> {
    match call_pipe(SettingsIpcCommand::RequestRefresh) {
        Ok(SettingsIpcResponse::RequestRefresh { result }) => return Ok(result),
        Ok(SettingsIpcResponse::Error { message }) => return Err(error(message)),
        Ok(_) => return Err(error("unexpected request_refresh response")),
        Err(pipe_error) if !fake_backend_enabled() => return Err(pipe_or_fake_error(pipe_error)),
        Err(_) => {}
    }

    let mut guard = state().lock().map_err(|_| error("fake state lock poisoned"))?;
    guard.snapshot.last_detection_refresh_at = Some(1_783_066_111_000);
    Ok(SettingsRefreshResultDto { accepted: true })
}

#[tauri::command]
fn notify_settings_applied(
    applied_keys: Vec<String>,
) -> Result<(), TauriCommandError> {
    match call_pipe(SettingsIpcCommand::NotifySettingsApplied {
        applied_keys: applied_keys.clone(),
    }) {
        Ok(SettingsIpcResponse::NotifySettingsApplied { acknowledged }) if acknowledged => {
            return Ok(());
        }
        Ok(SettingsIpcResponse::Error { message }) => return Err(error(message)),
        Ok(_) => return Err(error("unexpected notify_settings_applied response")),
        Err(pipe_error) if !fake_backend_enabled() => return Err(pipe_or_fake_error(pipe_error)),
        Err(_) => {}
    }

    let _guard = state().lock().map_err(|_| error("fake state lock poisoned"))?;
    Ok(())
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
            notify_settings_applied
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri settings");
}
