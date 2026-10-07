# Crash recovery

Edited unsaved tabs and modified saved documents are checkpointed as complete current
intent snapshots. The UI checks for changes once per second, waits for two seconds of
quiet, and limits snapshot batches to one per ten seconds. Continuous editing receives
a checkpoint after thirty seconds. Only one batch runs at a time on a background thread;
newer edits remain eligible for a later batch. Undo history, evaluated geometry and meshes
are never serialized. The existing `.con` writer validates intent and atomically replaces
snapshots, syncing the file and (on Unix) its containing directory.

On the next startup, an on-screen recovery prompt offers each abandoned snapshot in
turn. It shows point and parameter counts and approximate snapshot age. Recover opens
an unsaved tab with empty undo history and leaves a recovery copy protected until the
user saves or closes it. Discard removes only the displayed candidate. Damaged snapshots
show their error and can be discarded without blocking valid snapshots. Snapshot errors
appear in the status area with the storage path and troubleshooting advice.

Storage uses `confusion/recovery` under XDG state storage on Linux, Application Support
on macOS, or LocalAppData on Windows. `CONFUSION_RECOVERY_DIR` overrides the root for
troubleshooting and tests. Each running workspace owns a unique directory protected by
an OS advisory file lock. Startup skips live workspaces and claims abandoned directories,
so another running instance cannot offer or delete their snapshots. Per-tab UUIDs survive
tab switches. Save, intentional close, and undo back to saved state remove only the
relevant current-session snapshots. Cleanup commands run after prior queued writes,
preventing an older write from resurrecting intentionally discarded work. The worker
retains its session lock until queued commands drain after the UI is dropped. Final
intentional window close drains cleanup before allowing process exit; manager shutdown
closes the worker channel and joins the worker. Normal frame cleanup stays asynchronous.

Recovery is a checkpoint, so edits since the last successful snapshot can be lost.
Unchanged empty new documents do not receive snapshots. Recovery copies do not retain
original document paths, camera state, or tab names; users choose a destination when
saving recovered documents. Empty session directories and their lock files are retained
(their size is small), avoiding unsafe directory pruning while concurrent instances run.
If an instance quits without using the normal close controls, its remaining snapshots
are offered as recovery candidates. Recovery requires local storage supporting advisory
file locks and atomic replacement; errors are reported rather than silently claiming
protection. Startup validation reads each candidate independently; very large collections
of abandoned snapshots may slow startup.

Headless coverage: `nix-shell --run 'cargo test --test crash_recovery'`.
Desktop integration check: `nix-shell --run 'cargo check --features desktop'`.
