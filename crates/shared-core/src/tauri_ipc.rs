use serde::{Deserialize, Serialize};

use crate::{
    app_config::AppConfig,
    settings_service::StatusSnapshotView,
};

pub const TAURI_SETTINGS_PROTOCOL_VERSION: &str = "cc_traffic_light.settings.v1";
pub const TAURI_SETTINGS_PIPE_NAME: &str = r"\\.\pipe\cc-traffic-light-settings-v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsBootstrapDto {
    pub protocol_version: String,
    pub transport: SettingsTransportDto,
    pub fake_mode: bool,
    pub pages: Vec<String>,
    pub about: SettingsAboutMetadataDto,
    pub snapshot: StatusSnapshotView,
    pub settings: AppConfig,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsTransportDto {
    pub kind: String,
    pub endpoint: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsAboutMetadataDto {
    pub product_name: String,
    pub version: String,
    pub runtime_description: String,
    pub config_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsSaveResultDto {
    pub settings: AppConfig,
    pub applied_keys: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsAppliedNotificationDto {
    pub applied_keys: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsRefreshResultDto {
    pub accepted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SettingsIpcCommand {
    #[serde(rename = "get_snapshot")]
    GetSnapshot,
    #[serde(rename = "get_settings")]
    GetSettings,
    #[serde(rename = "save_settings")]
    SaveSettings { settings: AppConfig },
    #[serde(rename = "request_refresh")]
    RequestRefresh,
    #[serde(rename = "notify_settings_applied")]
    NotifySettingsApplied { applied_keys: Vec<String> },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsIpcEnvelope {
    pub protocol_version: String,
    pub request_id: String,
    pub command: SettingsIpcCommand,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SettingsIpcResponse {
    #[serde(rename = "get_snapshot")]
    GetSnapshot { snapshot: StatusSnapshotView },
    #[serde(rename = "get_settings")]
    GetSettings { settings: AppConfig },
    #[serde(rename = "save_settings")]
    SaveSettings { result: SettingsSaveResultDto },
    #[serde(rename = "request_refresh")]
    RequestRefresh { result: SettingsRefreshResultDto },
    #[serde(rename = "notify_settings_applied")]
    NotifySettingsApplied { acknowledged: bool },
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsIpcResponseEnvelope {
    pub protocol_version: String,
    pub request_id: String,
    pub response: SettingsIpcResponse,
}
