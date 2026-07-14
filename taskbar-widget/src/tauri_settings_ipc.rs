use std::{
    sync::{Mutex, OnceLock},
    thread,
};

use shared_core::{
    settings_service::StatusSnapshotView,
    tauri_ipc::{
        SettingsIpcCommand, SettingsIpcEnvelope, SettingsIpcResponse, SettingsIpcResponseEnvelope,
        SettingsRefreshResultDto, TAURI_SETTINGS_PIPE_NAME, TAURI_SETTINGS_PROTOCOL_VERSION,
    },
};
use windows::{
    Win32::{
        Foundation::{
            CloseHandle, ERROR_MORE_DATA, ERROR_PIPE_CONNECTED, HANDLE, INVALID_HANDLE_VALUE,
        },
        Storage::FileSystem::{FlushFileBuffers, PIPE_ACCESS_DUPLEX, ReadFile, WriteFile},
        System::Pipes::{
            ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_MESSAGE,
            PIPE_TYPE_MESSAGE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
        },
    },
    core::PCWSTR,
};

use crate::{settings_bridge, win32};

const PIPE_BUFFER_SIZE: u32 = 64 * 1024;
static IPC_SERVER_STARTED: OnceLock<()> = OnceLock::new();
static LAST_IPC_SERVER_ERROR: OnceLock<Mutex<Option<String>>> = OnceLock::new();

pub fn ensure_server_started() {
    let _ = IPC_SERVER_STARTED.get_or_init(|| {
        let _ = thread::Builder::new()
            .name("tauri-settings-ipc".to_string())
            .spawn(server_loop);
    });
}

fn server_loop() {
    loop {
        match serve_once() {
            Ok(()) => clear_last_server_error(),
            Err(error) => log_server_error_if_changed(error),
        }
    }
}

fn log_server_error_if_changed(error: String) {
    let lock = LAST_IPC_SERVER_ERROR.get_or_init(|| Mutex::new(None));
    let Ok(mut last_error) = lock.lock() else {
        win32::runtime_debug_log(&format!("[tauri-ipc] serve_once error={error}"));
        return;
    };
    if last_error.as_deref() == Some(error.as_str()) {
        return;
    }
    *last_error = Some(error.clone());
    win32::runtime_debug_log(&format!("[tauri-ipc] serve_once error={error}"));
}

fn clear_last_server_error() {
    if let Some(lock) = LAST_IPC_SERVER_ERROR.get()
        && let Ok(mut last_error) = lock.lock()
    {
        *last_error = None;
    }
}

fn serve_once() -> Result<(), String> {
    let pipe_name = win32::wide_null(TAURI_SETTINGS_PIPE_NAME);
    let pipe = unsafe {
        CreateNamedPipeW(
            PCWSTR(pipe_name.as_ptr()),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
            PIPE_UNLIMITED_INSTANCES,
            PIPE_BUFFER_SIZE,
            PIPE_BUFFER_SIZE,
            0,
            None,
        )
    };

    if pipe == INVALID_HANDLE_VALUE {
        return Err(format!(
            "CreateNamedPipeW failed last_error={}",
            win32::last_error_code()
        ));
    }

    let connected = unsafe { ConnectNamedPipe(pipe, None) }.is_ok();
    if !connected && win32::last_error_code() != ERROR_PIPE_CONNECTED.0 {
        unsafe {
            let _ = CloseHandle(pipe);
        }
        return Err(format!(
            "ConnectNamedPipe failed last_error={}",
            win32::last_error_code()
        ));
    }

    let request_bytes = read_pipe_message(pipe)?;
    let response = handle_request(&request_bytes);
    write_pipe_message(pipe, &response)?;

    unsafe {
        let _ = FlushFileBuffers(pipe);
        let _ = DisconnectNamedPipe(pipe);
        let _ = CloseHandle(pipe);
    }

    Ok(())
}

fn read_pipe_message(pipe: HANDLE) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();

    loop {
        let mut chunk = vec![0u8; 4096];
        let mut read = 0u32;
        let ok =
            unsafe { ReadFile(pipe, Some(chunk.as_mut_slice()), Some(&mut read), None) }.is_ok();

        if ok {
            if read == 0 {
                break;
            }
            output.extend_from_slice(&chunk[..read as usize]);
            if read < chunk.len() as u32 {
                break;
            }
            continue;
        }

        let last_error = win32::last_error_code();
        if last_error == ERROR_MORE_DATA.0 {
            output.extend_from_slice(&chunk[..read as usize]);
            continue;
        }

        return Err(format!("ReadFile failed last_error={last_error}"));
    }

    Ok(output)
}

