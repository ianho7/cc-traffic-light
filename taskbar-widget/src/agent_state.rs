use std::{
    collections::BTreeMap,
    env, fs, io,
    path::{Path, PathBuf},
    process,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::ui_state::SourceId;

use serde::{Deserialize, Serialize};
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, WAIT_ABANDONED, WAIT_OBJECT_0, WAIT_TIMEOUT},
        System::Threading::{CreateMutexW, INFINITE, ReleaseMutex, WaitForSingleObject},
    },
    core::w,
};

pub const STATE_DIR_ENV: &str = "TASKBAR_WIDGET_STATE_HOME";
const STATE_FILE_NAME: &str = "state.json";
const SCHEMA_VERSION: u32 = 1;
const LOCK_TIMEOUT_MS: u32 = 2_000;
const DONE_RETENTION_MS: u64 = 60 * 1_000;
const ERROR_RETENTION_MS: u64 = 30 * 60 * 1_000;
const WORKING_STALE_MS: u64 = 10 * 60 * 1_000;
const WAITING_STALE_MS: u64 = 24 * 60 * 60 * 1_000;
const MAX_MESSAGE_CHARS: usize = 160;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentState {
    Idle,
    Working,
    Done,
    Waiting,
    Error,
}

impl AgentState {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "idle" => Some(Self::Idle),
            "working" => Some(Self::Working),
            "done" => Some(Self::Done),
            "waiting" => Some(Self::Waiting),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::Done => "done",
            Self::Waiting => "waiting",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskStatus {
    pub agent: SourceId,
    pub task_key: String,
    pub session_id: String,
    pub session_id_source: String,
    pub state: AgentState,
    pub updated_at: u64,
    pub event_order: u64,
    pub event_order_source: String,
    pub hook_name: String,
    pub message: Option<String>,
    pub summary_eligible: bool,
    pub stale: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HookSummary {
    pub state: AgentState,
    pub active_task_count: usize,
    pub has_stale: bool,
    pub stale_task_count: usize,
    pub highest_priority_task: Option<String>,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HookDiagnostics {
    pub last_ignored_event: Option<String>,
    pub last_corrupt_recovery_at: Option<u64>,
    pub phase0_payload_sampling: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HookMonitorState {
    pub schema_version: u32,
    pub updated_at: u64,
    pub tasks: BTreeMap<String, TaskStatus>,
    pub global_summary: HookSummary,
    pub agents: BTreeMap<String, HookSummary>,
    pub diagnostics: HookDiagnostics,
}

#[derive(Clone, Debug)]
pub struct HookEventUpdate {
    pub agent: SourceId,
    pub session_id: Option<String>,
    pub session_id_source: String,
    pub hook_name: String,
    pub state: AgentState,
    pub event_order: u64,
    pub event_order_source: String,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisplayLoadOutcome {
    Loaded,
    MissingFile,
    ReadError(String),
    InvalidJson,
}

impl DisplayLoadOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Loaded => "loaded",
            Self::MissingFile => "missing_file",
            Self::ReadError(_) => "read_error",
            Self::InvalidJson => "invalid_json",
        }
    }
}

#[derive(Clone, Debug)]
pub struct DisplayLoadResult {
    pub state: HookMonitorState,
    pub outcome: DisplayLoadOutcome,
    pub path: PathBuf,
}

impl HookMonitorState {
    pub fn default_at(now_ms: u64) -> Self {
        let idle = HookSummary {
            state: AgentState::Idle,
            active_task_count: 0,
            has_stale: false,
            stale_task_count: 0,
            highest_priority_task: None,
            updated_at: now_ms,
        };
        let mut agents = BTreeMap::new();
        agents.insert(
            "claude".to_string(),
            idle.clone(),
        );
        agents.insert(
            "codex".to_string(),
            idle.clone(),
        );

        Self {
            schema_version: SCHEMA_VERSION,
            updated_at: now_ms,
            tasks: BTreeMap::new(),
            global_summary: idle,
            agents,
            diagnostics: HookDiagnostics {
                last_ignored_event: None,
                last_corrupt_recovery_at: None,
                phase0_payload_sampling: "synthetic_payload_until_real_hook_samples_are_available"
                    .to_string(),
            },
        }
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

pub fn state_file_path() -> PathBuf {
    let base = env::var_os(STATE_DIR_ENV)
        .map(PathBuf::from)
        .or_else(|| env::var_os("APPDATA").map(|path| PathBuf::from(path).join("CcTrafficLight")))
        .unwrap_or_else(|| PathBuf::from(".").join("CcTrafficLight"));

    base.join(STATE_FILE_NAME)
}

pub fn load_state_for_display() -> HookMonitorState {
    load_state_for_display_diagnostic().state
}

pub fn load_state_for_display_diagnostic() -> DisplayLoadResult {
    let now = now_ms();
    let path = state_file_path();
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return DisplayLoadResult {
                state: HookMonitorState::default_at(now),
                outcome: DisplayLoadOutcome::MissingFile,
                path,
            };
        }
        Err(error) => {
            return DisplayLoadResult {
                state: HookMonitorState::default_at(now),
                outcome: DisplayLoadOutcome::ReadError(error.to_string()),
                path,
            };
        }
    };
    let Ok(mut state) = serde_json::from_str::<HookMonitorState>(strip_utf8_bom(&text)) else {
        return DisplayLoadResult {
            state: HookMonitorState::default_at(now),
            outcome: DisplayLoadOutcome::InvalidJson,
            path,
        };
    };

    prune_expired_tasks(&mut state, now);
    refresh_summaries(&mut state, now);
    DisplayLoadResult {
        state,
        outcome: DisplayLoadOutcome::Loaded,
        path,
    }
}

pub fn update_state<F>(mutator: F) -> io::Result<HookMonitorState>
where
    F: FnOnce(&mut HookMonitorState),
{
    let _guard = StateMutexGuard::acquire(LOCK_TIMEOUT_MS)?;
    let path = state_file_path();
    let now = now_ms();
    let mut state = read_state_or_recover(&path, now)?;
    prune_expired_tasks(&mut state, now);
    mutator(&mut state);
    refresh_summaries(&mut state, now_ms());
    write_state_atomic(&path, &state)?;
    Ok(state)
}

pub fn apply_hook_event(update: HookEventUpdate) -> io::Result<HookMonitorState> {
    update_state(|state| {
        let task_key = task_key(update.agent.as_str(), update.session_id.as_deref());
        let session_id = update
            .session_id
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let summary_eligible = update.session_id.is_some();

        if let Some(current) = state.tasks.get(&task_key) {
            if update.event_order < current.event_order {
                state.diagnostics.last_ignored_event = Some(format!(
                    "ignored stale event task={} hook={} incoming={} current={}",
                    task_key, update.hook_name, update.event_order, current.event_order
                ));
                return;
            }
        }

        state.tasks.insert(
            task_key.clone(),
            TaskStatus {
                agent: update.agent,
                task_key,
                session_id,
                session_id_source: update.session_id_source,
                state: update.state,
                updated_at: now_ms(),
                event_order: update.event_order,
                event_order_source: update.event_order_source,
                hook_name: update.hook_name,
                message: update.message.map(|message| sanitize_message(&message)),
                summary_eligible,
                stale: false,
            },
        );
    })
}

pub fn debug_set_task(task_key: &str, state_value: AgentState) -> io::Result<HookMonitorState> {
    let now = now_ms();
    let (agent, session_id, eligible) = parse_task_key(task_key);
    update_state(|state| {
        state.tasks.insert(
            task_key.to_string(),
            TaskStatus {
                agent,
                task_key: task_key.to_string(),
                session_id,
                session_id_source: "debug_cli".to_string(),
                state: state_value,
                updated_at: now,
                event_order: now,
                event_order_source: "debug_cli".to_string(),
                hook_name: "debug_set".to_string(),
                message: Some("debug override".to_string()),
                summary_eligible: eligible,
                stale: false,
            },
        );
    })
}

pub fn debug_clear_task(task_key: &str) -> io::Result<HookMonitorState> {
    update_state(|state| {
        state.tasks.remove(task_key);
    })
}

pub fn clear_agent_tasks(agent: SourceId) -> io::Result<HookMonitorState> {
    update_state(|state| {
        remove_tasks_for_agent(state, agent);
    })
}

fn remove_tasks_for_agent(state: &mut HookMonitorState, agent: SourceId) {
    state.tasks.retain(|_, task| task.agent != agent);
}

pub fn task_key(agent_name: &str, session_id: Option<&str>) -> String {
    match session_id.filter(|value| !value.trim().is_empty()) {
        Some(session_id) => format!("{}_{}", agent_name, safe_key_part(session_id)),
        None => format!("{}_unknown", agent_name),
    }
}

fn read_state_or_recover(path: &Path, now: u64) -> io::Result<HookMonitorState> {
    if !path.exists() {
        return Ok(HookMonitorState::default_at(now));
    }

    let text = fs::read_to_string(path)?;
    match serde_json::from_str::<HookMonitorState>(strip_utf8_bom(&text)) {
        Ok(mut state) => {
            state.schema_version = SCHEMA_VERSION;
            Ok(state)
        }
        Err(_) => {
            backup_corrupt_state(path, now)?;
            let mut state = HookMonitorState::default_at(now);
            state.diagnostics.last_corrupt_recovery_at = Some(now);
            Ok(state)
        }
    }
}

fn write_state_atomic(path: &Path, state: &HookMonitorState) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_path = path.with_extension(format!("json.{}.{}.tmp", process::id(), now_ms()));
    let payload = serde_json::to_string_pretty(state).map_err(io::Error::other)?;
    fs::write(&temp_path, payload)?;
    fs::rename(&temp_path, path)?;
    Ok(())
}

fn strip_utf8_bom(value: &str) -> &str {
    value.strip_prefix('\u{feff}').unwrap_or(value)
}

fn backup_corrupt_state(path: &Path, now: u64) -> io::Result<()> {
    let backup = path.with_file_name(format!("state.corrupt.{now}.json"));
    fs::rename(path, backup)
}

fn prune_expired_tasks(state: &mut HookMonitorState, now: u64) {
    state.tasks.retain(|_, task| match task.state {
        AgentState::Done => now.saturating_sub(task.updated_at) <= DONE_RETENTION_MS,
        AgentState::Error => now.saturating_sub(task.updated_at) <= ERROR_RETENTION_MS,
        _ => true,
    });
}

fn refresh_summaries(state: &mut HookMonitorState, now: u64) {
    state.updated_at = now;
    for task in state.tasks.values_mut() {
        let age = now.saturating_sub(task.updated_at);
        task.stale = match task.state {
            AgentState::Working => age > WORKING_STALE_MS,
            AgentState::Waiting => age > WAITING_STALE_MS,
            _ => false,
        };
    }

    state.global_summary = summarize_tasks(state.tasks.values(), now, None);
    for agent in [SourceId::Codex, SourceId::Claude] {
        state.agents.insert(
            agent.as_str().to_string(),
            summarize_tasks(state.tasks.values(), now, Some(agent)),
        );
    }
}

fn summarize_tasks<'a>(
    tasks: impl Iterator<Item = &'a TaskStatus>,
    now: u64,
    agent_filter: Option<SourceId>,
) -> HookSummary {
    let mut summary = HookSummary {
        state: AgentState::Idle,
        active_task_count: 0,
        has_stale: false,
        stale_task_count: 0,
        highest_priority_task: None,
        updated_at: now,
    };
    let mut best_priority = 0;

    for task in tasks {
        if agent_filter
            .as_ref()
            .is_some_and(|agent| agent != &task.agent)
        {
            continue;
        }
        if task.stale {
            summary.has_stale = true;
            summary.stale_task_count += 1;
            continue;
        }
        if !task.summary_eligible {
            continue;
        }

        let priority = state_priority(&task.state);
        if priority > 0 {
            summary.active_task_count += 1;
        }
        if priority > best_priority {
            best_priority = priority;
            summary.state = task.state.clone();
            summary.highest_priority_task = Some(task.task_key.clone());
        }
    }

    summary
}

