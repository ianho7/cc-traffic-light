use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const CONFIG_FILE_NAME: &str = "config.json";
const CONFIG_SCHEMA_VERSION: u32 = 4;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub schema_version: u32,
    #[serde(default)]
    pub localization: LocalizationConfig,
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub monitoring: MonitoringConfig,
    #[serde(default)]
    pub appearance: AppearanceConfig,
    #[serde(default)]
    pub widget_visual: WidgetVisualConfig,
    #[serde(default)]
    pub diagnostics: DiagnosticsConfig,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalizationConfig {
    #[serde(default)]
    pub language: AppLanguage,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub autostart_enabled: bool,
    pub start_minimized_to_tray: bool,
    pub close_to_tray: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub codex_enabled: bool,
    pub claude_enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AppearanceConfig {
    #[serde(default)]
    pub ui_theme: UiTheme,
    pub indicator_style: IndicatorStyle,
    pub widget_size: WidgetSize,
    pub show_labels: bool,
    pub reduced_motion: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WidgetVisualConfig {
    #[serde(default)]
    pub placement: WidgetPlacement,
    #[serde(default)]
    pub palette: WidgetPaletteConfig,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetPlacement {
    Left,
    #[default]
    Right,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WidgetPaletteConfig {
    #[serde(default = "default_widget_green")]
    pub green: String,
    #[serde(default = "default_widget_yellow")]
    pub yellow: String,
    #[serde(default = "default_widget_red")]
    pub red: String,
    #[serde(default = "default_widget_inactive_brightness_percent")]
    pub inactive_brightness_percent: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsConfig {
    pub last_opened_page: SettingsPage,
    pub last_manual_refresh_at: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndicatorStyle {
    Classic,
    Minimal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetSize {
    Compact,
    Standard,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTheme {
    Light,
    #[default]
    Dark,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingsPage {
    Overview,
    General,
    Monitoring,
    Appearance,
    Diagnostics,
    About,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AppLanguage {
    #[default]
    #[serde(rename = "follow_system")]
    FollowSystem,
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en")]
    En,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigLoadOutcome {
    Loaded,
    CreatedDefaultFile,
    MissingFile,
    ReadError(String),
    InvalidJson,
}

#[derive(Clone, Debug)]
pub struct ConfigLoadResult {
    pub config: AppConfig,
    pub outcome: ConfigLoadOutcome,
    pub path: PathBuf,
}

impl AppConfig {
    pub fn default_v1() -> Self {
        Self {
            schema_version: CONFIG_SCHEMA_VERSION,
            localization: LocalizationConfig::default(),
            general: GeneralConfig {
                autostart_enabled: false,
                start_minimized_to_tray: true,
                close_to_tray: true,
            },
            monitoring: MonitoringConfig {
                codex_enabled: true,
                claude_enabled: true,
            },
            appearance: AppearanceConfig {
                ui_theme: UiTheme::Dark,
                indicator_style: IndicatorStyle::Classic,
                widget_size: WidgetSize::Standard,
                show_labels: true,
                reduced_motion: false,
            },
            widget_visual: WidgetVisualConfig::default(),
            diagnostics: DiagnosticsConfig {
                last_opened_page: SettingsPage::Overview,
                last_manual_refresh_at: None,
            },
        }
    }
}

impl Default for LocalizationConfig {
    fn default() -> Self {
        Self {
            language: AppLanguage::FollowSystem,
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        AppConfig::default_v1().general
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        AppConfig::default_v1().monitoring
    }
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        AppConfig::default_v1().appearance
    }
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        AppConfig::default_v1().diagnostics
    }
}

impl Default for WidgetVisualConfig {
    fn default() -> Self {
        Self {
            placement: WidgetPlacement::Right,
            palette: WidgetPaletteConfig::default(),
        }
    }
}

impl Default for WidgetPaletteConfig {
    fn default() -> Self {
        default_widget_palette()
    }
}

impl ConfigLoadOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Loaded => "loaded",
            Self::CreatedDefaultFile => "created_default_file",
            Self::MissingFile => "missing_file",
            Self::ReadError(_) => "read_error",
            Self::InvalidJson => "invalid_json",
        }
    }
}

pub fn config_dir_path() -> PathBuf {
    env::var_os("APPDATA")
        .map(|path| PathBuf::from(path).join("CcTrafficLight"))
        .unwrap_or_else(|| PathBuf::from(".").join("CcTrafficLight"))
}

pub fn config_file_path() -> PathBuf {
    config_dir_path().join(CONFIG_FILE_NAME)
}

pub fn load_config_diagnostic() -> ConfigLoadResult {
    let path = config_file_path();
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let config = AppConfig::default_v1();
            let outcome = match write_config(&path, &config) {
                Ok(()) => ConfigLoadOutcome::CreatedDefaultFile,
                Err(write_error) => ConfigLoadOutcome::ReadError(write_error.to_string()),
            };
            return ConfigLoadResult {
                config,
                outcome,
                path,
            };
        }
        Err(error) => {
            return ConfigLoadResult {
                config: AppConfig::default_v1(),
                outcome: ConfigLoadOutcome::ReadError(error.to_string()),
                path,
            };
        }
    };

    let Ok(parsed_config) = serde_json::from_str::<AppConfig>(strip_utf8_bom(&text)) else {
        return ConfigLoadResult {
            config: AppConfig::default_v1(),
            outcome: ConfigLoadOutcome::InvalidJson,
            path,
        };
    };

    let original_schema_version = parsed_config.schema_version;
    let config = normalize_config(parsed_config);
    if config.schema_version != original_schema_version {
        let _ = write_config(&path, &config);
    }

    ConfigLoadResult {
        config,
        outcome: ConfigLoadOutcome::Loaded,
        path,
    }
}

pub fn save_config(config: &AppConfig) -> io::Result<()> {
    let path = config_file_path();
    write_config(&path, &normalize_config(config.clone()))
}

fn write_config(path: &Path, config: &AppConfig) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let payload = serde_json::to_string_pretty(config).map_err(io::Error::other)?;
    fs::write(path, payload)
}

fn strip_utf8_bom(value: &str) -> &str {
    value.strip_prefix('\u{feff}').unwrap_or(value)
}

pub fn default_widget_palette() -> WidgetPaletteConfig {
    WidgetPaletteConfig {
        green: default_widget_green(),
        yellow: default_widget_yellow(),
        red: default_widget_red(),
        inactive_brightness_percent: default_widget_inactive_brightness_percent(),
    }
}

fn default_widget_green() -> String {
    "#34C759".to_string()
}

fn default_widget_yellow() -> String {
    "#FFCC00".to_string()
}

fn default_widget_red() -> String {
    "#FF3B30".to_string()
}

fn default_widget_inactive_brightness_percent() -> u8 {
    42
}

fn normalize_config(mut config: AppConfig) -> AppConfig {
    config.schema_version = CONFIG_SCHEMA_VERSION;
    config.widget_visual.palette.inactive_brightness_percent = config
        .widget_visual
        .palette
        .inactive_brightness_percent
        .clamp(12, 80);
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_localization_defaults_to_follow_system() {
        let config = serde_json::from_str::<AppConfig>(
            r##"{
                "schema_version": 1,
                "general": {
                    "autostart_enabled": false,
                    "start_minimized_to_tray": true,
                    "close_to_tray": true
                },
                "monitoring": {
                    "codex_enabled": true,
                    "claude_enabled": true
                },
                "appearance": {
                    "ui_theme": "dark",
                    "indicator_style": "classic",
                    "widget_size": "standard",
                    "show_labels": true,
                    "reduced_motion": false
                },
                "widget_visual": {
                    "palette": {
                        "green": "#34C759",
                        "yellow": "#FFCC00",
                        "red": "#FF3B30"
                    }
                },
                "diagnostics": {
                    "last_opened_page": "overview",
                    "last_manual_refresh_at": null
                }
            }"##,
        )
        .expect("config should deserialize");

        assert_eq!(config.localization.language, AppLanguage::FollowSystem);
    }

    #[test]
    fn language_round_trip_preserves_explicit_locale() {
        let mut config = AppConfig::default_v1();
        config.localization.language = AppLanguage::ZhCn;

        let encoded = serde_json::to_string(&config).expect("config should serialize");
        let decoded =
            serde_json::from_str::<AppConfig>(&encoded).expect("config should deserialize");

        assert_eq!(decoded.localization.language, AppLanguage::ZhCn);
        assert_eq!(decoded.schema_version, CONFIG_SCHEMA_VERSION);
    }

    #[test]
    fn missing_ui_theme_defaults_to_dark() {
        let config = serde_json::from_str::<AppConfig>(
            r#"{
                "schema_version": 2,
                "appearance": {
                    "indicator_style": "classic",
                    "widget_size": "standard",
                    "show_labels": true,
                    "reduced_motion": false
                }
            }"#,
        )
        .expect("config should deserialize");

        assert_eq!(config.appearance.ui_theme, UiTheme::Dark);
    }

    #[test]
    fn missing_widget_visual_defaults_to_phase1_palette() {
        let config = serde_json::from_str::<AppConfig>(
            r#"{
                "schema_version": 2,
                "appearance": {
                    "ui_theme": "dark",
                    "indicator_style": "classic",
                    "widget_size": "standard",
                    "show_labels": true,
                    "reduced_motion": false
                }
            }"#,
        )
        .expect("config should deserialize");

        assert_eq!(config.widget_visual.palette.green, "#34C759");
        assert_eq!(config.widget_visual.palette.yellow, "#FFCC00");
        assert_eq!(config.widget_visual.palette.red, "#FF3B30");
        assert_eq!(config.widget_visual.palette.inactive_brightness_percent, 42);
        assert_eq!(config.widget_visual.placement, WidgetPlacement::Right);
    }

    #[test]
    fn widget_visual_round_trip_preserves_custom_palette() {
        let mut config = AppConfig::default_v1();
        config.widget_visual.placement = WidgetPlacement::Left;
        config.widget_visual.palette.green = "#00FF88".to_string();
        config.widget_visual.palette.yellow = "#FFD700".to_string();
        config.widget_visual.palette.red = "#FF4D6D".to_string();
        config.widget_visual.palette.inactive_brightness_percent = 56;

        let encoded = serde_json::to_string(&config).expect("config should serialize");
        let decoded =
            serde_json::from_str::<AppConfig>(&encoded).expect("config should deserialize");

        assert_eq!(decoded.widget_visual, config.widget_visual);
    }
}
