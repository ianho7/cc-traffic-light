use crate::agent_state::AgentState;

pub use shared_core::ui_state::*;

pub fn hook_visual_state_from_agent_state(
    state: &AgentState,
    _has_stale: bool,
) -> SourceVisualState {
    match state {
        AgentState::Idle => SourceVisualState::Idle,
        AgentState::Working => SourceVisualState::Working,
        AgentState::Done => SourceVisualState::Completed,
        AgentState::Waiting => SourceVisualState::NeedsAttention,
        AgentState::Error => SourceVisualState::Error,
    }
}
