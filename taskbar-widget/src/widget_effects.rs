use crate::{
    agent_state,
    ui_state::{AppStatusSnapshot, SourceVisualState},
};

const SLOW_BLINK_PERIOD_MS: u64 = 1_400;
const FAST_BLINK_PERIOD_MS: u64 = 450;
const COMPLETED_HOLD_MS: u64 = 1_800;
const ACTIVE_ALPHA: u8 = 255;
const DIM_ALPHA: u8 = 136;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LampRenderState {
    pub alpha: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroupRenderState {
    pub display_state: SourceVisualState,
    pub lamps: [LampRenderState; 3],
}

#[derive(Clone, Debug, Default)]
pub struct WidgetEffectsState {
    codex: SourceEffectState,
    claude: SourceEffectState,
    reduced_motion: bool,
}

#[derive(Clone, Debug, Default)]
struct SourceEffectState {
    completed_hold_until_ms: Option<u64>,
}

impl WidgetEffectsState {
    pub fn set_reduced_motion(&mut self, reduced_motion: bool) {
        self.reduced_motion = reduced_motion;
    }

    pub fn sync_snapshot(&mut self, snapshot: &AppStatusSnapshot, now_ms: u64) {
        self.codex
            .sync_state(source_state(snapshot, "codex"), now_ms);
        self.claude
            .sync_state(source_state(snapshot, "claude"), now_ms);
    }

    pub fn render_state_for(
        &self,
        snapshot: &AppStatusSnapshot,
        key: &str,
        now_ms: u64,
    ) -> GroupRenderState {
        let factual_state = source_state(snapshot, key);
        let state = match key {
            "codex" => self.codex.render_state(factual_state, now_ms),
            "claude" => self.claude.render_state(factual_state, now_ms),
            _ => SourceVisualState::Idle,
        };

        build_group_render_state(state, now_ms, self.reduced_motion)
    }

    pub fn needs_animation_frame(&self, snapshot: &AppStatusSnapshot, now_ms: u64) -> bool {
        !self.reduced_motion
            && [("codex", &self.codex), ("claude", &self.claude)]
                .into_iter()
                .any(|(key, effect)| {
                    effect.needs_animation_frame(source_state(snapshot, key), now_ms)
                })
    }
}

impl SourceEffectState {
    fn sync_state(&mut self, factual_state: SourceVisualState, now_ms: u64) {
        match factual_state {
            SourceVisualState::Completed => {
                self.completed_hold_until_ms = Some(now_ms.saturating_add(COMPLETED_HOLD_MS));
            }
            SourceVisualState::Working
            | SourceVisualState::NeedsAttention
            | SourceVisualState::Error => {
                self.completed_hold_until_ms = None;
            }
            SourceVisualState::Idle => {}
        }

        if self
            .completed_hold_until_ms
            .is_some_and(|hold_until| now_ms >= hold_until)
        {
            self.completed_hold_until_ms = None;
        }
    }

    fn render_state(&self, factual_state: SourceVisualState, now_ms: u64) -> SourceVisualState {
        match factual_state {
            SourceVisualState::Idle => {
                if self
                    .completed_hold_until_ms
                    .is_some_and(|hold_until| now_ms < hold_until)
                {
                    SourceVisualState::Completed
                } else {
                    SourceVisualState::Idle
                }
            }
            other => other,
        }
    }

    fn needs_animation_frame(&self, factual_state: SourceVisualState, now_ms: u64) -> bool {
        matches!(
            self.render_state(factual_state, now_ms),
            SourceVisualState::Working
                | SourceVisualState::NeedsAttention
                | SourceVisualState::Error
        ) || self
            .completed_hold_until_ms
            .is_some_and(|hold_until| now_ms < hold_until)
    }
}

pub fn now_ms() -> u64 {
    agent_state::now_ms()
}

fn build_group_render_state(
    display_state: SourceVisualState,
    now_ms: u64,
    reduced_motion: bool,
) -> GroupRenderState {
    let lamps = match display_state {
        SourceVisualState::Idle => [lamp(0), lamp(0), lamp(0)],
        SourceVisualState::Working => [
            lamp(if reduced_motion {
                ACTIVE_ALPHA
            } else {
                blink_alpha(now_ms, SLOW_BLINK_PERIOD_MS)
            }),
            lamp(0),
            lamp(0),
        ],
        SourceVisualState::NeedsAttention => [
            lamp(0),
            lamp(if reduced_motion {
                ACTIVE_ALPHA
            } else {
                blink_alpha(now_ms, FAST_BLINK_PERIOD_MS)
            }),
            lamp(0),
        ],
        SourceVisualState::Completed => [lamp(ACTIVE_ALPHA), lamp(0), lamp(0)],
        SourceVisualState::Error => [
            lamp(0),
            lamp(0),
            lamp(if reduced_motion {
                ACTIVE_ALPHA
            } else {
                blink_alpha(now_ms, SLOW_BLINK_PERIOD_MS)
            }),
        ],
    };

    GroupRenderState {
        display_state,
        lamps,
    }
}

fn lamp(alpha: u8) -> LampRenderState {
    LampRenderState { alpha }
}

fn blink_alpha(now_ms: u64, period_ms: u64) -> u8 {
    if period_ms == 0 {
        return ACTIVE_ALPHA;
    }

    let phase = now_ms % period_ms;
    if phase < period_ms / 2 {
        ACTIVE_ALPHA
    } else {
        DIM_ALPHA
    }
}

fn source_state(snapshot: &AppStatusSnapshot, key: &str) -> SourceVisualState {
    snapshot
        .sources
        .get(key)
        .map(|source| source.state)
        .unwrap_or(SourceVisualState::Idle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_state::{AppStatusSnapshot, SourceId, SourceStatus};

    fn snapshot_with_state(key: &str, state: SourceVisualState) -> AppStatusSnapshot {
        let mut snapshot = AppStatusSnapshot::empty();
        if let Some(source) = snapshot.sources.get_mut(key) {
            source.state = state;
        } else {
            snapshot.sources.insert(
                key.to_string(),
                SourceStatus::idle(match key {
                    "claude" => SourceId::Claude,
                    _ => SourceId::Codex,
                }),
            );
            snapshot.sources.get_mut(key).expect("source").state = state;
        }
        snapshot
    }

    #[test]
    fn completed_state_holds_after_snapshot_returns_to_idle() {
        let mut effects = WidgetEffectsState::default();
        let completed = snapshot_with_state("codex", SourceVisualState::Completed);
        effects.sync_snapshot(&completed, 1_000);

        let idle = snapshot_with_state("codex", SourceVisualState::Idle);
        effects.sync_snapshot(&idle, 1_100);
        assert_eq!(
            effects
                .render_state_for(&idle, "codex", 1_100)
                .display_state,
            SourceVisualState::Completed
        );
        assert_eq!(
            effects
                .render_state_for(&idle, "codex", 3_000)
                .display_state,
            SourceVisualState::Idle
        );
    }

    #[test]
    fn blocking_states_cancel_completed_hold_immediately() {
        let mut effects = WidgetEffectsState::default();
        effects.sync_snapshot(
            &snapshot_with_state("codex", SourceVisualState::Completed),
            1_000,
        );
        effects.sync_snapshot(
            &snapshot_with_state("codex", SourceVisualState::Error),
            1_100,
        );

        assert_eq!(
            effects
                .render_state_for(
                    &snapshot_with_state("codex", SourceVisualState::Error),
                    "codex",
                    1_200
                )
                .display_state,
            SourceVisualState::Error
        );
    }

    #[test]
    fn blink_profiles_use_distinct_periods() {
        let working = build_group_render_state(SourceVisualState::Working, 300, false);
        let attention = build_group_render_state(SourceVisualState::NeedsAttention, 300, false);

        assert_eq!(working.lamps[0].alpha, ACTIVE_ALPHA);
        assert_eq!(attention.lamps[1].alpha, DIM_ALPHA);
    }

    #[test]
    fn completed_uses_the_left_green_lamp_position() {
        let completed = build_group_render_state(SourceVisualState::Completed, 300, false);

        assert_eq!(completed.lamps[0].alpha, ACTIVE_ALPHA);
        assert_eq!(completed.lamps[1].alpha, 0);
        assert_eq!(completed.lamps[2].alpha, 0);
    }

    #[test]
    fn reduced_motion_keeps_each_active_state_distinguishable_without_blinking() {
        let mut effects = WidgetEffectsState::default();
        effects.set_reduced_motion(true);

        for (state, lamp_index) in [
            (SourceVisualState::Working, 0),
            (SourceVisualState::NeedsAttention, 1),
            (SourceVisualState::Error, 2),
        ] {
            let snapshot = snapshot_with_state("codex", state);
            let first = effects.render_state_for(&snapshot, "codex", 0);
            let later = effects.render_state_for(&snapshot, "codex", 900);
            assert_eq!(first.lamps[lamp_index].alpha, ACTIVE_ALPHA);
            assert_eq!(later.lamps[lamp_index].alpha, ACTIVE_ALPHA);
            assert!(!effects.needs_animation_frame(&snapshot, 900));
        }
    }
}
