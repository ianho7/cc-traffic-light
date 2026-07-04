# FILETREE

## (root)/

- `AGENTS.md`: Contributor guide for workspace layout, build commands, validation expectations, and agent constraints.
- `FILETREE.md`: Workspace structure manifest for the Win32 host, Tauri settings app, shared core, and migration docs.
- `.gitignore`: Ignore rules for build output, package-manager caches, screenshots, and local diagnostics.
- `Cargo.toml`: Root Cargo workspace manifest for `shared-core`, `taskbar-settings-tauri`, and `taskbar-widget`.
- `Cargo.lock`: Locked Rust dependency graph for the whole workspace.
- `package.json`: Root PNPM scripts for building and running the Tauri settings frontend and app shell.
- `pnpm-workspace.yaml`: PNPM workspace definition that includes the Tauri settings frontend package.
- `pnpm-lock.yaml`: Locked PNPM dependency graph for the frontend workspace.

## .claude/

- `settings.local.json`: Project-local Claude Code hook configuration that writes shared task state through `taskbar_widget_hook.exe`.

## .claude/hook-logs/

- `*.jsonl`: Real Claude hook samples captured for payload-shape verification and field-path evidence.

## .claude/hooks/

- `sample-hook.ps1`: Shape-only Claude hook sampler that records redacted payload structure for integration debugging.

## .codex/

- `hooks.json`: Project-local Codex lifecycle hook configuration that writes shared task state through `taskbar_widget_hook.exe`.

## crates/

- `shared-core/`: Shared Rust business layer for config models, snapshot DTOs, settings services, and Tauri IPC contract types.

## crates/shared-core/

- `Cargo.toml`: Rust package manifest for the shared settings/config core crate.

## docs/

- `checklist/`: Execution checklists for widget, hook, traffic-light UI, and Tauri settings migration work.
- `handoff/`: Session handoffs that capture current diagnosis, build constraints, and next-step recommendations.
- `plan/`: Architecture, migration, and phased implementation plans for the host, hooks, UI, and Tauri integration.
- `reflections/`: Per-task decision logs generated from checklist execution.

## docs/checklist/

- `tauri-settings-migration.md`: Active execution checklist for the Win32 host plus Tauri settings migration.

## docs/handoff/

- `2026-07-04-0130.md`: Current migration handoff covering the `TaskDialogIndirect` root cause, separate-build constraint, and lifecycle validation result.

## docs/plan/

- `README.md`: Reading guide for the plan set and how the major implementation phases relate to one another.
- `tauri-settings-architecture-baseline.md`: Current architecture baseline for the Win32 host, Tauri settings process, shared core, and build constraints.
- `tauri-settings-ipc-contract.md`: Named-pipe IPC contract for host and Tauri settings commands, envelopes, and DTO boundaries.
- `tauri-settings-visual-fidelity-pass-1.md`: Visual comparison record for the first Tauri settings fidelity pass against the HTML demo baseline.

## docs/reflections/

- `task-TSM-*.md`: Tauri settings migration reflections for architecture, IPC, UI, lifecycle, validation, and documentation decisions.
- `task-SSM-*.md`: Slint settings migration reflections retained as historical context for the Tauri follow-up work.
- `task-*.md`: Earlier widget, hook, installer, and traffic-light UI reflections retained as project history.

## target/

- `debug/`: Root workspace Rust build output, including the runnable host and Tauri settings executables.

## taskbar-settings-tauri/

- `package.json`: PNPM package manifest for the React frontend and Tauri shell commands.
- `index.html`: Vite HTML entry for the settings frontend.
- `src/`: React settings UI, page structure, polling logic, and frontend DTO rendering.
- `src-tauri/`: Tauri Rust backend that bridges frontend commands to the host named pipe.
- `dist/`: Built frontend assets produced by `pnpm build`.

## taskbar-settings-tauri/src-tauri/

- `Cargo.toml`: Rust package manifest for the Tauri backend crate in the workspace.
- `build.rs`: Tauri build-script entry for generated config and resources.
- `tauri.conf.json`: Tauri app configuration for the standalone settings process.

