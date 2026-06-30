# FILETREE

_Manual fallback manifest. The filetree script could not apply because this workspace has nested Git repositories._

## (root)/

- `AGENTS.md`: Contributor guide for repository structure, commands, style, validation, and agent-specific constraints.
- `FILETREE.md`: Human-readable map of key source, script, and documentation files in this workspace.

## taskbar-widget/

- `Cargo.toml`: Rust package manifest for the Win11 taskbar MVP and its `windows` crate feature set.
- `Cargo.lock`: Locked Rust dependency graph for reproducible local builds.
- `README.md`: Project overview, runtime strategy, commands, environment notes, and current limitations.

## taskbar-widget/src/

- `main.rs`: Starts the Win32 MVP, creates the self-painted window, attaches it to the taskbar, and runs the message loop.
- `taskbar.rs`: Probes Win11 taskbar windows, attaches the module, computes placement, logs runtime state, and writes diagnostics JSON.
- `win32.rs`: Small Win32 helper layer for logging, DPI awareness, HWND formatting, RECT formatting, and window rectangle lookup.

## taskbar-widget/scripts/

- `diagnose-taskbar-loop.ps1`: PowerShell diagnosis loop for parent, anchor, coordinate, render, and visibility evidence collection.

## docs/plan/mvp-startup/

- `README.md`: Reading guide for the MVP startup planning documents and current Rust taskbar context.
- `01-mvp-plan.md`: Baseline MVP-first plan for the original Rust taskbar proof of concept.
- `02-scope-and-decisions.md`: Scope boundaries, design decisions, and excluded product features for the MVP.
- `03-project-bootstrap.md`: Bootstrap notes for creating and structuring the Rust MVP project.
- `04-implementation-phases.md`: Phase plan for implementing the MVP from window creation through taskbar embedding.
- `05-file-layout.md`: Planned Rust file layout and responsibilities for `main`, `taskbar`, and `win32` modules.
- `06-validation-and-debugging.md`: Validation approach and debugging guidance for taskbar embedding behavior.
- `07-risks-and-watchlist.md`: Risk list and over-engineering watchlist for Win11 taskbar integration.
- `08-trafficmonitor-reference-map.md`: Reference map to TrafficMonitor docs and implementation files used for comparison.
- `09-win11-diagnosis-replan.md`: Current MVP replan after Win11 diagnosis; defines the active taskbar visibility strategy.

## docs/checklist/

- `taskbar-phase4-printwindow.png`: Diagnostic image artifact from a PrintWindow-based taskbar capture.
- `taskbar-visibility-diagnosis-loop.md`: Diagnosis loop notes for taskbar visibility failures and evidence collection.
- `win11-taskbar-widget-checklist.md`: Original implementation checklist for building the Win11 Rust taskbar MVP.
- `win11-taskbar-widget-loop-spec.md`: Loop specification for executing the original Win11 Rust taskbar checklist.
- `win11-taskbar-widget-preflight.md`: Environment and prerequisite checks for the Rust taskbar MVP.
- `win11-taskbar-runtime-map.md`: Runtime map of discovered taskbar windows, handles, and current embedding choices.
- `win11-taskbar-visibility-replan-checklist.md`: Active checklist for fixing Win11 taskbar visibility after diagnosis.
- `win11-taskbar-visibility-replan-loop-spec.md`: Loop rules and stop conditions for the visibility replan checklist.

## docs/handoff/

- `2026-06-30-1127.md`: Session handoff explaining current implementation, DPI fix, visibility contradiction, and next debugging step.

## docs/reflections/

- `task-PRE-*.md`: Preflight task reflections recording environment checks, scope confirmations, and setup decisions.
- `task-P1-*.md`: Phase 1 reflections covering project setup, host path selection, and taskbar probing.
- `task-P2-*.md`: Phase 2 reflections covering window creation, style choices, attachment behavior, and early validation.
- `task-P3-*.md`: Phase 3 reflections covering state modeling, positioning, and layout decisions.
- `task-P4-*.md`: Phase 4 reflections covering diagnostic scripts, captures, and runtime evidence.
- `task-P5-*.md`: Phase 5 reflections covering documentation, constraints, and deferred scope decisions.
- `task-P6-*.md`: Phase 6 reflections covering cleanup, runtime maps, and finalization tasks.
- `task-VAL-*.md`: Validation reflections recording build, run, visibility, and repeatability checks.
- `task-DOC-*.md`: Documentation reflections for README, runtime map, and related project docs.
- `task-CLN-*.md`: Cleanup reflections for logs, screenshots, structure, and naming consistency.
