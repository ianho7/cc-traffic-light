export type SettingsPageId =
  | "overview"
  | "general"
  | "monitoring"
  | "appearance"
  | "diagnostics"
  | "about";

export interface SourceStatusView {
  source_id: string;
  state: string;
  confidence: string;
  method: string;
  updated_at: number;
  message: string | null;
}

export interface StatusSnapshotView {
  widget_mount_state: string;
  overall_state: string;
  last_widget_attach_at: number | null;
  last_detection_refresh_at: number | null;
  last_error_summary: string | null;
  sources: Record<string, SourceStatusView>;
}

export interface AppConfig {
  schema_version: number;
  localization: {
    language: "follow_system" | "zh-CN" | "en";
  };
  general: {
    autostart_enabled: boolean;
    start_minimized_to_tray: boolean;
    close_to_tray: boolean;
  };
  monitoring: {
    codex_enabled: boolean;
    claude_enabled: boolean;
  };
  appearance: {
    ui_theme: "light" | "dark";
    indicator_style: "classic" | "minimal";
    widget_size: "compact" | "standard";
    show_labels: boolean;
    reduced_motion: boolean;
  };
  widget_visual: {
    placement: "left" | "right";
    palette: WidgetPaletteConfig;
    material_groups: MaterialGroup[];
    codex_material_group_id: string | null;
    claude_material_group_id: string | null;
  };
  diagnostics: {
    last_opened_page: SettingsPageId;
    last_manual_refresh_at: number | null;
    last_hook_notification_key: string | null;
  };
}

export interface WidgetPaletteConfig {
  green: string;
  yellow: string;
  red: string;
  inactive_brightness_percent: number;
}

export interface MaterialGroup {
  id: string;
  name: string;
  green_path: string;
  yellow_path: string;
  red_path: string;
}

export interface MaterialGroupAvailability {
  group_id: string;
  available: boolean;
}

export interface SettingsTransportDto {
  kind: string;
  endpoint: string;
}

export interface SettingsAboutMetadataDto {
  product_name: string;
  version: string;
  runtime_description: string;
  config_path: string;
}

export interface SettingsBootstrapDto {
  protocol_version: string;
  transport: SettingsTransportDto;
  fake_mode: boolean;
  pages: SettingsPageId[];
  about: SettingsAboutMetadataDto;
  default_widget_palette: WidgetPaletteConfig;
  snapshot: StatusSnapshotView;
  settings: AppConfig;
}

export interface SettingsSaveResultDto {
  settings: AppConfig;
  applied_keys: string[];
}

export interface SettingsRefreshResultDto {
  accepted: boolean;
}

export type HookStatus =
  | "not_installed"
  | "configured_unverified"
  | "active"
  | "process_only"
  | "error";

export interface HookStatusDto {
  codex: HookStatus;
  claude: HookStatus;
}

export interface HookDiagnosticPathsDto {
  config_path: string;
  config_exists: boolean;
  backup_path: string;
  backup_exists: boolean;
  hook_executable_path: string;
  hook_executable_exists: boolean;
}

export interface HookDiagnosticsDto {
  codex: HookDiagnosticPathsDto;
  claude: HookDiagnosticPathsDto;
}

export interface RuntimeLogDiagnosticsDto {
  directory_path: string;
  runtime_log_path: string;
  runtime_log_exists: boolean;
}
