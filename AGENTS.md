# Repository Guidelines

## Project Structure & Module Organization

This repository is centered on a Win11 taskbar proof of concept in `taskbar-widget/`.

- `taskbar-widget/src/main.rs` owns process startup, window creation, painting, and the message loop.
- `taskbar-widget/src/taskbar.rs` owns taskbar probing, `SetParent`, positioning, layout diagnostics, and JSON output.
- `taskbar-widget/src/win32.rs` contains small Win32 helpers for logging, DPI, HWND, and RECT formatting.
- `taskbar-widget/scripts/diagnose-taskbar-loop.ps1` runs focused taskbar visibility diagnostics.
- `docs/plan/`, `docs/checklist/`, `docs/reflections/`, and `docs/handoff/` hold planning, execution checklists, task reflections, and session handoffs.

Build artifacts live under `taskbar-widget/target/` and should not be treated as source.

## Build, Test, and Development Commands

Run commands from `taskbar-widget/` unless noted otherwise.

```powershell
cargo check
```

Checks Rust code quickly without producing a runnable binary.

```powershell
cargo build
cargo run
```

Builds or runs the Win32 taskbar MVP. `cargo run` requires a Windows desktop session for meaningful manual verification.

```powershell
.\scripts\diagnose-taskbar-loop.ps1 -SkipBuild -Parents shell -Anchors tray_notify -CoordModes rect_delta
```

Runs the focused Win11 diagnostic path and writes output under `target/diagnose-taskbar-loop/`.

## Coding Style & Naming Conventions

Use Rust 2024 conventions and keep the current simple module split. Prefer clear snake_case function names such as `probe_taskbar`, `attach_to_taskbar`, and `position_in_taskbar`. Keep Win32 wrapper helpers in `win32.rs`; keep taskbar-specific policy in `taskbar.rs`. Use `cargo fmt` before larger Rust changes.

## Testing Guidelines

There is no formal test suite yet. Minimum validation is `cargo check`, followed by manual `cargo run` on Win11. For taskbar visibility work, record both logs and human observations; `PrintWindow` or screen captures are diagnostic aids, not final proof of user-visible success.

## Commit & Pull Request Guidelines

Current git history is not reliably readable from this workspace, so no established commit convention was confirmed. Use short imperative commits with scope, for example `taskbar: add dpi diagnostics`. Pull requests should include purpose, changed files, validation commands, manual visibility result, and links to relevant docs or checklist items.

## Agent-Specific Instructions

Keep changes narrow and evidence-driven. Do not expand into classic taskbar, multi-monitor, transparency, D2D, or general plugin architecture unless the active checklist explicitly asks for it. Update `docs/reflections/` or `docs/handoff/` when a debugging turn changes the diagnosis or next-step recommendation.
