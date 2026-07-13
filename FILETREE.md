# FILETREE

This is a maintained map of source, entry points, validation scripts, and project documentation. Build output and dependency caches are intentionally omitted.

## Root

- `AGENTS.md`: Repository and agent working rules.
- `README.md`: Product overview, architecture, state semantics, build order, and current release limitations.
- `FILETREE.md`: This source and documentation map.
- `Cargo.toml` / `Cargo.lock`: Rust workspace manifest and lockfile.
- `package.json` / `pnpm-workspace.yaml` / `pnpm-lock.yaml`: Root PNPM scripts, workspace definition, and lockfile.
- `installer.iss`: Inno Setup installer definition.
- `scripts/rebuild-all.ps1`: Frontend and Rust rebuild orchestration.
- `scripts/pack-all.ps1`: Packaging orchestration.
- `scripts/clear-local-history.ps1`: Local development cleanup helper.
- `reasonix.toml`: Project automation metadata.

## Project configuration

- `.codex/hooks.json`: Codex lifecycle hook configuration.
- `.claude/hooks.json`: Claude Code hook configuration.
- `.claude/settings.json`: Project-local Claude Code settings.
- `.claude/hooks/`: Hook sampling helpers.
- `.claude/hook-logs/`: Captured redacted hook payload samples when present.

## Rust workspace

### `crates/shared-core/`

Shared Rust business and IPC layer. `src/` contains `app_config.rs`, `runtime_contract.rs`, `settings_service.rs`, `tauri_ipc.rs`, `ui_state.rs`, and `lib.rs`.

### `taskbar-widget/`

Native Win32 host, tray icon, taskbar widget, detector loop, hook state persistence, fallback settings window, and host-side named-pipe server.

- `src/main.rs`: Host startup, widget window lifecycle, Explorer recovery, painting, and message loop.
- `src/detector.rs`: Codex/Claude detection and refresh loop.
- `src/agent_state.rs`: Hook state schema, persistence, stale handling, and aggregation.
- `src/hook_rules.rs`: Hook payload extraction and event-to-state mapping.
- `src/settings_process.rs`: Tauri settings launch, reuse, close, recovery, and fallback selection.
- `src/settings_bridge.rs`: Host settings facade.
- `src/tauri_settings_ipc.rs`: Named-pipe IPC server.
- `src/taskbar.rs`: Taskbar probing, attachment, positioning, and diagnostics.
- `src/widget_effects.rs` / `src/widget_render.rs`: Animation/reduced-motion state and lamp rendering.
- `src/tray_icon.rs`, `src/settings_window.rs`, `src/autostart.rs`, `src/i18n.rs`, `src/runtime_contract.rs`, `src/ui_state.rs`, `src/win32.rs`: Host support modules.
- `src/bin/taskbar_widget_hook.rs`: Hook CLI for state writes, payload sampling, and debug commands.
- `resources/`: Runtime logos and application icon.
- `scripts/`: Hook installers/dumps, taskbar diagnostics, named-pipe/read-model checks, and Tauri lifecycle validators.
- `examples.codex-hooks.toml` / `examples.claude-hooks.json`: Example hook configurations.
- `README.md` / `FILETREE.md`: Host-specific documentation.

### `taskbar-settings-tauri/`

Standalone Tauri settings application.

- `src/`: React UI, shared components, pages, DTOs, Tauri client, localization, and styles.
- `src/pages/`: Overview, General, Appearance, Monitoring, and About pages.
- `src/components/`: Layout, navigation, signal, source, status, toggle, primitive, and appearance components.
- `src-tauri/`: Tauri Rust backend, IPC command bridge, capabilities, icons, and app configuration.
- `messages/`: English and Chinese localization messages.
- `scripts/`: Frontend build and settings coverage verification.
- `project.inlang/`: Localization project configuration.

## Documentation

- `docs/plan/`: Architecture and phased implementation plans, including end-to-end monitoring and UI componentization.
- `docs/checklist/`: Active execution and release-readiness checklists.
- `docs/handoff/`: Current handoffs, hook investigations, release gate report, and evidence manifest. The latest handoff is `2026-07-13-1608.md`.
- `docs/reflections/`: Per-task decision and validation records, including the 2026-07-13 release records.
- `docs/design/`: UI design and componentization specifications.
- `docs/ui/`: UI demos and visual checklists.
- `docs/references/`: Hook reference material and automation notes.

## Generated or local-only output

- `target/`: Root Rust build output, including debug/release executables.
- `taskbar-settings-tauri/dist/`: Generated frontend assets.
- `node_modules/`, `.pnpm-store/`, `design-previews/`, and local diagnostic artifacts: dependency caches or generated/local evidence; do not edit as source.
