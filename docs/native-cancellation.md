# Cancelling obsolete model rebuilds

The desktop worker now cancels its active evaluation when a newer document
revision is submitted. Dropping the worker also requests cancellation. The token
uses a shared Rust atomic flag, so OCCT progress callbacks can check it from
parallel tasks without acquiring the document/queue mutex. Native shapes and
caches remain confined to the owning evaluation worker.

`solid::evaluate_cancellable(design, token, cache)` uses the same token during
sketch solving and native evaluation. Existing closure-based evaluation APIs
remain available; their closures are checked in Rust phases, but cannot interrupt
native algorithms. The desktop worker uses the token-based API.

Native evaluation checks cancellation between features, edits, plane requests,
pattern copies, gear teeth, and inspection/mesh face traversal. Progress indicators
also request interruption inside supported OCCT Boolean, fillet, chamfer, draft,
loft, split/defeaturing, shell/offset, and meshing algorithms. Boolean inputs are
configured before `Build` rather than using constructors that eagerly evaluate
without the cancellation indicator. Cancellation is reported as `Evaluation
superseded`, rather than a misleading geometry failure.

The token is borrowed only for a synchronous native call. A scoped thread-local
pointer makes it available to native checkpoints and is restored on every exit,
including exceptions. Each progress indicator captures the token explicitly so
parallel OCCT callbacks do not depend on a thread-local pointer on another thread.
Cancelled calls can retain completed intermediate cache entries, but never publish
a successful partial mesh. Retrying with a fresh token regenerates missing results.

Worker publication and submission use the same lock order. A newer request clears
an obsolete completed result, and the worker keeps the queue lock through its final
revision check and publication. Duplicate/older revisions cannot replace a newer
pending or completed request. The UI still performs its own revision check.

## Limits

Cancellation is cooperative. Some OCCT routines, including primitive/sweep
construction and certain topology/property computations, do not expose progress
callbacks through the current bridge. Cancellation is observed at their next
checkpoint. There is no hard latency guarantee across all models or algorithms.
Create/Modify candidate validation now also uses this worker through
[background Solid Apply](async-solid-apply.md), with commit guarded by document
and editor identity.

## Validation and measurement

Unit tests cancel from inside an OCCT Boolean progress callback, verify reuse of
its completed predecessor, and retry successfully. Other tests submit a newer
revision only after the worker has entered native evaluation, verify that only the
newest result is delivered, reject older requests, and cancel an active worker on
drop. All 111 solver/kernel tests and 13 headless library tests passed, alongside
desktop compilation, formatting, and whitespace checks. The solver/kernel suite
includes the existing cached/full-rebuild equivalence tests.

The release benchmark uses the same 24-pocket plate fixture as incremental
regeneration. It requests cancellation 50 ms after evaluation starts, measures from
the actual cancellation request to return, and verifies successful regeneration
with the partially populated cache. A second scenario replaces an active 24-pocket
request with a simple plate after 50 ms and measures delivery to the worker result
slot. Both scenarios collect five samples and report medians. They exclude UI
rendering and GPU upload; the replacement measurement includes a 1 ms result-poll
interval.

Measured on this development machine on 7 October 2026:

| Measurement | Time |
| --- | ---: |
| Uncached 24-pocket rebuild, single reference sample | 567.2 ms |
| Cancellation request → native return, median | 1.06 ms |
| Replacement plate submission → worker result, median | 5.31 ms |

These timings are indicative results for this fixture, not a bound on cancellation
latency or a comparison across hardware. The Boolean construction change also
removes eager/repeated work, so the reference rebuild time should not be compared
with earlier reports as if cancellation alone made rebuilds faster.

Reproduce:

```sh
nix-shell --run 'RUST_BACKTRACE=0 RUST_LIB_BACKTRACE=0 cargo run --release --features solver,kernel --example cancellation_review'
nix-shell --run 'cargo test --features solver,kernel'
nix-shell --run 'cargo check --features desktop'
cargo test --lib
cargo fmt --check
```
