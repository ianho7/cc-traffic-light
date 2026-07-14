import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  HookDiagnosticsDto,
  HookStatusDto,
  MaterialGroupAvailability,
  RuntimeLogDiagnosticsDto,
  SettingsBootstrapDto,
  SettingsRefreshResultDto,
  SettingsSaveResultDto,
  StatusSnapshotView
} from "../types";

export function bootstrapWindow(): Promise<SettingsBootstrapDto> {
  return invoke("bootstrap_window");
}

export function getSnapshot(): Promise<StatusSnapshotView> {
  return invoke("get_snapshot");
}

export function getSettings(): Promise<AppConfig> {
  return invoke("get_settings");
}

export function saveSettings(settings: AppConfig): Promise<SettingsSaveResultDto> {
  return invoke("save_settings", { settings });
}

export function saveMaterialGroup(
  settings: AppConfig,
  groupId: string,
  name: string,
  greenPng: number[],
  yellowPng: number[],
  redPng: number[]
): Promise<SettingsSaveResultDto> {
  return invoke("save_material_group", {
    settings,
    groupId,
    name,
    greenPng,
    yellowPng,
    redPng
  });
}

export function deleteMaterialGroup(settings: AppConfig, groupId: string): Promise<SettingsSaveResultDto> {
  return invoke("delete_material_group", { settings, groupId });
}

export function getMaterialGroupAvailability(settings: AppConfig): Promise<MaterialGroupAvailability[]> {
  return invoke("get_material_group_availability", { settings });
}

export function requestRefresh(): Promise<SettingsRefreshResultDto> {
  return invoke("request_refresh");
}

export function notifySettingsApplied(appliedKeys: string[]): Promise<void> {
  return invoke("notify_settings_applied", { appliedKeys });
}

export function getHookStatus(): Promise<HookStatusDto> {
  return invoke("get_hook_status");
}

export function getHookDiagnostics(): Promise<HookDiagnosticsDto> {
  return invoke("get_hook_diagnostics");
}

export function getRuntimeLogDiagnostics(): Promise<RuntimeLogDiagnosticsDto> {
  return invoke("get_runtime_log_diagnostics");
}

export function openRuntimeLogDirectory(): Promise<string> {
  return invoke("open_runtime_log_directory");
}

export function installCodexHooks(): Promise<string> {
  return invoke("install_codex_hooks");
}

export function installClaudeHooks(): Promise<string> {
  return invoke("install_claude_hooks");
}

export function uninstallCodexHooks(): Promise<string> {
  return invoke("uninstall_codex_hooks");
}

export function uninstallClaudeHooks(): Promise<string> {
  return invoke("uninstall_claude_hooks");
}
