use std::{
    collections::HashMap,
    env, fs,
    path::PathBuf,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use shared_core::{
    app_config::{
        AppConfig, MATERIAL_DISPLAY_SIZE_MAX_PX, MATERIAL_DISPLAY_SIZE_MIN_PX, MaterialGroup,
        changed_keys, config_dir_path, config_file_path, default_widget_palette,
    },
    settings_service::{SourceStatusView, StatusSnapshotView},
    tauri_ipc::{
        HookDiagnosticPathsDto, HookDiagnosticsDto, HookStatusDto, RuntimeLogDiagnosticsDto,
        SettingsAboutMetadataDto, SettingsBootstrapDto, SettingsIpcCommand, SettingsIpcEnvelope,
        SettingsIpcResponse, SettingsIpcResponseEnvelope, SettingsRefreshResultDto,
        SettingsSaveResultDto, SettingsTransportDto, TAURI_SETTINGS_PIPE_NAME,
        TAURI_SETTINGS_PROTOCOL_VERSION,
    },
};
use tauri::Manager;
use windows::{
    Win32::System::Pipes::{CallNamedPipeW, WaitNamedPipeW},
    core::PCWSTR,
};

struct WindowBehaviorState {
    close_to_tray: Mutex<bool>,
}

impl Default for WindowBehaviorState {
    fn default() -> Self {
        let close_to_tray = env::var(CLOSE_TO_TRAY_ENV)
            .ok()
            .and_then(|value| match value.as_str() {
                "1" | "true" | "TRUE" | "True" => Some(true),
                "0" | "false" | "FALSE" | "False" => Some(false),
                _ => None,
            })
            .unwrap_or_else(|| {
                shared_core::app_config::load_config_diagnostic()
                    .config
                    .general
                    .close_to_tray
            });
        Self {
            // Initialize before the frontend's bootstrap command runs, so a
            // close request immediately after startup still honors the saved
            // preference. bootstrap_window remains the live host sync point.
            close_to_tray: Mutex::new(close_to_tray),
        }
    }
}

impl WindowBehaviorState {
    fn sync_from_settings(&self, settings: &AppConfig) {
        if let Ok(mut close_to_tray) = self.close_to_tray.lock() {
            *close_to_tray = settings.general.close_to_tray;
        }
    }

    fn should_close_to_tray(&self) -> bool {
        self.close_to_tray
            .lock()
            .map(|value| *value)
            .unwrap_or(true)
    }
}

#[derive(Clone)]
struct FakeBackendState {
    settings: AppConfig,
    snapshot: StatusSnapshotView,
}

static FAKE_STATE: OnceLock<Mutex<FakeBackendState>> = OnceLock::new();
static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);
const FAKE_BACKEND_ENV: &str = "CC_TRAFFIC_LIGHT_TAURI_FAKE_BACKEND";
const CLOSE_TO_TRAY_ENV: &str = "CC_TRAFFIC_LIGHT_CLOSE_TO_TRAY";
const MATERIAL_IMAGE_SIZE: u32 = 64;
const MATERIAL_MAX_BYTES: usize = 256 * 1024;

#[derive(serde::Serialize)]
struct MaterialGroupPreviewDto {
    green: String,
    yellow: String,
    red: String,
}

struct MaterialFileSwap {
    final_path: PathBuf,
    backup_path: Option<PathBuf>,
}

struct MaterialWriteTransaction {
    swaps: Vec<MaterialFileSwap>,
}

impl MaterialWriteTransaction {
    fn commit(self) {
        for swap in self.swaps {
            if let Some(backup_path) = swap.backup_path {
                let _ = fs::remove_file(backup_path);
            }
        }
    }

    fn rollback(self) {
        for swap in self.swaps {
            let _ = fs::remove_file(&swap.final_path);
            if let Some(backup_path) = swap.backup_path {
                let _ = fs::rename(backup_path, swap.final_path);
            }
        }
    }
}

struct MaterialDeleteTransaction {
    original_path: PathBuf,
    staged_path: Option<PathBuf>,
}

