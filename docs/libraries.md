# Library choices

These choices serve the confirmed local-only scope: sketches, solids, components and
technical drawings. Research used primary project documentation on 7 October 2026.
`Cargo.toml` contains the foundation and optional core backend dependency groups.
Supporting IO/UI libraries below are selected to add when their implementations
start, rather than forcing every unused codec/native toolchain into this scaffold.
`Cargo.lock` is the reproducible resolution; do not use `*` for the GPUI family.

## Core selections

| Library | Purpose and owner | Decision / limitation |
| --- | --- | --- |
| [GPUI](https://gpui.rs), pinned GPUI-compatible WGPUI fork | Desktop UI, focus, input, window lifecycle; `ui`, `application`, `platform` | Optional `desktop` feature. The viewport proof uses shared-device surfaces from WGPUI; validate the experimental compositor across platforms before committing to it long term. |
| [wgpu](https://docs.rs/wgpu/latest/wgpu/), matching pinned WGPUI Git revision (28.0.0) | Depth-tested 3D, shading, edge passes, picking; `render` | Optional `gpu` feature. Keep behind renderer interfaces; source and revision match the compositor so Device/Texture types are shared. |
| [bytemuck](https://docs.rs/bytemuck/latest/bytemuck/) | Checked POD buffer layouts; `render` | Optional with GPU. GPU structs are derived data, separate from persisted f64 geometry. |
| [Open CASCADE](https://dev.opencascade.org/doc/overview/html/occt_user_guides__modeling_algos.html) | Exact B-rep curves/solids, Booleans, fillets, shells, offsets, meshing and hidden-line removal; `kernel` | Selected native kernel. Pin a tested native release/build, then expose only required APIs. No native kernel is built or bundled yet. |
| [CXX](https://cxx.rs) | Owned Rust/C++ OCCT bridge; `kernel/bridge` | Optional `kernel` feature currently brings only CXX. Add `cxx-build`/CMake/native link configuration with the first real bridge, not a fake empty build script. |
| [faer](https://docs.rs/faer/latest/faer/), 0.24 | Sparse/dense least squares and rank analysis; `solver` | Optional `solver` feature. Matrix decompositions support an owned geometric solver; faer is not itself a CAD constraint solver. |
| [Rayon](https://docs.rs/rayon/latest/rayon/) | Parallel solve islands, residuals and Jacobians; `solver/parallel` | Optional `solver` feature. One explicit thread budget coordinates Rayon and faer parallelism. |
| [nalgebra](https://docs.rs/nalgebra/latest/nalgebra/), 0.34 | Small f64 vectors, frames, rigid transforms; `foundation/math` | Use one spatial math vocabulary across model, solver and kernel. faer serves larger linear systems. |
| [petgraph](https://docs.rs/petgraph/latest/petgraph/), 0.8 | Dependency graph construction, traversal and cycle detection | Graph indexes are temporary; UUIDs are persisted identity. Our graph types retain CAD semantics. |
| [serde](https://serde.rs) + [serde_json](https://docs.rs/serde_json/latest/serde_json/) | Explicit `.con` DTOs and one JSON settings schema | Deserialize validated disk schemas, not arbitrary runtime state. |
| [uuid](https://docs.rs/uuid/latest/uuid/) | Durable typed IDs | v4 ID generation and serde support; wrap IDs in distinct newtypes. |
| [blake3](https://docs.rs/blake3/latest/blake3/) | Asset digests and semantic cache keys | Canonicalize inputs before hashing; a cache hash is not a face identity strategy. |
| [thiserror](https://docs.rs/thiserror/latest/thiserror/) | Typed errors at module boundaries | Preserve diagnostic codes and source entity IDs. |
| [tracing](https://docs.rs/tracing/latest/tracing/) | Job/solver/kernel/render spans and diagnostics | Local logging only. Add a subscriber at application startup when runtime code exists. |

The desktop/GPU features now implement the viewport proof; solver and kernel
features still reserve dependencies for future implementations. Headless checks should be the default while these
contracts are being established. A later desktop package can enable its required
backends by default once it is an actual application.

## Supporting libraries to bring in with implementation

| Library | Responsibility | Why this choice |
| --- | --- | --- |
| [zip](https://docs.rs/zip/latest/zip/) | `persistence/container` | `.con` bundles JSON and required exact-source/texture/font assets. Enable only required compression, with explicit decompression limits. |
| [tempfile](https://docs.rs/tempfile/latest/tempfile/) | `persistence/atomic` | Same-directory temporary writes; platform adapter handles flush/replace durability. |
| [directories](https://docs.rs/directories/latest/directories/) | `platform/paths` | Native settings/cache/recovery directories without inventing path conventions. |
| [notify](https://docs.rs/notify/latest/notify/) | `settings/store`, `platform/files` | Watch the one settings JSON file and explicit local component sources; debounce and validate updates. |
| [stl_io](https://docs.rs/stl_io/latest/stl_io/) | `exchange/stl` | Binary/ASCII STL encoding/decoding. Our code handles scale, tolerances and mesh validity. |
| [dxf](https://docs.rs/dxf/latest/dxf/) | `exchange/dxf`, `drawing/export` | DXF entity IO for sketches and vector drafting; implement and test an explicit supported-entity subset. |
| [usvg](https://docs.rs/usvg/latest/usvg/) | `exchange/svg` | Normalize SVG paths/transforms for sketch input. Reject or report effects that cannot become CAD curves. |
| [pdf-writer](https://docs.rs/pdf-writer/latest/pdf_writer/) | `drawing/export` | Vector PDF primitives and document encoding; own layout, line weights, fonts and associative geometry. |
| [ttf-parser](https://docs.rs/ttf-parser/latest/ttf_parser/) + [rustybuzz](https://docs.rs/rustybuzz/latest/rustybuzz/) | Sketch text and drawing text | Font outlines plus shaping. Preserve exact font content used for geometry in `.con`, subject to embedding permissions. |
| [rstar](https://docs.rs/rstar/latest/rstar/) | Sketch snapping, 2D region broad phase | Spatial indexes accelerate candidate queries; they do not replace exact curve intersection algorithms. |
| [tracing-subscriber](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/) | `runtime/telemetry` | Local configurable log output, with settings read from the one JSON file. |
| [proptest](https://docs.rs/proptest/latest/proptest/), [approx](https://docs.rs/approx/latest/approx/), [criterion](https://docs.rs/criterion/latest/criterion/) | Future development-only verification | Numeric invariants, dimensional checks, graph/patch properties and representative performance benchmarks. Add alongside meaningful implementation, not tests for empty placeholders. |

Use standard-library channels/mutexes initially for bounded job plumbing plus
dedicated Rayon pools. GPUI's executors handle UI tasks. Do not add Tokio merely to
schedule CPU work, or a second window/input stack such as winit over GPUI. No HTTP,
authentication, database or cloud-sync libraries are needed.

Use GPUI/native dialogs before adding another toolkit. For controls, own a compact
Fusion/Zed-inspired design system. [gpui-base](https://github.com/longbridge/gpui-kit)
is a possible behavior dependency once its GPUI revision matches the selected
compositor; it is not a committed dependency. Do not couple domain APIs to a UI kit
or assume its default styling fits a CAD application.

## Why OCCT instead of a new Rust kernel

The selected scope requires analytic/NURBS geometry, robust B-rep topology,
Booleans, fillets, shells, loft/sweep operations and STEP assemblies. The
[OCCT modeling guide](https://dev.opencascade.org/doc/overview/html/occt_user_guides__modeling_algos.html)
documents the needed algorithm families. It is the practical baseline; writing a
new exact kernel would turn this project into a separate multi-year kernel effort.

[opencascade-rs](https://github.com/bschwind/opencascade-rs) is useful precedent for
CXX-based bindings and simple modeling APIs, but its own README calls it a work in
progress. Choose an owned narrow bridge so we can expose provenance, exact queries,
native validity reports, cancellation and
[XDE assembly exchange](https://dev.opencascade.org/doc/overview/html/occt_user_guides__xde.html)
without waiting for complete upstream wrapper coverage. Reuse or contribute binding
code where suitable; do not expose third-party wrapper shape types across the app.

Build dependencies will include a C++ toolchain, CMake and the pinned OCCT development
libraries. Decide bundled versus packaged OCCT per platform after reproducible build
spikes; shipping binaries must not rely on users installing a CAD development SDK.
Use the OCCT license and third-party notice requirements for that chosen release.

## Why an owned Rust solver

[SolveSpace/slvs](https://github.com/thekakkun/rust_slvs) provides an existing geometric
solver, and would reduce initial numerical implementation. The wrapper is GPL-3.0
and requires native build/binding tools. [SolveSpace's library interface](https://solvespace.github.io/solvespace-web/library.html)
is worth benchmarking if its coverage and distribution terms fit the eventual project.
Do not silently choose the application's license by adding it now.

The current decision is an owned Rust solver: faer supplies sparse QR and dense
rank-revealing QR/SVD, while Rayon provides island/residual parallelism. This permits
one revision/cancellation model, dimensional residual scaling, interactive continuation
and source-linked diagnostics. It also carries a large implementation burden:
equations, derivatives, degeneracy handling, branch continuity and conflict analysis
must be implemented and validated. Preserve `ConstraintSolver`/`LinearSolveBackend`
interfaces to replace the numerical backend if benchmarks warrant it.

## GPUI version and viewport gate

The starting manifest used wildcards and a crates.io `gpui_platform` dependency.
`cargo info gpui_platform` found no published crate. Published GPUI 0.2.2 includes
its own platform backends; the manifest now pins it explicitly and keeps it optional.
The latest upstream split
[gpui_platform manifest](https://github.com/zed-industries/zed/blob/main/crates/gpui_platform/Cargo.toml)
is a different dependency arrangement and must not be mixed blindly with that release.

Inspection of the installed GPUI 0.2.2 `src/elements/surface.rs` shows only a macOS
CoreVideo surface variant. Upstream also describes the cross-platform viewport gap
in its [external compositor proposal](https://github.com/zed-industries/zed/discussions/60572).
Thus `gpui + wgpu` alone is **not** a finished cross-platform integration. Prove a
compositor adapter/fork and pin the full compatible GPUI family. Our
`render/gpui_bridge` and `platform/gpu_surface` are the explicit implementation sites.
Keep production presentation on the GPU; do not conceal the gap with per-frame
CPU image uploads. The chosen bridge also determines whether wgpu matching pinned WGPUI Git revision (28.0.0) stays.

## Work that libraries do not supply

No selected library delivers Fusion workflow parity automatically. Confusion owns
the persistent dependency graph, profile recognition, stable topology naming,
constraint/DOF feedback, parameter/configuration semantics, component joint workflows,
associative drawing standards and repair UX. OCCT supplies exact geometric operations;
faer supplies linear algebra; GPUI and wgpu supply presentation foundations. The
parity checklist measures the complete workflows across those boundaries.

## Implemented sketch/extrusion path

The `desktop` feature now enables `solver` (faer 0.24 and Rayon) and `kernel` (CXX
and cxx-build). `shell.nix` supplies OCCT 7.9.3 include/library paths. Root `build.rs`
compiles the value-only owned extrusion bridge, with native shapes kept in the worker.
ZIP 6 with deflate and tempfile implement bounded, atomic `.con` containers.
Unicode segmentation supports GPUI input editing. The reusable input is adapted from
the pinned WGPUI Apache-2.0 example; its license is retained under `licenses/`.
See [parametric-workflow.md](parametric-workflow.md) for actual behavior and limits.
