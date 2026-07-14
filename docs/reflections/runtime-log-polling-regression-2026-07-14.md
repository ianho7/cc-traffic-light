# Runtime log polling regression (2026-07-14)

## Diagnosis

The one-second hook-state timer called `load_state_for_display_diagnostic`, which rebuilt each source summary with a fresh `updated_at` timestamp. `display_snapshot_changed` compared the complete source DTO, so an otherwise idle snapshot was considered changed every tick and emitted a persistent `snapshot/config changed` record.

## Resolution

- Compare only display-relevant fields when deciding whether to repaint or record a snapshot transition: overall state, widget mount state, error summary, source membership, and per-source visual state.
- Keep the latest full snapshot for IPC/tray consumers; only the repaint/log decision ignores volatile diagnostic fields such as `updated_at`.
- Remove timer and paint enter/exit logging from the runtime log.
- Suppress repeated identical tray-registration and named-pipe server errors until the error changes or the pipe next succeeds.

## Regression coverage

`display_snapshot_tests::ignores_source_refresh_timestamp_when_visual_state_is_unchanged` proves a changed `updated_at` value alone does not create a transition. A complementary test proves a visual source-state change still does.

## Validation

- `cargo test -p taskbar-widget --offline`
- `cargo build -p taskbar-widget --offline`
