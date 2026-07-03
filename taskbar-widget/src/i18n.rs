use std::{collections::BTreeMap, env, sync::OnceLock};

use serde::Deserialize;

use crate::{
    app_config::{AppConfig, AppLanguage, IndicatorStyle, UiTheme, WidgetSize},
    ui_state::{
        AppStatusSnapshot, DetectionMethod, SourceConfidence, SourceId, SourceStatus,
        SourceVisualState, WidgetMountState,
    },
};

const EN_RESOURCE: &str = include_str!("../ui/i18n/en.json");
const ZH_CN_RESOURCE: &str = include_str!("../ui/i18n/zh-CN.json");

static EN_BUNDLE: OnceLock<TranslationBundle> = OnceLock::new();
static ZH_CN_BUNDLE: OnceLock<TranslationBundle> = OnceLock::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppLocale {
    En,
    ZhCn,
}

#[derive(Clone, Copy, Debug)]
pub struct Localizer {
    locale: AppLocale,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct TranslationBundle(BTreeMap<String, String>);

impl Localizer {
    pub fn for_config(config: &AppConfig) -> Self {
        Self {
            locale: effective_locale(config),
        }
    }

    pub fn locale(self) -> AppLocale {
        self.locale
    }

    pub fn text(self, key: &str) -> String {
        bundle_for(self.locale)
            .get(key)
            .or_else(|| bundle_for(AppLocale::En).get(key))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    pub fn bool_label(self, value: bool) -> String {
        self.text(if value { "common.on" } else { "common.off" })
    }

    pub fn language_label(self, language: AppLanguage) -> String {
        self.text(match language {
            AppLanguage::FollowSystem => "language.follow_system",
            AppLanguage::ZhCn => "language.zh_cn",
            AppLanguage::En => "language.en",
        })
    }

    pub fn indicator_style_label(self, style: IndicatorStyle) -> String {
        self.text(match style {
            IndicatorStyle::Classic => "indicator_style.classic",
            IndicatorStyle::Minimal => "indicator_style.minimal",
        })
    }

    pub fn widget_size_label(self, size: WidgetSize) -> String {
        self.text(match size {
            WidgetSize::Compact => "widget_size.compact",
            WidgetSize::Standard => "widget_size.standard",
        })
    }

    pub fn ui_theme_label(self, theme: UiTheme) -> String {
        self.text(match theme {
            UiTheme::Light => "ui_theme.light",
            UiTheme::Dark => "ui_theme.dark",
        })
    }

    pub fn source_label(self, source_id: SourceId) -> String {
        self.text(match source_id {
            SourceId::Codex => "source.codex",
            SourceId::Claude => "source.claude",
        })
    }

    pub fn state_label(self, state: SourceVisualState) -> String {
        self.text(match state {
            SourceVisualState::Undiscovered => "state.undiscovered",
            SourceVisualState::Idle => "state.idle",
            SourceVisualState::Working => "state.working",
            SourceVisualState::Attention => "state.attention",
            SourceVisualState::Blocking => "state.blocking",
            SourceVisualState::Untrusted => "state.untrusted",
        })
    }

    pub fn widget_mount_label(self, state: WidgetMountState) -> String {
        self.text(match state {
            WidgetMountState::Attached => "widget_mount.attached",
            WidgetMountState::TrayOnly => "widget_mount.tray_only",
            WidgetMountState::Retrying => "widget_mount.retrying",
        })
    }

    pub fn method_label(self, method: DetectionMethod) -> String {
        self.text(match method {
            DetectionMethod::LogFile => "method.log_file",
            DetectionMethod::StateFile => "method.state_file",
            DetectionMethod::SessionFile => "method.session_file",
            DetectionMethod::Process => "method.process",
            DetectionMethod::HookState => "method.hook_state",
            DetectionMethod::Unknown => "method.unknown",
        })
    }

    pub fn confidence_label(self, confidence: SourceConfidence) -> String {
        self.text(match confidence {
            SourceConfidence::Confirmed => "confidence.confirmed",
            SourceConfidence::Degraded => "confidence.degraded",
            SourceConfidence::Untrusted => "confidence.untrusted",
        })
    }

    pub fn source_detail(self, source: &SourceStatus) -> String {
        let mut parts = vec![
            format!(
                "{} {}",
                self.text("detail.method"),
                self.method_label(source.method)
            ),
            format!(
                "{} {}",
                self.text("detail.confidence"),
                self.confidence_label(source.confidence)
            ),
            format!("{} {}", self.text("detail.updated"), source.updated_at),
        ];
        if let Some(message) = &source.message {
            parts.push(message.clone());
        }
        parts.join(" | ")
    }

    pub fn status_detail(self, snapshot: &AppStatusSnapshot) -> String {
        let codex_line = snapshot
            .sources
            .get("codex")
            .map(|source| {
                format!(
                    "{} {}",
                    self.source_label(source.source_id),
                    self.state_label(source.state)
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "{} {}",
                    self.source_label(SourceId::Codex),
                    self.text("detail.pending")
                )
            });
        let claude_line = snapshot
            .sources
            .get("claude")
            .map(|source| {
                format!(
                    "{} {}",
                    self.source_label(source.source_id),
                    self.state_label(source.state)
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "{} {}",
                    self.source_label(SourceId::Claude),
                    self.text("detail.pending")
                )
            });
        format!("{codex_line} | {claude_line}")
    }

    pub fn timestamp_line(self, prefix_key: &str, value: Option<u64>) -> String {
        let prefix = self.text(prefix_key);
        match value {
            Some(timestamp) => format!("{prefix}: {timestamp}"),
            None => format!("{prefix}: {}", self.text("detail.pending")),
        }
    }

    pub fn tray_tooltip(self, snapshot: &AppStatusSnapshot) -> String {
        let codex = snapshot
            .sources
            .get("codex")
            .map(|source| self.state_label(source.state))
            .unwrap_or_else(|| self.state_label(SourceVisualState::Undiscovered));
        let claude = snapshot
            .sources
            .get("claude")
            .map(|source| self.state_label(source.state))
            .unwrap_or_else(|| self.state_label(SourceVisualState::Undiscovered));

        format!(
            "{} | {}={} | {}={} | {}={}",
            self.text("app.name"),
            self.text("tray.tooltip.overall"),
            self.state_label(snapshot.overall_state),
            self.text("tray.tooltip.codex"),
            codex,
            self.text("tray.tooltip.claude"),
            claude
        )
    }
}

pub fn effective_locale(config: &AppConfig) -> AppLocale {
    match config.localization.language {
        AppLanguage::ZhCn => AppLocale::ZhCn,
        AppLanguage::En => AppLocale::En,
        AppLanguage::FollowSystem => detect_system_locale(),
    }
}

fn detect_system_locale() -> AppLocale {
    let raw = env::var("LC_ALL")
        .ok()
        .or_else(|| env::var("LC_MESSAGES").ok())
        .or_else(|| env::var("LANG").ok())
        .or_else(|| env::var("LANGUAGE").ok())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if raw.contains("zh") {
        AppLocale::ZhCn
    } else {
        AppLocale::En
    }
}

fn bundle_for(locale: AppLocale) -> &'static TranslationBundle {
    match locale {
        AppLocale::En => EN_BUNDLE.get_or_init(|| parse_bundle(EN_RESOURCE)),
        AppLocale::ZhCn => ZH_CN_BUNDLE.get_or_init(|| parse_bundle(ZH_CN_RESOURCE)),
    }
}

fn parse_bundle(raw: &str) -> TranslationBundle {
    serde_json::from_str(raw).expect("translation resource should be valid JSON")
}

impl TranslationBundle {
    fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }

    #[cfg(test)]
    fn keys(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zh_cn_bundle_matches_en_keys() {
        let en_keys: Vec<_> = bundle_for(AppLocale::En).keys().cloned().collect();
        let zh_keys: Vec<_> = bundle_for(AppLocale::ZhCn).keys().cloned().collect();

        assert_eq!(en_keys, zh_keys);
    }

    #[test]
    fn localizer_uses_explicit_locale_from_config() {
        let mut config = AppConfig::default_v1();
        config.localization.language = AppLanguage::ZhCn;

        let localizer = Localizer::for_config(&config);

        assert_eq!(localizer.locale(), AppLocale::ZhCn);
        assert_eq!(localizer.text("settings.nav.general"), "通用");
    }
}
