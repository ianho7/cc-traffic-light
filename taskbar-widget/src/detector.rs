use std::collections::BTreeMap;

use crate::{
    agent_state::{AgentState, DisplayLoadOutcome, DisplayLoadResult, HookSummary},
    app_config::AppConfig,
    ui_state::{
        AppStatusSnapshot, DetectionMethod, SourceConfidence, SourceId, SourceStatus,
        SourceVisualState, WidgetMountState,
    },
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationKind {
    LogFile,
    StateFile,
    SessionFile,
    Process,
    HookState,
}

impl ObservationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LogFile => "log_file",
            Self::StateFile => "state_file",
            Self::SessionFile => "session_file",
            Self::Process => "process",
            Self::HookState => "hook_state",
        }
    }

    pub fn method(self) -> DetectionMethod {
        match self {
            Self::LogFile => DetectionMethod::LogFile,
            Self::StateFile => DetectionMethod::StateFile,
            Self::SessionFile => DetectionMethod::SessionFile,
            Self::Process => DetectionMethod::Process,
            Self::HookState => DetectionMethod::HookState,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceObservation {
    pub source_id: SourceId,
    pub kind: ObservationKind,
    pub state: SourceVisualState,
    pub confidence: SourceConfidence,
    pub updated_at: u64,
    pub message: Option<String>,
}

pub fn build_status_snapshot(
    config: &AppConfig,
    result: &DisplayLoadResult,
    widget_mount_state: WidgetMountState,
    last_widget_attach_at: Option<u64>,
) -> AppStatusSnapshot {
    let mut snapshot = AppStatusSnapshot::empty();
    snapshot.widget_mount_state = widget_mount_state;
    snapshot.last_widget_attach_at = last_widget_attach_at;
    snapshot.last_detection_refresh_at = Some(result.state.updated_at);
    snapshot.last_error_summary = diagnostic_error(result);

    let mut per_source = BTreeMap::new();
    for source_id in [SourceId::Codex, SourceId::Claude] {
        let enabled = match source_id {
            SourceId::Codex => config.monitoring.codex_enabled,
            SourceId::Claude => config.monitoring.claude_enabled,
        };
        let observations = collect_source_observations(source_id, enabled, result);
        let status = aggregate_source_status(source_id, observations);
        per_source.insert(source_id.as_str().to_string(), status);
    }

    snapshot.sources = per_source;
    snapshot.overall_state = aggregate_overall_state(snapshot.sources.values());
    snapshot
}

fn diagnostic_error(result: &DisplayLoadResult) -> Option<String> {
    match &result.outcome {
        DisplayLoadOutcome::Loaded | DisplayLoadOutcome::MissingFile => None,
        DisplayLoadOutcome::ReadError(error) => Some(format!("state_read_error: {error}")),
        DisplayLoadOutcome::InvalidJson => Some("state_invalid_json".to_string()),
    }
}

fn collect_source_observations(
    source_id: SourceId,
    enabled: bool,
    result: &DisplayLoadResult,
) -> Vec<SourceObservation> {
    if !enabled {
        return Vec::new();
    }

    let mut observations = Vec::new();
    if matches!(result.outcome, DisplayLoadOutcome::Loaded) {
        let key = source_id.as_str();
        if let Some(summary) = result.state.agents.get(key).map(|monitor| &monitor.summary) {
            observations.push(SourceObservation::from_hook_summary(
                source_id,
                ObservationKind::StateFile,
                summary,
            ));
        }
    }

    if let Some(observation) = process_fallback_observation(source_id) {
        observations.push(observation);
    }

    observations
}

fn aggregate_source_status(
    source_id: SourceId,
    observations: Vec<SourceObservation>,
) -> SourceStatus {
    if observations.is_empty() {
        return SourceStatus::undiscovered(source_id);
    }

    let mut best: Option<SourceObservation> = None;
    let mut conflict = false;

    for observation in observations {
        if let Some(current) = &best {
            let current_priority = source_priority(source_id, current.kind);
            let next_priority = source_priority(source_id, observation.kind);

            if next_priority < current_priority {
                best = Some(observation);
                conflict = false;
                continue;
            }

            if next_priority == current_priority {
                if observation.updated_at > current.updated_at {
                    if current.state != observation.state {
                        conflict = true;
                    }
                    best = Some(observation);
                    continue;
                }

                if observation.updated_at == current.updated_at
                    && current.state != observation.state
                {
                    conflict = true;
                }
            }
        } else {
            best = Some(observation);
        }
    }

    let Some(best) = best else {
        return SourceStatus::undiscovered(source_id);
    };

    if conflict {
        return SourceStatus {
            source_id,
            state: SourceVisualState::Untrusted,
            confidence: SourceConfidence::Untrusted,
            method: best.kind.method(),
            updated_at: best.updated_at,
            message: Some(format!("conflict_on_{}", best.kind.as_str())),
        };
    }

    SourceStatus {
        source_id,
        state: best.state,
        confidence: best.confidence,
        method: best.kind.method(),
        updated_at: best.updated_at,
        message: best.message,
    }
}

fn aggregate_overall_state<'a>(
    sources: impl Iterator<Item = &'a SourceStatus>,
) -> SourceVisualState {
    let mut best = SourceVisualState::Undiscovered;
    let mut best_priority = overall_priority(best);

    for source in sources {
        let priority = overall_priority(source.state);
        if priority > best_priority {
            best = source.state;
            best_priority = priority;
        }
    }

    best
}

fn source_priority(source_id: SourceId, kind: ObservationKind) -> u8 {
    match source_id {
        SourceId::Codex => match kind {
            ObservationKind::LogFile => 0,
            ObservationKind::StateFile => 1,
            ObservationKind::SessionFile => 2,
            ObservationKind::Process => 3,
            ObservationKind::HookState => 4,
        },
        SourceId::Claude => match kind {
            ObservationKind::LogFile => 0,
            ObservationKind::StateFile => 1,
            ObservationKind::SessionFile => 2,
            ObservationKind::Process => 3,
            ObservationKind::HookState => 4,
        },
    }
}

fn overall_priority(state: SourceVisualState) -> u8 {
    match state {
        SourceVisualState::Undiscovered => 0,
        SourceVisualState::Idle => 1,
        SourceVisualState::Working => 2,
        SourceVisualState::Attention => 3,
        SourceVisualState::Blocking => 4,
        SourceVisualState::Untrusted => 2,
    }
}

impl SourceObservation {
    fn from_hook_summary(
        source_id: SourceId,
        kind: ObservationKind,
        summary: &HookSummary,
    ) -> Self {
        Self {
            source_id,
            kind,
            state: SourceVisualState::from_hook_state(&summary.state, summary.has_stale),
            confidence: confidence_from_hook_summary(summary),
            updated_at: summary.updated_at,
            message: summary.highest_priority_task.clone(),
        }
    }
}

fn confidence_from_hook_summary(summary: &HookSummary) -> SourceConfidence {
    if summary.has_stale {
        SourceConfidence::Untrusted
    } else if matches!(summary.state, AgentState::Idle) && summary.active_task_count == 0 {
        SourceConfidence::Degraded
    } else {
        SourceConfidence::Confirmed
    }
}

fn process_fallback_observation(source_id: SourceId) -> Option<SourceObservation> {
    if !is_process_present(source_id) {
        return None;
    }

    Some(SourceObservation {
        source_id,
        kind: ObservationKind::Process,
        state: SourceVisualState::Idle,
        confidence: SourceConfidence::Degraded,
        updated_at: crate::agent_state::now_ms(),
        message: Some("process_present_only".to_string()),
    })
}

fn is_process_present(source_id: SourceId) -> bool {
    let Ok(snapshot) = (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }) else {
        return false;
    };

    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let mut found = false;
    unsafe {
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let process_name = utf16_to_string(&entry.szExeFile);
                if process_name_matches(source_id, &process_name) {
                    found = true;
                    break;
                }

                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = windows::Win32::Foundation::CloseHandle(snapshot);
    }

    found
}

fn process_name_matches(source_id: SourceId, process_name: &str) -> bool {
    let normalized = process_name.trim_end_matches('\0').to_ascii_lowercase();
    let normalized = normalized.strip_suffix(".exe").unwrap_or(&normalized);

    match source_id {
        SourceId::Codex => matches!(normalized, "codex"),
        SourceId::Claude => matches!(normalized, "claude" | "claude-code" | "claudecode"),
    }
}

fn utf16_to_string(value: &[u16]) -> String {
    let end = value
        .iter()
        .position(|code_unit| *code_unit == 0)
        .unwrap_or(value.len());
    String::from_utf16_lossy(&value[..end])
}
