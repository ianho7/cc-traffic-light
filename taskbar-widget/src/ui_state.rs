use crate::agent_state::AgentState;

pub use shared_core::ui_state::*;

pub fn hook_visual_state_from_agent_state(
    state: &AgentState,
    has_stale: bool,
) -> SourceVisualState {
    if has_stale {
        return SourceVisualState::Untrusted;
    }

    match state {
        AgentState::Idle => SourceVisualState::Idle,
        AgentState::Working => SourceVisualState::Working,
        AgentState::Done => SourceVisualState::Attention,
        AgentState::Waiting => SourceVisualState::Blocking,
        AgentState::Error => SourceVisualState::Blocking,
    }
}
