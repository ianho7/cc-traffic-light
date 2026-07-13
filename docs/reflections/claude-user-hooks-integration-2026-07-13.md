# Claude Code user-level hooks integration

- Decision: deploy CC Traffic Light Claude Code command hooks into `%USERPROFILE%\.claude\settings.json`, the official user scope that applies across all projects.
- Safety: the installer merges only entries identified by the `CcTrafficLight Claude` status-message prefix, preserves unrelated settings and hook entries, and saves an original-file backup plus metadata for restore.
- Runtime path: `taskbar_widget_hook.exe claude <event>` continues to write the existing shared `%APPDATA%\CcTrafficLight\state.json`; the widget and Tauri snapshot paths require no new state transport.
- UI: the Monitoring page can invoke a dedicated host IPC command to deploy Claude hooks and then refresh the reported hook status.
- Validation: isolated apply/restore fixture preserved an unrelated `theme` key and a pre-existing `Stop` hook; `cargo test --workspace --offline` and `pnpm build` passed.
- Remaining evidence: validate one interactive Claude Code session against the installed release binary, confirm `/status` loads user settings, then record a real `claude_<session_id>` state entry before claiming `Active`.