## taskbar-widget/

- `Cargo.toml`: Rust package manifest for the Win32 host, tray integration, and fallback settings components.
- `README.md`: Host-focused project overview, diagnostics guidance, and runtime notes.
- `app.manifest`: Embedded Common Controls v6 manifest for the host executable.
- `build.rs`: Host build script that embeds the application manifest during MSVC linking.
- `examples.codex-hooks.toml`: Example Codex lifecycle hook configuration for feeding shared task state into the widget.
- `examples.claude-hooks.json`: Example Claude Code hooks configuration for feeding shared task state into the widget.
- `scripts/`: PowerShell diagnostics and lifecycle validation scripts for the host and settings process.
- `src/`: Win32 host, tray, detector, fallback settings, shared-state, and IPC server code.

## taskbar-widget/scripts/

- `diagnose-taskbar-loop.ps1`: Focused Win11 taskbar visibility diagnosis loop for parent, anchor, coordinate, and render evidence.
- `diagnose-widget-liveness.ps1`: Widget lifecycle and redraw diagnosis harness for runtime visibility and repaint regressions.
- `validate-tauri-settings-lifecycle.ps1`: End-to-end lifecycle validator for spawning, reusing, closing, and recovering the Tauri settings process.
- `codex-lifecycle-hook-dump.ps1`: Codex lifecycle hook dump script for redacted shape-only payload sampling.
- `codex-notify-probe-config.ps1`: Helper that prepares or inspects Codex notify probe configuration during low-fidelity notify experiments.
- `codex-notify-probe-wrapper.ps1`: Wrapper that captures Codex notify probe inputs while forwarding the original notify command shape.
- `install-codex-hooks.ps1`: User-scoped Codex hooks installer for merging managed lifecycle hook entries into `hooks.json`.

## taskbar-widget/src/

- `main.rs`: Starts the host, binds settings infrastructure, paints the widget, and runs the Win32 plus Slint event loop.
- `settings_process.rs`: Manages Tauri settings process launch, focus, reuse, fallback selection, and shutdown bookkeeping.
- `settings_bridge.rs`: Host-side facade for reading snapshots, reading and saving config, applying changes, and issuing refresh requests.
- `tauri_settings_ipc.rs`: Named-pipe server for Tauri settings commands and JSON request/response envelopes.
- `settings_slint.rs`: Slint settings host retained as an explicit fallback while Tauri migration is still hardening.
- `settings_window.rs`: Win32 fallback settings window and helper commands retained for deep fallback scenarios.
- `app_config.rs`: Host-facing config helpers and compatibility layer around the shared settings model.
- `ui_state.rs`: Runtime snapshot model and source-status projection used by the widget and settings surfaces.
- `detector.rs`: Polling logic for Codex and Claude state detection and runtime refresh behavior.
- `tray_icon.rs`: Tray icon creation, menu wiring, and command dispatch helpers for the host process.
- `taskbar.rs`: Taskbar probing, parent attachment, layered-window setup, placement, diagnostics, and runtime state logging.
- `win32.rs`: Small Win32 helper layer for logging, DPI awareness, HWND formatting, RECT formatting, and error helpers.
- `agent_state.rs`: Shared hook state schema, JSON persistence, stale handling, and summary aggregation for Codex and Claude.
- `hook_rules.rs`: Hook payload field extraction and event-to-state mapping for working, waiting, done, and error transitions.
- `runtime_contract.rs`: Shared runtime DTOs and compatibility helpers used between host layers and UI surfaces.
- `i18n.rs`: Localization loader and label mapping for tray, fallback settings, and status text.
- `autostart.rs`: Windows autostart registration helpers for settings-controlled launch behavior.
- `lib.rs`: Library entry that exposes reusable host modules for the widget binary and hook CLI.

## taskbar-widget/src/bin/

- `taskbar_widget_hook.rs`: Hook CLI for payload sampling, shared-state writes, and debug `set` or `clear` commands.
