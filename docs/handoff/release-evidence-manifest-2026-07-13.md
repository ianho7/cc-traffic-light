# Release evidence manifest — 2026-07-13

## Verified build and runtime evidence

| Area | Result | Evidence |
|---|---|---|
| Rust workspace | PASS | `cargo test --workspace --offline` on 2026-07-13 (workspace tests passed) |
| Frontend | PASS | `pnpm build` on 2026-07-13; Vite production assets generated |
| Release packages | PASS | Separate `cargo build -p taskbar-settings-tauri --release --offline`, then `cargo build -p taskbar-widget --release --offline` |
| Settings lifecycle | PASS | `taskbar-widget/target/validate-tauri-settings-lifecycle/report.json`; debug and release verifier passed |
| Installed Settings IPC | PASS | Installed Monitoring page showed `hooks deployed successfully`, then `检测刷新已请求`; Codex displayed ready/confirmed and diagnostics `attached`, no recent error |
| Explorer recovery | PASS | `%TEMP%\cc-traffic-light-explorer-recovery-20260713-163739.log`; `WM_NCDESTROY → new HWND → tray retry → attach`, same PID and host_count=1 |
| Normal motion behavior | PASS (human observation) | `docs/handoff/2026-07-13-1608.md`: user confirmed Working, Waiting, Error, Done, Idle and dual-source behavior |
| Claude support | LIMITED ACTIVE | Project-level `shell: powershell` worked for SessionStart/UserPromptSubmit/PreToolUse/PostToolUse/Stop; product default remains ProcessOnly and does not promise other forms |

## Accepted limitations and open evidence

| Item | Status | Owner / next action |
|---|---|---|
| RLS-6-01 desktop screenshots or recording | ACCEPTED LIMITATION | User explicitly waived final normal-motion image/recording artifacts. Existing human observation remains documented but is not represented as image evidence. |
| Reduced-motion desktop recording | ACCEPTED LIMITATION | User explicitly waived; UI persistence/config reload/renderer test retained. |
| Installer, upgrade, uninstall, isolated account | ACCEPTED LIMITATION | User explicitly waived; no installer evidence is claimed. |
| Formatting baseline | ACCEPTED LIMITATION | `cargo fmt --all -- --check` reports pre-existing dirty-file differences. No formatter was run to avoid overwriting user work. |

## Cleanup and safety

- Removed `%TEMP%\cc-traffic-light-claude-hooks` dump-only experiment files after recording the shape-only result.
- Preserved Explorer recovery and state-write failure evidence paths named above.
- Working tree remains intentionally dirty; do not stage/commit broad existing changes without a separate review.