fn state_priority(state: &AgentState) -> u8 {
    match state {
        AgentState::Idle => 0,
        AgentState::Done => 1,
        AgentState::Working => 2,
        AgentState::Waiting => 3,
        AgentState::Error => 4,
    }
}

fn parse_task_key(task_key: &str) -> (SourceId, String, bool) {
    let agent = if task_key.to_ascii_lowercase().starts_with("claude_") {
        SourceId::Claude
    } else {
        SourceId::Codex
    };
    let session_id = task_key
        .split_once('_')
        .map(|(_, session_id)| session_id.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let eligible = session_id != "unknown";

    (agent, session_id, eligible)
}

fn safe_key_part(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn sanitize_message(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_control())
        .take(MAX_MESSAGE_CHARS)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(agent: SourceId, key: &str) -> TaskStatus {
        TaskStatus {
            agent,
            task_key: key.to_string(),
            session_id: key.to_string(),
            session_id_source: "test".to_string(),
            state: AgentState::Working,
            updated_at: 10,
            event_order: 10,
            event_order_source: "test".to_string(),
            hook_name: "PreToolUse".to_string(),
            message: None,
            summary_eligible: true,
            stale: false,
        }
    }

    #[test]
    fn removing_claude_tasks_preserves_codex_tasks() {
        let mut state = HookMonitorState::default_at(10);
        state.tasks.insert("codex_a".to_string(), task(SourceId::Codex, "codex_a"));
        state.tasks.insert("claude_b".to_string(), task(SourceId::Claude, "claude_b"));

        remove_tasks_for_agent(&mut state, SourceId::Claude);

        assert!(state.tasks.contains_key("codex_a"));
        assert!(!state.tasks.contains_key("claude_b"));
    }
}

struct StateMutexGuard {
    handle: HANDLE,
}

impl StateMutexGuard {
    fn acquire(timeout_ms: u32) -> io::Result<Self> {
        let handle = unsafe { CreateMutexW(None, false, w!("Local\\CcTrafficLightStateMutex")) }
            .map_err(io::Error::other)?;
        let wait = unsafe {
            WaitForSingleObject(
                handle,
                if timeout_ms == 0 {
                    INFINITE
                } else {
                    timeout_ms
                },
            )
        };

        if wait == WAIT_OBJECT_0 || wait == WAIT_ABANDONED {
            Ok(Self { handle })
        } else {
            unsafe {
                let _ = CloseHandle(handle);
            }
            if wait == WAIT_TIMEOUT {
                Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "state mutex wait timed out",
                ))
            } else {
                Err(io::Error::other(format!(
                    "state mutex wait failed: {wait:?}"
                )))
            }
        }
    }
}

impl Drop for StateMutexGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseMutex(self.handle);
            let _ = CloseHandle(self.handle);
        }
    }
}
