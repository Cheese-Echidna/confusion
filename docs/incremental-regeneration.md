# Incremental regeneration

The desktop evaluation worker retains disposable sketch, native feature, and final
mesh/inspection caches between requests. `.con` files still contain only model
intent; opening a file with a fresh worker regenerates it without cached data.
`solid::evaluate` remains an uncached reference path. `solid::evaluate_cached`
accepts a caller-owned `EvaluationCache`, which must stay on its owning thread.

Sketch keys include geometry, constraints, and the resolved values of parameters
used by those constraints. Changing an unrelated parameter does not rerun that
sketch's numerical solver. Parameter expressions and document validity are checked
on every request, including cache hits.

Extrusion and Create keys include their resolved inputs, exact profile definitions,
and upstream shape keys. A changed feature invalidates its descendants; independent
bodies can retain their cached shapes. Native entries preserve extrusion cap
provenance, cut-body pieces, and body-consumption effects. Shapes are deep-copied
when stored and retrieved to prevent OCCT operations from mutating cached topology.

Modify checkpoints include all preceding feature/edit keys because face ordinals
and body availability depend on the complete model. Changing a late Modify input
can reuse earlier checkpoints; an upstream change conservatively invalidates later
Modify checkpoints, even if it affects another body. Reordering features or changing
profile-array offsets can also invalidate otherwise equivalent entries.

The final mesh, support planes, and inspection are reused when all native inputs
and plane requests match. After an actual geometry change, final tessellation,
face mapping, and inspection still run. Each cache keeps one entry per current
sketch/feature/edit and one final mesh, rather than accumulating edit history.
Modify checkpoints retain complete native model snapshots, so their memory cost
grows with both model size and the number of Modify operations. Removing entries
releases their cached results.

Failures do not overwrite the last successful final mesh. Successful intermediate
results may remain cached after a later feature fails or a request is superseded.
Revision checks continue to prevent stale results from reaching the UI. The worker
now also uses [cooperative native cancellation](native-cancellation.md) to interrupt
obsolete requests at checkpoints and inside supported OCCT operations.

## Validation and measurement

Six integration tests compare cached evaluation with full regeneration, including
triangle data, volume, inspection area/validity, support planes, and face anchors.
They cover driving dimensions, upstream and downstream changes, independent
extrusion/Create bodies, Modify checkpoints, undo-like restoration, save/reopen,
failed operations, and cancellation recovery. The full solver/kernel suite passed
107 tests, the headless library suite passed 13 tests, and desktop compilation and
formatting checks passed.

The release benchmark uses the same 80 × 50 × 10 mm plate with 24 sequential
face-attached pockets as the performance review. It alternates each selected
parameter between two values and measures five paired cached/full evaluations;
the reported time is the median. The cache is warmed before measurements. Full
rebuilds use the actual uncached native path. Assertions check equal volumes for
every pair. These are model-evaluation wall times, excluding UI delivery and GPU
upload, measured on this development machine; five samples do not establish tail
latency or performance on other hardware.

Measured on 7 October 2026:

| Request | Full rebuild | Cached rebuild | Reused native features |
| --- | ---: | ---: | ---: |
| Change final pocket depth | 725.5 ms | 159.0 ms | 24 of 25 |
| Change base plate thickness | 739.9 ms | 751.0 ms | 0 of 25 |
| Repeat unchanged model | — | 3.87 ms | 25 of 25 |

The final-pocket edit is approximately 4.6× faster. Base-plate changes still rebuild
the entire chain and incur a small cache-storage overhead in this run. All 25
sketch solutions were reused for these solid-depth edits. Remaining late-edit cost
includes copying native topology and regenerating final inspection/tessellation.

Reproduce:

```sh
nix-shell --run 'RUST_BACKTRACE=0 RUST_LIB_BACKTRACE=0 cargo run --release --features solver,kernel --example incremental_review'
nix-shell --run 'cargo test --features solver,kernel'
nix-shell --run 'cargo check --features desktop'
cargo test --lib
cargo fmt --check
```
