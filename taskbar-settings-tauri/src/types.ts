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
  };
  diagnostics: {
    last_opened_page: SettingsPageId;
    last_manual_refresh_at: number | null;
  };
}

export interface WidgetPaletteConfig {
  green: string;
  yellow: string;
  red: string;
  inactive_brightness_percent: number;
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
