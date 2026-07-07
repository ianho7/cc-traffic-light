use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::settings_service::StatusSnapshotView;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceId {
    Codex,
    Claude,
}

impl SourceId {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "codex" => Some(Self::Codex),
            "claude" | "claude_code" | "claudecode" => Some(Self::Claude),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DetectionMethod {
    LogFile,
    StateFile,
    SessionFile,
    Process,
    HookState,
    Unknown,
}

impl DetectionMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LogFile => "log_file",
            Self::StateFile => "state_file",
            Self::SessionFile => "session_file",
            Self::Process => "process",
            Self::HookState => "hook_state",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceConfidence {
    Confirmed,
    Degraded,
    Untrusted,
}

impl SourceConfidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::Degraded => "degraded",
            Self::Untrusted => "untrusted",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceVisualState {
    Idle,
    Working,
    NeedsAttention,
    Completed,
    Error,
}

impl SourceVisualState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::NeedsAttention => "needs_attention",
            Self::Completed => "completed",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WidgetMountState {
    Attached,
    TrayOnly,
    Retrying,
}

impl WidgetMountState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Attached => "attached",
            Self::TrayOnly => "tray_only",
            Self::Retrying => "retrying",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceStatus {
    pub source_id: SourceId,
    pub state: SourceVisualState,
    pub confidence: SourceConfidence,
    pub method: DetectionMethod,
    pub updated_at: u64,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppStatusSnapshot {
    pub widget_mount_state: WidgetMountState,
    pub overall_state: SourceVisualState,
    pub last_widget_attach_at: Option<u64>,
    pub last_detection_refresh_at: Option<u64>,
    pub last_error_summary: Option<String>,
    pub sources: BTreeMap<String, SourceStatus>,
}

impl AppStatusSnapshot {
    pub fn empty() -> Self {
        let mut sources = BTreeMap::new();
        for source_id in [SourceId::Codex, SourceId::Claude] {
            sources.insert(
                source_id.as_str().to_string(),
                SourceStatus::idle(source_id),
            );
        }

        Self {
            widget_mount_state: WidgetMountState::Attached,
            overall_state: SourceVisualState::Idle,
            last_widget_attach_at: None,
            last_detection_refresh_at: None,
            last_error_summary: None,
            sources,
        }
    }
}

impl SourceStatus {
    pub fn idle(source_id: SourceId) -> Self {
        Self {
            source_id,
            state: SourceVisualState::Idle,
            confidence: SourceConfidence::Degraded,
            method: DetectionMethod::Unknown,
            updated_at: 0,
            message: None,
        }
    }
}

impl From<AppStatusSnapshot> for StatusSnapshotView {
    fn from(value: AppStatusSnapshot) -> Self {
        Self {
            widget_mount_state: value.widget_mount_state.as_str().to_string(),
            overall_state: value.overall_state.as_str().to_string(),
            last_widget_attach_at: value.last_widget_attach_at,
            last_detection_refresh_at: value.last_detection_refresh_at,
            last_error_summary: value.last_error_summary,
            sources: value
                .sources
                .into_iter()
                .map(|(key, source)| {
                    (
                        key,
                        crate::settings_service::SourceStatusView {
                            source_id: source.source_id.as_str().to_string(),
                            state: source.state.as_str().to_string(),
                            confidence: source.confidence.as_str().to_string(),
                            method: source.method.as_str().to_string(),
                            updated_at: source.updated_at,
                            message: source.message,
                        },
                    )
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_projection_preserves_source_fields() {
        let mut snapshot = AppStatusSnapshot::empty();
        snapshot.overall_state = SourceVisualState::Working;
        snapshot.last_error_summary = Some("state_read_error".to_string());
        snapshot.sources.insert(
            "codex".to_string(),
            SourceStatus {
                source_id: SourceId::Codex,
                state: SourceVisualState::Working,
                confidence: SourceConfidence::Confirmed,
                method: DetectionMethod::HookState,
                updated_at: 42,
                message: Some("codex_task".to_string()),
            },
        );

        let projected: StatusSnapshotView = snapshot.into();
        let codex = projected.sources.get("codex").expect("codex source");

        assert_eq!(projected.overall_state, "working");
        assert_eq!(projected.last_error_summary.as_deref(), Some("state_read_error"));
        assert_eq!(codex.source_id, "codex");
        assert_eq!(codex.state, "working");
        assert_eq!(codex.confidence, "confirmed");
        assert_eq!(codex.method, "hook_state");
        assert_eq!(codex.updated_at, 42);
        assert_eq!(codex.message.as_deref(), Some("codex_task"));
    }
}
