# Confirm-dialog exit content snapshot

When adding an exit animation that retains a React dialog after its `open`
prop becomes false, retain the displayed content as well as the DOM. The
material-library caller clears `pendingDeletion` immediately on cancel, so its
derived title and description become empty while the dialog remains mounted for
the 120ms exit window. `ConfirmDialog` now snapshots its visible labels while
open and uses that snapshot from the first render where `open` becomes false,
including the render before the closing-state effect runs.
