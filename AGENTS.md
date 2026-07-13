# Repository Guidelines

## Project Structure & Module Organization

This repository is now a mixed Rust plus PNPM workspace for a Win32 taskbar host with a standalone Tauri settings app.

For the maintained source and documentation map, see [FILETREE.md](FILETREE.md).

- `taskbar-widget/` is the native host crate. It owns the widget window, tray, detector loop, fallback settings paths, and the host-side named-pipe server.
- `taskbar-settings-tauri/` is the React plus Tauri settings application. `src/` holds the frontend UI and `src-tauri/` holds the Rust backend crate.
- `crates/shared-core/` is the shared Rust business layer for config models, snapshot DTOs, settings services, and IPC contract types.
- `docs/plan/`, `docs/checklist/`, `docs/design/`, `docs/references/`, `docs/ui/`, `docs/reflections/`, and `docs/handoff/` track architecture, execution state, design/reference material, reflections, and session handoffs.

Key host entry points:

- `taskbar-widget/src/main.rs` owns process startup, host window creation, settings infrastructure binding, painting, and the message loop.
- `taskbar-widget/src/settings_process.rs` owns Tauri settings process launch, reuse, fallback, and shutdown behavior.
- `taskbar-widget/src/settings_bridge.rs` owns the host-side settings facade used by fallback UIs and Tauri IPC.
- `taskbar-widget/src/tauri_settings_ipc.rs` owns the named-pipe server for Tauri settings commands.
- `taskbar-widget/src/taskbar.rs` owns taskbar probing, `SetParent`, positioning, layout diagnostics, and attach/recovery support.
- `taskbar-widget/src/widget_effects.rs` owns blink timing and reduced-motion behavior; `taskbar-widget/src/widget_render.rs` owns lamp rendering.
- `taskbar-widget/src/win32.rs` contains small Win32 helpers for logging, DPI, HWND, and RECT formatting.

Build artifacts live under the root `target/` directory and `taskbar-settings-tauri/dist/`; they should not be treated as source.

## Build, Test, and Development Commands

Run commands from the repository root `D:\project\cc-traffic-light` unless a script explicitly says otherwise.

```powershell
cargo check -p taskbar-widget --offline
```

Checks the native host crate quickly without producing a runnable binary.

```powershell
cargo test --workspace --offline
```

Runs the current Rust test set for `shared-core`, the Tauri backend crate, and the host workspace members.

```powershell
pnpm build
```

Builds the Tauri settings frontend assets under `taskbar-settings-tauri/dist/`.

```powershell
cargo build -p taskbar-settings-tauri --release --offline
cargo build -p taskbar-widget --release --offline
```

Builds the standalone settings process first, then rebuilds the host as the final release validation artifact. The same ordering applies to debug builds.

```powershell
.\taskbar-widget\scripts\validate-tauri-settings-lifecycle.ps1 -Configuration debug -TimeoutSeconds 20
```

Runs the end-to-end lifecycle check for spawning, reusing, closing, reopening, and recovering the Tauri settings process.

Important build constraint:

- Do not use `cargo build --workspace` as the host acceptance build for `taskbar-widget.exe`.
- The workspace-wide build can unify Tauri-side features back into the host dependency graph and reintroduce the `TaskDialogIndirect` loader failure.
- When validating the host executable, keep `taskbar-widget` as the last separately built package.
- The 2026-07-13 release gate passed workspace tests, `pnpm build`, separate release builds, Settings lifecycle checks, and Explorer restart recovery. Installer/upgrade/uninstall and some desktop recording evidence remain explicitly waived.

## Coding Style & Naming Conventions

Use Rust 2024 conventions and keep the current boundary split intact.

- Prefer clear snake_case function names such as `probe_taskbar`, `open_or_focus_tauri_settings`, and `request_manual_refresh`.
- Keep Win32 wrapper helpers in `win32.rs`.
- Keep taskbar-specific policy in `taskbar.rs`.
- Keep settings process lifecycle in `settings_process.rs`.
- Keep shared business types and IPC DTOs in `crates/shared-core/`; do not leak Win32 handles or Tauri runtime types into that crate.

Use `cargo fmt` before larger Rust changes. Keep frontend naming consistent with the existing settings page and DTO terminology.

## Testing Guidelines

There is no large formal end-to-end test suite, so validation is layered:

1. `cargo check -p taskbar-widget --offline`
2. `cargo test --workspace --offline`
3. `pnpm build`
4. targeted `cargo build -p ... --offline`
5. manual or scripted Windows desktop verification

For taskbar visibility or lifecycle work, record both logs and observable behavior. `PrintWindow`, screenshots, and runtime logs are diagnostics, not final proof by themselves.

When touching settings lifecycle or monitoring behavior, prefer updating or reusing:

- `taskbar-widget/scripts/validate-tauri-settings-lifecycle.ps1`
- `docs/checklist/tauri-settings-migration.md`
- `docs/checklist/remaining-release-readiness-2026-07-13.md`
- `docs/handoff/2026-07-13-1608.md`
- `docs/handoff/release-evidence-manifest-2026-07-13.md`

## Commit & Pull Request Guidelines

Use short imperative commits with scope, for example `settings: harden tauri lifecycle validation`.

Pull requests should include:

- purpose
- changed files
- validation commands
- manual Windows verification result when relevant
- links to the active checklist or handoff docs

## Agent-Specific Instructions

Keep changes narrow and evidence-driven.

- Do not expand Tauri migration into rewriting the widget, taskbar attach path, or detector main loop unless the active checklist explicitly asks for it.
- Preserve the current architecture boundary: Win32 host plus standalone Tauri settings plus `shared-core`.
- Update `docs/reflections/` or `docs/handoff/` when a debugging turn changes the diagnosis, build constraint, or next-step recommendation.
- If a task touches settings lifecycle validation, be explicit about which executable path was validated: root `target\debug\taskbar-widget.exe` / `target\release\taskbar-widget.exe` versus any stale per-crate artifact.
- Do not describe Claude as fully Active unless the current environment has evidence for the exact hook form being documented; the product default is currently `ProcessOnly`.
- Do not treat user-waived installer or desktop-recording evidence as a passing verification result.
