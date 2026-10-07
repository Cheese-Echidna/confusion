# Background validation for Solid Apply

Create and Modify now validate candidate geometry on the existing evaluation worker.
Apply changes to **Validating…** while the request runs. The current document, undo
stack, and displayed geometry remain unchanged until exact evaluation succeeds.
The editor stays open during validation, and repeated Apply/Enter presses do not
submit duplicate requests.

On success, the candidate commits one undo checkpoint, the editor closes, and the
worker's solved sketches, mesh, face anchors, and inspection populate the viewport.
There is no second geometry rebuild just to display the committed result. Parameter
edit fields are refreshed from the committed design. Undo/redo then behave as for
other committed operations.

On failure, the editor retains its inputs and displays the error. The document and
undo stack remain unchanged. Users can correct the inputs and apply again.

Changing inputs or targets, dismissing the editor, switching tools/documents, or
changing the canonical model invalidates an in-flight candidate. Cancellation
abandons the candidate and regenerates the committed document. Undo/redo during
validation cancels that pending operation without stepping through the committed
undo stack; the next undo/redo operates normally.

A commit requires the matching evaluation revision, unchanged canonical document
fingerprint, unchanged editor inputs/selection/targets, and successful solve/mesh
results. The same revision checks protect normal viewport evaluation. Candidate
intent is never saved or added to undo until validation succeeds. Saving during
validation saves the currently committed document; later candidate success marks
it dirty as a new edit.

## Validation

The headless transaction tests reject failed results, stale revisions, changed
base documents, and changed editors, and verify that successful candidates do not
mutate their base. Worker tests cancel active candidates and then evaluate a new
request successfully. Existing solver/kernel equivalence tests cover the geometry
results reused at commit.

`examples/async_apply_review.py` runs the actual desktop WorkspaceView in an
isolated instrumented copy and temporary settings directory. It sends no input
events and automatically closes its own window. Its temporary manifest disables
automatic binary discovery. It builds `confusion-async-apply-review` explicitly
in a separate target directory under `reviews/async-apply`. This isolates
both the executable and instrumented library from normal builds.
The performance review likewise uses `confusion-desktop-review` and
`reviews/desktop-performance`. Assertions cover Create/Modify
success, one undo checkpoint per commit, editor persistence while pending,
duplicate Apply suppression, undo/redo, failed fillet recovery, dismissal, edited
inputs, undo during pending validation, document switching, and clean/dirty state
before and after commit. All 114 solver/kernel tests and 15 headless library tests
passed, along with desktop compilation and formatting checks. Submitting a
simple Box candidate returned in 0.025 ms in that single sample. This demonstrates
the asynchronous submission path, not an end-to-end latency guarantee for all tools.

Reproduce:

```sh
nix-shell --run 'python3 examples/async_apply_review.py'
nix-shell --run 'cargo test --features solver,kernel'
nix-shell --run 'cargo check --features desktop'
cargo test --lib
cargo fmt --check
```

This change covers geometry actions in Solid/Create and Solid/Modify. Other
transactions, including sketch operations and material metadata, retain their
existing paths. It does not add live geometry previews while tool fields change.
