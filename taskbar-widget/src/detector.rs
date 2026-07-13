use std::collections::BTreeMap;

use crate::{
    agent_state::{AgentState, DisplayLoadOutcome, DisplayLoadResult, HookSummary},
    app_config::AppConfig,
    ui_state::{
        AppStatusSnapshot, DetectionMethod, SourceConfidence, SourceId, SourceStatus,
        SourceVisualState, WidgetMountState, hook_visual_state_from_agent_state,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceObservation {
    pub source_id: SourceId,
    pub kind: DetectionMethod,
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
        if let Some(summary) = result.state.agents.get(key) {
            observations.push(SourceObservation::from_hook_summary(
                source_id,
                DetectionMethod::StateFile,
                summary,
            ));
        }
    }

    observations
}

fn aggregate_source_status(
    source_id: SourceId,
    observations: Vec<SourceObservation>,
) -> SourceStatus {
    if observations.is_empty() {
        return SourceStatus::idle(source_id);
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
        return SourceStatus::idle(source_id);
    };

    if conflict {
        return SourceStatus {
            source_id,
            state: SourceVisualState::Idle,
            confidence: SourceConfidence::Degraded,
            method: best.kind,
            updated_at: best.updated_at,
            message: None,
        };
    }

    SourceStatus {
        source_id,
        state: best.state,
        confidence: best.confidence,
        method: best.kind,
        updated_at: best.updated_at,
        message: best.message,
    }
}

fn aggregate_overall_state<'a>(
    sources: impl Iterator<Item = &'a SourceStatus>,
) -> SourceVisualState {
    let mut best = SourceVisualState::Idle;
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

fn source_priority(_source_id: SourceId, kind: DetectionMethod) -> u8 {
    match kind {
        DetectionMethod::LogFile => 0,
        DetectionMethod::StateFile => 1,
        DetectionMethod::SessionFile => 2,
        DetectionMethod::Process => 3,
        DetectionMethod::HookState => 4,
        DetectionMethod::Unknown => 0,
    }
}

fn overall_priority(state: SourceVisualState) -> u8 {
    match state {
        SourceVisualState::Idle => 0,
        SourceVisualState::Completed => 1,
        SourceVisualState::Working => 2,
        SourceVisualState::NeedsAttention => 3,
        SourceVisualState::Error => 4,
    }
}

impl SourceObservation {
    fn from_hook_summary(
        source_id: SourceId,
        kind: DetectionMethod,
        summary: &HookSummary,
    ) -> Self {
        Self {
            source_id,
            kind,
            state: hook_visual_state_from_agent_state(&summary.state, summary.has_stale),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn status(source_id: SourceId, state: SourceVisualState) -> SourceStatus {
        SourceStatus {
            source_id,
            state,
            confidence: SourceConfidence::Confirmed,
            method: DetectionMethod::StateFile,
            updated_at: 1,
            message: None,
        }
    }

    fn observation(
        kind: DetectionMethod,
        state: SourceVisualState,
        updated_at: u64,
    ) -> SourceObservation {
        SourceObservation {
            source_id: SourceId::Codex,
            kind,
            state,
            confidence: SourceConfidence::Confirmed,
            updated_at,
            message: None,
        }
    }

    #[test]
    fn overall_state_uses_error_then_attention_then_working_priority() {
        assert_eq!(
            aggregate_overall_state(
                [
                    status(SourceId::Codex, SourceVisualState::Completed),
                    status(SourceId::Claude, SourceVisualState::Working),
                    status(SourceId::Claude, SourceVisualState::NeedsAttention),
                    status(SourceId::Codex, SourceVisualState::Error),
                ]
                .iter()
            ),
            SourceVisualState::Error
        );
    }

    #[test]
    fn higher_priority_observation_wins_over_lower_priority_observation() {
        let result = aggregate_source_status(
            SourceId::Codex,
            vec![
                observation(
                    DetectionMethod::Process,
                    SourceVisualState::Error,
                    99,
                ),
                observation(
                    DetectionMethod::StateFile,
                    SourceVisualState::Working,
                    1,
                ),
            ],
        );

        assert_eq!(result.method, DetectionMethod::StateFile);
        assert_eq!(result.state, SourceVisualState::Working);
    }
}