fn write_pipe_message(pipe: HANDLE, response: &SettingsIpcResponseEnvelope) -> Result<(), String> {
    let payload = serde_json::to_vec(response).map_err(|error| error.to_string())?;
    let mut written = 0u32;
    unsafe { WriteFile(pipe, Some(payload.as_slice()), Some(&mut written), None) }
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn handle_request(request_bytes: &[u8]) -> SettingsIpcResponseEnvelope {
    let request = serde_json::from_slice::<SettingsIpcEnvelope>(request_bytes);
    match request {
        Ok(request) => {
            if request.protocol_version != TAURI_SETTINGS_PROTOCOL_VERSION {
                return response_error(
                    request.request_id,
                    format!(
                        "protocol mismatch expected={} got={}",
                        TAURI_SETTINGS_PROTOCOL_VERSION, request.protocol_version
                    ),
                );
            }

            let response = match request.command {
                SettingsIpcCommand::GetSnapshot => SettingsIpcResponse::GetSnapshot {
                    snapshot: StatusSnapshotView::from(settings_bridge::current_snapshot()),
                },
                SettingsIpcCommand::GetSettings => SettingsIpcResponse::GetSettings {
                    settings: settings_bridge::current_config(),
                },
                SettingsIpcCommand::SaveSettings { settings } => {
                    match settings_bridge::apply_full_settings(settings) {
                        Ok(result) => SettingsIpcResponse::SaveSettings { result },
                        Err(error) => {
                            return response_error(request.request_id, error);
                        }
                    }
                }
                SettingsIpcCommand::RequestRefresh => {
                    match settings_bridge::request_manual_refresh_command() {
                        Ok(_) => SettingsIpcResponse::RequestRefresh {
                            result: SettingsRefreshResultDto { accepted: true },
                        },
                        Err(error) => {
                            return response_error(request.request_id, error);
                        }
                    }
                }
                SettingsIpcCommand::NotifySettingsApplied { applied_keys } => {
                    settings_bridge::notify_settings_applied(&applied_keys);
                    SettingsIpcResponse::NotifySettingsApplied { acknowledged: true }
                }
                SettingsIpcCommand::GetHookStatus => {
                    let status = settings_bridge::detect_hook_status();
                    SettingsIpcResponse::GetHookStatus { status }
                }
                SettingsIpcCommand::GetHookDiagnostics => {
                    let diagnostics = settings_bridge::hook_diagnostics();
                    SettingsIpcResponse::GetHookDiagnostics { diagnostics }
                }
                SettingsIpcCommand::GetRuntimeLogDiagnostics => {
                    let diagnostics = settings_bridge::runtime_log_diagnostics();
                    SettingsIpcResponse::GetRuntimeLogDiagnostics { diagnostics }
                }
                SettingsIpcCommand::OpenRuntimeLogDirectory => {
                    match settings_bridge::open_runtime_log_directory() {
                        Ok(directory_path) => SettingsIpcResponse::OpenRuntimeLogDirectory {
                            directory_path,
                        },
                        Err(error) => return response_error(request.request_id, error),
                    }
                }
                SettingsIpcCommand::InstallCodexHooks => {
                    let (success, message) = settings_bridge::install_codex_hooks();
                    SettingsIpcResponse::InstallCodexHooks { success, message }
                }
                SettingsIpcCommand::InstallClaudeHooks => {
                    let (success, message) = settings_bridge::install_claude_hooks();
                    SettingsIpcResponse::InstallClaudeHooks { success, message }
                }
                SettingsIpcCommand::UninstallCodexHooks => {
                    let (success, message) = settings_bridge::uninstall_codex_hooks();
                    SettingsIpcResponse::UninstallCodexHooks { success, message }
                }
                SettingsIpcCommand::UninstallClaudeHooks => {
                    let (success, message) = settings_bridge::uninstall_claude_hooks();
                    SettingsIpcResponse::UninstallClaudeHooks { success, message }
                }
            };

            SettingsIpcResponseEnvelope {
                protocol_version: TAURI_SETTINGS_PROTOCOL_VERSION.to_string(),
                request_id: request.request_id,
                response,
            }
        }
        Err(error) => response_error("invalid-request".to_string(), error.to_string()),
    }
}

fn response_error(request_id: String, message: String) -> SettingsIpcResponseEnvelope {
    SettingsIpcResponseEnvelope {
        protocol_version: TAURI_SETTINGS_PROTOCOL_VERSION.to_string(),
        request_id,
        response: SettingsIpcResponse::Error { message },
    }
}
