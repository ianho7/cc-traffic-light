import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
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

export function requestRefresh(): Promise<SettingsRefreshResultDto> {
  return invoke("request_refresh");
}

export function notifySettingsApplied(appliedKeys: string[]): Promise<void> {
  return invoke("notify_settings_applied", { appliedKeys });
}
