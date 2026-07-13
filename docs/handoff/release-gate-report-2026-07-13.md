# Release gate report — 2026-07-13

## Decision

**APPROVED WITH ACCEPTED LIMITATIONS.**

No product behavior is currently known to be failing. The user explicitly waived RLS-6-01 final normal-motion desktop screenshots/recording; earlier human observation confirms the behavior, but this report does not represent it as image evidence.

## Passed

- Explorer recovery: installed release host survives Explorer restart and recovers its widget, tray registration, and attach path without host restart.
- Reduced motion: persisted Settings control and renderer behavior verified; desktop recording explicitly waived.
- Settings lifecycle and real IPC operations: debug/release lifecycle verifier plus installed Settings deployment/refresh actions passed.
- Claude: current Windows environment validates project-level PowerShell shell-form lifecycle hooks; product default remains conservative ProcessOnly.
- Release quality: workspace tests, frontend build, and separate release settings then host builds passed.
- Evidence manifest: [release-evidence-manifest-2026-07-13.md](release-evidence-manifest-2026-07-13.md).

## Accepted limitations

- Installer/upgrade/uninstall/isolated-account validation is waived by the user; this report does not claim installer evidence.
- Reduced-motion desktop visual matrix is waived by the user.
- Normal-motion desktop screenshots/recording are waived by the user; manual confirmation and diagnostics remain the available evidence.
- `cargo fmt --all -- --check` reports existing dirty-file formatting drift; no destructive formatting was performed.

## Follow-up owner

The next release owner should decide whether to normalize the existing formatting baseline in a dedicated change and whether to restore installer/clean-environment coverage before a broader distribution.
