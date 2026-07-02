use serde_json::{Map, Value, json};

use crate::agent_state::AgentState;

const HOOK_NAME_KEYS: &[&str] = &["hook_event_name", "hookName", "eventName", "hook_name"];
const SESSION_ID_KEYS: &[&str] = &["session_id", "sessionId", "sessionID"];
const TURN_ID_KEYS: &[&str] = &["turn_id", "turnId", "turnID"];
const EVENT_ORDER_KEYS: &[&str] = &[
    "event_order",
    "eventOrder",
    "timestamp",
    "created_at",
    "createdAt",
    "time",
];

pub fn payload_sample_report(value: &Value) -> Value {
    json!({
        "mode": "sample",
        "privacy": "values_redacted_shape_only",
        "hook_name_candidates": find_key_paths(value, HOOK_NAME_KEYS),
        "session_id_candidates": find_key_paths(value, SESSION_ID_KEYS),
        "turn_id_candidates": find_key_paths(value, TURN_ID_KEYS),
        "event_order_candidates": find_key_paths(value, EVENT_ORDER_KEYS),
        "shape": payload_shape(value),
    })
}

pub fn resolve_hook_name(payload: &Value, argv_hook_name: String) -> String {
    find_string_value(payload, HOOK_NAME_KEYS).unwrap_or(argv_hook_name)
}

pub fn extract_session_id(payload: &Value) -> Option<String> {
    find_string_value(payload, SESSION_ID_KEYS)
}

pub fn extract_event_order(payload: &Value) -> Option<u64> {
    find_u64_value(payload, EVENT_ORDER_KEYS)
}

pub fn infer_state(hook_name: &str, _previous: Option<&AgentState>, payload: &Value) -> AgentState {
    let text = collect_text_for_heuristics(payload);
    match hook_name.to_ascii_lowercase().as_str() {
        "sessionstart" => AgentState::Idle,
        "userpromptsubmit" | "pretooluse" | "posttooluse" | "precompact" | "postcompact"
        | "subagentstart" => AgentState::Working,
        "permissionrequest" | "notification" => AgentState::Waiting,
        "stopfailure" | "posttoolusefailure" | "toolusefailure" => AgentState::Error,
        "subagentstop" => AgentState::Done,
        "stop" => {
            if looks_error(&text) {
                AgentState::Error
            } else {
                AgentState::Done
            }
        }
        _ => AgentState::Working,
    }
}

fn looks_error(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        "error",
        "failed",
        "failure",
        "cannot",
        "can't",
        "unable",
        "not found",
        "does not exist",
        "no such file",
        "permission denied",
        "denied",
        "refused",
        "timed out",
        "exception",
        "不存在",
        "未找到",
        "失败",
        "无法",
        "错误",
        "拒绝",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn find_string_value(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if keys.iter().any(|candidate| key == candidate) {
                    if let Some(value) = child.as_str() {
                        return Some(value.to_string());
                    }
                    if let Some(value) = child.as_u64() {
                        return Some(value.to_string());
                    }
                }
                if let Some(found) = find_string_value(child, keys) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(items) => items.iter().find_map(|item| find_string_value(item, keys)),
        _ => None,
    }
}

fn find_u64_value(value: &Value, keys: &[&str]) -> Option<u64> {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if keys.iter().any(|candidate| key == candidate) {
                    if let Some(value) = child.as_u64() {
                        return Some(value);
                    }
                    if let Some(value) = child.as_str().and_then(parse_u64_string) {
                        return Some(value);
                    }
                }
                if let Some(found) = find_u64_value(child, keys) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(items) => items.iter().find_map(|item| find_u64_value(item, keys)),
        _ => None,
    }
}

fn parse_u64_string(value: &str) -> Option<u64> {
    value.trim().parse::<u64>().ok()
}

fn collect_text_for_heuristics(value: &Value) -> String {
    let mut parts = Vec::new();
    collect_text_values(value, &mut parts);
    parts.join(" ")
}

fn collect_text_values(value: &Value, parts: &mut Vec<String>) {
    match value {
        Value::String(value) => {
            if parts.len() < 16 {
                parts.push(value.chars().take(160).collect());
            }
        }
        Value::Object(map) => {
            for child in map.values() {
                collect_text_values(child, parts);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_text_values(item, parts);
            }
        }
        _ => {}
    }
}

fn find_key_paths(value: &Value, keys: &[&str]) -> Vec<String> {
    let mut paths = Vec::new();
    collect_key_paths(value, keys, "$", &mut paths);
    paths
}

fn collect_key_paths(value: &Value, keys: &[&str], prefix: &str, paths: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let next = format!("{prefix}.{key}");
                if keys.iter().any(|candidate| key == candidate) {
                    paths.push(next.clone());
                }
                collect_key_paths(child, keys, &next, paths);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_key_paths(child, keys, &format!("{prefix}[{index}]"), paths);
            }
        }
        _ => {}
    }
}

fn payload_shape(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut output = Map::new();
            for (key, child) in map {
                output.insert(key.clone(), payload_shape(child));
            }
            Value::Object(output)
        }
        Value::Array(items) => {
            let shape = items
                .first()
                .map(payload_shape)
                .unwrap_or(Value::String("empty_array".to_string()));
            json!({ "type": "array", "item": shape })
        }
        Value::String(_) => json!({ "type": "string", "value": "<redacted>" }),
        Value::Number(_) => json!({ "type": "number", "value": "<redacted>" }),
        Value::Bool(_) => json!({ "type": "boolean", "value": "<redacted>" }),
        Value::Null => json!({ "type": "null" }),
    }
}