impl MaterialDeleteTransaction {
    fn commit(self) {
        if let Some(staged_path) = self.staged_path {
            let _ = fs::remove_dir_all(staged_path);
        }
    }

    fn rollback(self) {
        if let Some(staged_path) = self.staged_path {
            let _ = fs::rename(staged_path, self.original_path);
        }
    }
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
        runtime_log_path: r"C:\Users\fake\AppData\Local\CC Traffic Light\logs\runtime.log"
            .to_string(),
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
fn bootstrap_window(
    window_behavior: tauri::State<'_, WindowBehaviorState>,
) -> Result<SettingsBootstrapDto, String> {
    if let (
        Ok(SettingsIpcResponse::GetSnapshot { snapshot }),
        Ok(SettingsIpcResponse::GetSettings { settings }),
    ) = (
        call_pipe(SettingsIpcCommand::GetSnapshot),
        call_pipe(SettingsIpcCommand::GetSettings),
    ) {
        let bootstrap = SettingsBootstrapDto {
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
            material_display_size_min_px: MATERIAL_DISPLAY_SIZE_MIN_PX,
            material_display_size_max_px: MATERIAL_DISPLAY_SIZE_MAX_PX,
            snapshot,
            settings,
        };
        window_behavior.sync_from_settings(&bootstrap.settings);
        return Ok(bootstrap);
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
    let bootstrap = SettingsBootstrapDto {
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
        material_display_size_min_px: MATERIAL_DISPLAY_SIZE_MIN_PX,
        material_display_size_max_px: MATERIAL_DISPLAY_SIZE_MAX_PX,
        snapshot: guard.snapshot.clone(),
        settings: guard.settings.clone(),
    };
    window_behavior.sync_from_settings(&bootstrap.settings);
    Ok(bootstrap)
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
fn save_settings(
    settings: AppConfig,
    window_behavior: tauri::State<'_, WindowBehaviorState>,
) -> Result<SettingsSaveResultDto, String> {
    let result = save_settings_inner(settings)?;
    window_behavior.sync_from_settings(&result.settings);
    Ok(result)
}

fn save_settings_inner(settings: AppConfig) -> Result<SettingsSaveResultDto, String> {
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
fn save_material_group(
    settings: AppConfig,
    group_id: String,
    name: String,
    green_png: Vec<u8>,
    yellow_png: Vec<u8>,
    red_png: Vec<u8>,
) -> Result<SettingsSaveResultDto, String> {
    let group = build_material_group(&group_id, &name)?;
    let transaction = stage_material_group_write(&group, [&green_png, &yellow_png, &red_png])?;
    let mut next = settings;
    if let Some(existing) = next
        .widget_visual
        .material_groups
        .iter_mut()
        .find(|existing| existing.id == group.id)
    {
        *existing = group;
    } else {
        next.widget_visual.material_groups.push(group);
    }

    match save_settings_inner(next) {
        Ok(result) => {
            transaction.commit();
            Ok(result)
        }
        Err(error) => {
            transaction.rollback();
            Err(error)
        }
    }
}

#[tauri::command]
fn delete_material_group(
    settings: AppConfig,
    group_id: String,
) -> Result<SettingsSaveResultDto, String> {
    if settings.widget_visual.codex_material_group_id.as_deref() == Some(group_id.as_str())
        || settings.widget_visual.claude_material_group_id.as_deref() == Some(group_id.as_str())
    {
        return Err("material group is currently applied to an agent".to_string());
    }
    if !settings
        .widget_visual
        .material_groups
        .iter()
        .any(|group| group.id == group_id)
    {
        return Err("material group does not exist".to_string());
    }

    let transaction = stage_material_group_delete(&group_id)?;
    let mut next = settings;
    next.widget_visual
        .material_groups
        .retain(|group| group.id != group_id);

    match save_settings_inner(next) {
        Ok(result) => {
            transaction.commit();
            Ok(result)
        }
        Err(error) => {
            transaction.rollback();
            Err(error)
        }
    }
}

#[derive(serde::Serialize)]
struct MaterialGroupAvailability {
    group_id: String,
    available: bool,
}

#[tauri::command]
fn get_material_group_availability(settings: AppConfig) -> Vec<MaterialGroupAvailability> {
    settings
        .widget_visual
        .material_groups
        .iter()
        .map(|group| MaterialGroupAvailability {
            group_id: group.id.clone(),
            available: [
                group.green_path.as_str(),
                group.yellow_path.as_str(),
                group.red_path.as_str(),
            ]
            .into_iter()
            .all(|path| {
                fs::read(path)
                    .ok()
                    .and_then(|bytes| validate_material_png(&bytes).ok())
                    .is_some()
            }),
        })
        .collect()
}

#[tauri::command]
fn get_material_group_previews(
    settings: AppConfig,
) -> Result<HashMap<String, MaterialGroupPreviewDto>, String> {
    settings
        .widget_visual
        .material_groups
        .into_iter()
        .map(|group| {
            let green = material_preview_data_url(&group.green_path)?;
            let yellow = material_preview_data_url(&group.yellow_path)?;
            let red = material_preview_data_url(&group.red_path)?;
            Ok((group.id, MaterialGroupPreviewDto { green, yellow, red }))
        })
        .collect()
}

fn material_preview_data_url(path: &str) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read material preview: {error}"))?;
    validate_material_png(&bytes)?;
    Ok(format!("data:image/png;base64,{}", BASE64.encode(bytes)))
}

fn build_material_group(group_id: &str, name: &str) -> Result<MaterialGroup, String> {
    if !is_safe_material_group_id(group_id) {
        return Err("material group id must contain only letters, numbers, '-' or '_'".to_string());
    }
    if name.trim().is_empty() {
        return Err("material group name cannot be empty".to_string());
    }
    let directory = material_group_directory(group_id);
    Ok(MaterialGroup {
        id: group_id.to_string(),
        name: name.trim().to_string(),
        green_path: directory.join("green.png").display().to_string(),
        yellow_path: directory.join("yellow.png").display().to_string(),
        red_path: directory.join("red.png").display().to_string(),
    })
}

fn is_safe_material_group_id(group_id: &str) -> bool {
    !group_id.is_empty()
        && group_id.len() <= 64
        && group_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn material_assets_directory() -> PathBuf {
    config_dir_path().join("assets")
}

fn material_group_directory(group_id: &str) -> PathBuf {
    material_assets_directory().join(group_id)
}

fn stage_material_group_write(
    group: &MaterialGroup,
    images: [&[u8]; 3],
) -> Result<MaterialWriteTransaction, String> {
    for image in images {
        validate_material_png(image)?;
    }

    let directory = material_group_directory(&group.id);
    fs::create_dir_all(&directory).map_err(|error| {
        format!(
            "cannot create material directory {}: {error}",
            directory.display()
        )
    })?;
    let request = next_request_id();
    let paths = [
        (directory.join("green.png"), images[0]),
        (directory.join("yellow.png"), images[1]),
        (directory.join("red.png"), images[2]),
    ];
    let mut temporary_paths = Vec::with_capacity(paths.len());
    for (final_path, image) in &paths {
        let temporary_path = final_path.with_extension(format!("png.{request}.tmp"));
        if let Err(error) = fs::write(&temporary_path, image) {
            for path in temporary_paths {
                let _ = fs::remove_file(path);
            }
            return Err(format!(
                "cannot stage material {}: {error}",
                final_path.display()
            ));
        }
        temporary_paths.push(temporary_path);
    }

    let mut swaps = Vec::with_capacity(paths.len());
    for ((final_path, _), temporary_path) in paths.iter().zip(temporary_paths.iter()) {
        let backup_path = if final_path.exists() {
            let backup_path = final_path.with_extension(format!("png.{request}.bak"));
            if let Err(error) = fs::rename(final_path, &backup_path) {
                let transaction = MaterialWriteTransaction { swaps };
                transaction.rollback();
                for path in &temporary_paths {
                    let _ = fs::remove_file(path);
                }
                return Err(format!(
                    "cannot back up material {}: {error}",
                    final_path.display()
                ));
            }
            Some(backup_path)
        } else {
            None
        };
        swaps.push(MaterialFileSwap {
            final_path: final_path.clone(),
            backup_path,
        });

        if let Err(error) = fs::rename(temporary_path, final_path) {
            let transaction = MaterialWriteTransaction { swaps };
            transaction.rollback();
            for path in &temporary_paths {
                let _ = fs::remove_file(path);
            }
            return Err(format!(
                "cannot promote material {}: {error}",
                final_path.display()
            ));
        }
    }

    Ok(MaterialWriteTransaction { swaps })
}

fn stage_material_group_delete(group_id: &str) -> Result<MaterialDeleteTransaction, String> {
    if !is_safe_material_group_id(group_id) {
        return Err("material group id is invalid".to_string());
    }
    let original_path = material_group_directory(group_id);
    if !original_path.exists() {
        return Ok(MaterialDeleteTransaction {
            original_path,
            staged_path: None,
        });
    }
    let staged_path =
        original_path.with_file_name(format!("{group_id}.{}.delete", next_request_id()));
    fs::rename(&original_path, &staged_path).map_err(|error| {
        format!(
            "cannot stage material group deletion {}: {error}",
            original_path.display()
        )
    })?;
    Ok(MaterialDeleteTransaction {
        original_path,
        staged_path: Some(staged_path),
    })
}

fn validate_material_png(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > MATERIAL_MAX_BYTES {
        return Err("material PNG size is invalid".to_string());
    }
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|error| format!("material must be a PNG: {error}"))?;
    if image.width() != MATERIAL_IMAGE_SIZE || image.height() != MATERIAL_IMAGE_SIZE {
        return Err(format!(
            "material PNG must be {MATERIAL_IMAGE_SIZE}x{MATERIAL_IMAGE_SIZE}"
        ));
    }
    Ok(())
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
        .plugin(tauri_plugin_opener::init())
        .manage(WindowBehaviorState::default())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.state::<WindowBehaviorState>().should_close_to_tray() {
                    api.prevent_close();
                    let _ = window.hide();
                } else {
                    // Closing the last Tauri window does not necessarily end
                    // the process on every Windows runtime configuration.
                    // This application owns a single settings window, so OFF
                    // has an explicit process-exit contract.
                    std::process::exit(0);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap_window,
            get_snapshot,
            get_settings,
            save_settings,
            save_material_group,
            delete_material_group,
            get_material_group_availability,
            get_material_group_previews,
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

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ColorType, ImageBuffer, ImageEncoder, RgbaImage, codecs::png::PngEncoder};

    fn valid_material_png(width: u32, height: u32) -> Vec<u8> {
        let image: RgbaImage = ImageBuffer::from_pixel(width, height, image::Rgba([1, 2, 3, 255]));
        let mut bytes = Vec::new();
        PngEncoder::new(&mut bytes)
            .write_image(image.as_raw(), width, height, ColorType::Rgba8.into())
            .expect("PNG should encode");
        bytes
    }

    #[test]
    fn material_png_validation_requires_fixed_size_png() {
        assert!(validate_material_png(&valid_material_png(64, 64)).is_ok());
        assert!(validate_material_png(&valid_material_png(63, 64)).is_err());
        assert!(validate_material_png(b"not a PNG").is_err());
    }

    #[test]
    fn material_group_ids_are_path_safe() {
        assert!(is_safe_material_group_id("retro_green-1"));
        assert!(!is_safe_material_group_id("../outside"));
        assert!(!is_safe_material_group_id(""));
    }

    #[test]
    fn material_write_rollback_restores_previous_asset() {
        let root = env::temp_dir().join(format!(
            "cc-traffic-light-material-rollback-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("test directory should exist");
        let final_path = root.join("green.png");
        let backup_path = root.join("green.png.backup");
        fs::write(&final_path, b"new").expect("new material should exist");
        fs::write(&backup_path, b"old").expect("backup material should exist");

        MaterialWriteTransaction {
            swaps: vec![MaterialFileSwap {
                final_path: final_path.clone(),
                backup_path: Some(backup_path),
            }],
        }
        .rollback();

        assert_eq!(
            fs::read(final_path).expect("old material should be restored"),
            b"old"
        );
        let _ = fs::remove_dir_all(root);
    }
}
