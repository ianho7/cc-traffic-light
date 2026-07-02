# FILETREE

_Manual fallback manifest. The filetree script could not apply because this workspace has nested Git repositories._

## (root)/

- `AGENTS.md`: Contributor guide for repository layout, commands, style, validation, and agent operating constraints.
- `FILETREE.md`: Human-maintained map of the current workspace structure, key code paths, and project documentation.
- `.gitignore`: Ignore rules for local artifacts, build output, and generated workspace files.

## .codex/

- `hooks.json`: Project-local Codex lifecycle hook configuration that writes shared task state through `taskbar_widget_hook.exe`.

## .claude/

- `settings.local.json`: Project-local Claude Code hooks configuration that writes shared task state through `taskbar_widget_hook.exe`.

## .claude/hooks/

- `sample-hook.ps1`: Shape-only Claude hook sampler that records redacted payload structure for integration debugging.

## .claude/hook-logs/

- `*.jsonl`: Real Claude hook samples captured for payload-shape verification and field-path evidence.

## taskbar-widget/

- `Cargo.toml`: Rust package manifest for the Win11 taskbar widget, hook state integration, and Win32 dependency features.
- `Cargo.lock`: Locked Rust dependency graph for reproducible local builds.
- `README.md`: Project overview, stable runtime path, hook integration notes, diagnostics, limitations, and related docs.
- `examples.codex-hooks.toml`: Example Codex lifecycle hook configuration for feeding shared task state into the widget.
- `examples.claude-hooks.json`: Example Claude Code hooks configuration for feeding shared task state into the widget.

## taskbar-widget/src/

- `main.rs`: Starts the widget, polls shared hook state, paints the traffic-light UI, and runs the Win32 message loop.
- `agent_state.rs`: Shared hook state schema, JSON persistence, stale/TTL handling, and summary aggregation for Codex and Claude.
- `hook_rules.rs`: Hook payload field extraction and event-to-state mapping for working, waiting, done, and error transitions.
- `taskbar.rs`: Taskbar probing, parent attachment, layered-window setup, placement, diagnostics, and runtime state logging.
- `win32.rs`: Small Win32 helper layer for logging, DPI awareness, HWND formatting, RECT formatting, and error helpers.
- `lib.rs`: Library entry that exposes reusable modules for the widget binary and hook CLI.

## taskbar-widget/src/bin/

- `taskbar_widget_hook.rs`: Hook CLI for payload sampling, shared-state writes, and debug `set`/`clear`/`list` commands.

## taskbar-widget/scripts/

- `diagnose-taskbar-loop.ps1`: Focused Win11 taskbar visibility diagnosis loop for parent, anchor, coordinate, and render evidence.
- `diagnose-widget-liveness.ps1`: Widget lifecycle and redraw diagnosis harness for runtime visibility and repaint regressions.
- `codex-lifecycle-hook-dump.ps1`: Codex lifecycle hook dump script for redacted shape-only payload sampling.
- `codex-notify-probe-config.ps1`: Helper that prepares or inspects Codex notify probe configuration during low-fidelity notify experiments.
- `codex-notify-probe-wrapper.ps1`: Wrapper that captures Codex notify probe inputs while forwarding the original notify command shape.
- `install-codex-hooks.ps1`: User-scoped Codex hooks installer for merging managed lifecycle hook entries into `hooks.json`.

## docs/

- `claude-code-hooks-integration.md`: Integration notes, schema examples, and observed behavior for Claude Code hooks.

## docs/plan/

- `README.md`: Reading guide for the plan set and how the major implementation phases relate to one another.

## docs/plan/mvp-startup/

- `README.md`: Reading guide for the original Win11 taskbar MVP startup plan and supporting design notes.
- `01-mvp-plan.md`: Baseline MVP plan for the first Rust taskbar proof of concept.
- `02-scope-and-decisions.md`: Scope boundaries, assumptions, and excluded product work for the MVP.
- `03-project-bootstrap.md`: Bootstrap notes for creating and structuring the Rust taskbar widget project.
- `04-implementation-phases.md`: Phase-by-phase implementation path from window creation through taskbar embedding.
- `05-file-layout.md`: Planned Rust module layout and responsibilities for `main`, `taskbar`, and `win32`.
- `06-validation-and-debugging.md`: Validation strategy and debugging guidance for taskbar visibility and runtime behavior.
- `07-risks-and-watchlist.md`: Risk list and anti-overengineering guidance for the Win11 taskbar path.
- `08-trafficmonitor-reference-map.md`: Reference map to external TrafficMonitor materials used for comparison.
- `09-win11-diagnosis-replan.md`: Replan after early Win11 diagnosis, defining the active visibility-fix strategy.

## docs/plan/hook-integration/

- `README.md`: Reading guide for the hook integration design and adjustment planning documents.
- `01-mvp-plan.md`: MVP-first plan for Codex and Claude hook integration into the shared widget state model.
- `02-grill-decisions-and-adr.md`: Decision record and tradeoff analysis for hook integration architecture.
- `03-hook-adjustment-plan.md`: Follow-up plan for tightening hook semantics, config boundaries, and validation steps.

