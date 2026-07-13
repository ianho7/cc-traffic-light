use std::{
    env, io,
    io::{Read, Write},
    process::{self, ExitCode},
};

use serde_json::{Map, Value};
use taskbar_widget::agent_state::{self, AgentState, HookEventUpdate};
use taskbar_widget::hook_rules;
use taskbar_widget::ui_state::SourceId;

fn main() -> ExitCode {
    match run() {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("taskbar_widget_hook: {error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<String, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();

    // --version must be checked before any subcommand parsing
    if args.first().map(String::as_str) == Some("--version") {
        return Ok(env!("CARGO_PKG_VERSION").to_string());
    }

    let Some(command) = args.first().map(String::as_str) else {
        return Err(usage());
    };

    match command {
        "sample" => sample_payload(),
        "set" => debug_set(&args),
        "clear" => debug_clear(&args),
        "clear-all" => debug_clear_all(&args),
        "list" => debug_list(),
        _ => handle_hook(&args),
    }
}

fn sample_payload() -> Result<String, String> {
    let input = read_stdin().map_err(|error| error.to_string())?;
    let value = parse_json_input(&input, false)?;
    let shape = hook_rules::payload_sample_report(&value);
    serde_json::to_string_pretty(&shape).map_err(|error| error.to_string())
}

fn debug_set(args: &[String]) -> Result<String, String> {
    if args.len() != 3 {
        return Err(
            "usage: taskbar_widget_hook set <task_key> <idle|working|done|waiting|error>"
                .to_string(),
        );
    }
    let state = AgentState::parse(&args[2]).ok_or_else(|| "invalid state".to_string())?;
    let state = agent_state::debug_set_task(&args[1], state).map_err(|error| error.to_string())?;
    serde_json::to_string_pretty(&state).map_err(|error| error.to_string())
}

fn debug_clear(args: &[String]) -> Result<String, String> {
    if args.len() != 2 {
        return Err("usage: taskbar_widget_hook clear <task_key>".to_string());
    }
    let state = agent_state::debug_clear_task(&args[1]).map_err(|error| error.to_string())?;
    serde_json::to_string_pretty(&state).map_err(|error| error.to_string())
}

fn debug_clear_all(args: &[String]) -> Result<String, String> {
    if args.len() != 1 {
        return Err("usage: taskbar_widget_hook clear-all".to_string());
    }
    let state = agent_state::update_state(|state| {
        state.tasks.clear();
    })
    .map_err(|error| error.to_string())?;
    serde_json::to_string_pretty(&state).map_err(|error| error.to_string())
}

fn debug_list() -> Result<String, String> {
    let state = agent_state::load_state_for_display();
    serde_json::to_string_pretty(&state).map_err(|error| error.to_string())
}

fn handle_hook(args: &[String]) -> Result<String, String> {
    if args.len() < 2 {
        return Err(usage());
    }

    let agent = SourceId::parse(&args[0]).ok_or_else(|| "invalid agent".to_string())?;
    let argv_hook_name = args[1].clone();
    let input = read_stdin().map_err(|error| error.to_string())?;
    let received_at = agent_state::now_ms();
    let payload = parse_json_input(&input, true)?;
    let hook_name = hook_rules::resolve_hook_name(&payload, argv_hook_name);
    let session_id = hook_rules::extract_session_id(&payload);
    let event_order = hook_rules::extract_event_order(&payload).unwrap_or(received_at);
    let event_order_source = if event_order == received_at {
        "received_at"
    } else {
        "payload"
    }
    .to_string();

    let state = hook_rules::infer_state(&hook_name);
    let update = HookEventUpdate {
        agent: agent.clone(),
        session_id: session_id.clone(),
        session_id_source: if session_id.is_some() {
            "payload".to_string()
        } else {
            "missing_session_id".to_string()
        },
        hook_name: hook_name.clone(),
        state,
        event_order,
        event_order_source,
        message: Some(format!("hook={hook_name}")),
    };

    agent_state::apply_hook_event(update).map_err(|error| error.to_string())?;
    // Codex treats a successful command hook with no stdout as success.  Do
    // not emit an empty JSON object here: some Windows hook runners close the
    // stdout pipe after delivering stdin, and `println!` would then panic
    // after the state write has already succeeded.
    Ok(String::new())
}

fn read_stdin() -> io::Result<String> {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;
    decode_stdin_bytes(&input)
}

fn decode_stdin_bytes(input: &[u8]) -> io::Result<String> {
    if input.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8(input[3..].to_vec())
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
    }
    if input.starts_with(&[0xFF, 0xFE]) {
        return decode_utf16_le(&input[2..]);
    }
    if input.len() >= 2 && input.iter().skip(1).step_by(2).any(|byte| *byte == 0) {
        return decode_utf16_le(input);
    }
    String::from_utf8(input.to_vec())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn decode_utf16_le(input: &[u8]) -> io::Result<String> {
    let mut chunks = input.chunks_exact(2);
    let units = chunks
        .by_ref()
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    if !chunks.remainder().is_empty() {
        io::stderr().flush().ok();
    }
    String::from_utf16(&units).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn parse_json_input(input: &str, allow_empty: bool) -> Result<Value, String> {
    if input.trim().is_empty() {
        if allow_empty {
            return Ok(Value::Object(Map::new()));
        }
        return Err("empty sample input".to_string());
    }
    serde_json::from_str(input).map_err(|error| format!("invalid json: {error}"))
}

fn usage() -> String {
    format!(
        "usage: taskbar_widget_hook <codex|claude> <HookName> | --version | sample | set <task_key> <state> | clear <task_key> | clear-all | list (pid={})",
        process::id()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_plain_utf8() {
        assert_eq!(
            decode_stdin_bytes(br#"{"session_id":"plain"}"#).unwrap(),
            r#"{"session_id":"plain"}"#
        );
    }

    #[test]
    fn strips_utf8_bom() {
        assert_eq!(
            decode_stdin_bytes(&[0xEF, 0xBB, 0xBF, b'{', b'}']).unwrap(),
            "{}"
        );
    }

    #[test]
    fn decodes_utf16_le_with_bom() {
        let mut input = vec![0xFF, 0xFE];
        for unit in "{}".encode_utf16() {
            input.extend_from_slice(&unit.to_le_bytes());
        }

        assert_eq!(decode_stdin_bytes(&input).unwrap(), "{}");
    }

    #[test]
    fn empty_input_is_allowed_for_hook_but_not_sampling() {
        assert_eq!(
            parse_json_input("", true).unwrap(),
            Value::Object(Map::new())
        );
        assert!(parse_json_input("", false).is_err());
    }
}
