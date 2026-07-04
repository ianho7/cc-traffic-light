use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use crate::{app_config::AppConfig, ui_state::AppStatusSnapshot};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettingsReadModel {
    pub config: AppConfig,
    pub snapshot: AppStatusSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceStatusView {
    pub source_id: String,
    pub state: String,
    pub confidence: String,
    pub method: String,
    pub updated_at: u64,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatusSnapshotView {
    pub widget_mount_state: String,
    pub overall_state: String,
    pub last_widget_attach_at: Option<u64>,
    pub last_detection_refresh_at: Option<u64>,
    pub last_error_summary: Option<String>,
    pub sources: BTreeMap<String, SourceStatusView>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettingsServiceError {
    StoreUnavailable(String),
    Persistence(String),
    SnapshotUnavailable(String),
    Refresh(String),
}

pub trait SettingsService {
    fn load_settings(&self) -> Result<AppConfig, SettingsServiceError>;
    fn save_settings(&self, config: &AppConfig) -> Result<(), SettingsServiceError>;
    fn read_status_snapshot(&self) -> Result<AppStatusSnapshot, SettingsServiceError>;
    fn apply_settings_change<F>(&self, mutate: F) -> Result<AppConfig, SettingsServiceError>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = self.load_settings()?;
        mutate(&mut config);
        self.save_settings(&config)?;
        Ok(config)
    }

    fn read_model(&self) -> Result<SettingsReadModel, SettingsServiceError> {
        Ok(SettingsReadModel {
            config: self.load_settings()?,
            snapshot: self.read_status_snapshot()?,
        })
    }
}