## docs/plan/p0-codex-state-write/

- `README.md`: Plan for proving Codex lifecycle hooks can write shared state for the widget.

## docs/plan/p1-global-hook-install/

- `README.md`: Plan for installing managed Codex lifecycle hooks at user scope.

## docs/plan/p2-claude-code-hook-validation/

- `README.md`: Plan for validating real Claude Code hooks, payload fields, and shared-state writes.

## docs/plan/p3-taskbar-traffic-light-ui/

- `README.md`: Plan for replacing the text-first widget with a minimal traffic-light taskbar UI.

## docs/plan/p4-runtime-hardening/

- `README.md`: Plan stub for runtime hardening after the traffic-light UI and hook-state path stabilize.

## docs/checklist/

- `win11-taskbar-widget-preflight.md`: Environment and prerequisite checks for the original Rust taskbar widget MVP.
- `win11-taskbar-widget-checklist.md`: Original implementation checklist for building the Win11 taskbar widget MVP.
- `win11-taskbar-widget-loop-spec.md`: Original loop specification for executing the Win11 widget checklist.
- `taskbar-visibility-diagnosis-loop.md`: Diagnosis notes for visibility failures and evidence-driven taskbar debugging.
- `win11-taskbar-runtime-map.md`: Runtime map of discovered taskbar windows, handles, anchors, and embedding choices.
- `win11-taskbar-visibility-replan-checklist.md`: Active checklist for fixing Win11 visibility after early diagnosis.
- `win11-taskbar-visibility-replan-loop-spec.md`: Loop rules and stop conditions for the visibility replan work.
- `taskbar-phase4-printwindow.png`: Diagnostic PrintWindow capture artifact used during early visibility investigations.
- `hook-integration-checklist.md`: Initial checklist for shared hook state integration across Codex and Claude.
- `hook-integration-validation.md`: Validation notes for hook state writes, ordering, and state transitions.
- `hook-adjustment-checklist.md`: Checklist for tightening hook rules, config boundaries, and state semantics.
- `hook-payload-sampling.md`: Evidence log of sampled real hook payload shapes and confirmed field paths.
- `codex-notify-probe.md`: Findings showing Codex notify is low-fidelity and not suitable as the main state path.
- `codex-lifecycle-hooks-validation.md`: Validation record proving real Codex lifecycle hooks provide usable session identity.
- `p0-codex-state-write.md`: Execution checklist for proving Codex lifecycle hooks write shared task state.
- `p1-global-hook-install.md`: Execution checklist for user-scoped Codex hook installation and validation.
- `p2-claude-code-hook-validation.md`: Execution checklist for validating Claude Code hook payloads and state writes.
- `p3-taskbar-traffic-light-ui.md`: Execution checklist for the traffic-light taskbar UI, manual matrix tests, and real-hook smoke tests.

## docs/handoff/

- `2026-06-30-*.md`: Early session handoffs covering taskbar visibility diagnosis, runtime mapping, and MVP path corrections.
- `2026-07-01-*.md`: Mid-project handoffs covering Codex hooks, installer work, Claude validation, and runtime redraw debugging.
- `2026-07-02-1128.md`: Current handoff summarizing P3 traffic-light UI work, real-hook mapping fixes, and remaining stale verification.

## docs/reflections/

- `task-PRE-*.md`: Early preflight reflections for setup checks, scope confirmation, and environment assumptions.
- `task-P1-*.md`: Early Phase 1 reflections for probing, window setup, and taskbar attachment choices.
- `task-P2-*.md`: Early Phase 2 reflections for rendering, state modeling, and placement decisions.
- `task-P3-*.md`: Early Phase 3 reflections for positioning, layout, and visibility experiments.
- `task-P4-*.md`: Early Phase 4 reflections for diagnostics, captures, and runtime evidence gathering.
- `task-P5-*.md`: Early Phase 5 reflections for documentation, constraints, and deferred scope decisions.
- `task-P6-*.md`: Early Phase 6 reflections for cleanup, runtime mapping, and finalization tasks.
- `task-VAL-*.md`: Early validation reflections for build, run, visibility, and repeatability checks.
- `task-DOC-*.md`: Early documentation reflections for README, maps, and supporting notes.
- `task-CLN-*.md`: Early cleanup reflections for logs, screenshots, and naming consistency.
- `task-HC-*.md`: Hook integration reflections for initial shared-state design and validation.
- `task-HCA-*.md`: Hook adjustment reflections for notify probing, rule tightening, and Codex boundary corrections.
- `task-CSW-*.md`: Codex state-write reflections for lifecycle hook validation and live widget update checks.
- `task-GHI-*.md`: Global hook installer reflections for merge logic, dry-runs, apply flows, and multi-session validation.
- `task-CCH-*.md`: Claude Code hook validation reflections for real payload sampling, field confirmation, and shared-state writes.
- `task-TLU-*.md`: Traffic-light UI reflections for visual contract decisions, implementation, manual state tests, and real-hook smoke tests.
