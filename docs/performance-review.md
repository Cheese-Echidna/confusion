Performance review — 7 October 2026

Measurements use the current working tree, an AMD Ryzen 5 3600 (6 cores / 12 threads), and an NVIDIA RTX 3080 Ti with driver 595.104.02 on Vulkan. This is one machine, not a cross-hardware performance claim. The working tree contained ongoing changes; nothing was reset for this review.

**Desktop frame rate — measured directly**

The final desktop measurements use an isolated instrumented copy of the production app on the normal X11 desktop. Each process loads a `.con` fixture at startup, verifies its mesh and feature count, warms for three seconds, measures four seconds idle, and measures four seconds of automatic camera orbit. No file picker, keyboard/mouse injection, screenshots, or coordinate automation is used for these final results. The production source is unchanged. FPS is the rate of application render callbacks/viewport submissions, matching the app's counter; this is not an independent measurement of displayed frames.

| Design | Orbit FPS, backtraces enabled | Orbit FPS, backtraces disabled | Orbit p95 interval, enabled / disabled |
| --- | ---: | ---: | ---: |
| Plate | 49.3 | 62.4 | 22.9 / 27.1 ms |
| One pocket | 49.8 | 65.3 | 21.9 / 27.0 ms |
| Eight pockets | 33.4 | 60.0 | 35.8 / 27.2 ms |
| Twenty-four pockets | 20.0 | 60.0 | 61.8 / 27.2 ms |

Idle rates were similar: 50.2 / 49.4 / 33.7 / 20.2 FPS with backtraces enabled, and 62.3 / 67.0 / 60.0 / 60.0 FPS disabled. These are four-second windows in one run per configuration, not long-session or worst-case guarantees. The 60-ish FPS cases have irregular pacing, with short intervals interspersed with roughly 27 ms intervals; average FPS alone hides that jitter. Both `RUST_BACKTRACE` and `RUST_LIB_BACKTRACE` were set to 1 or 0 for the comparison; the inherited session had `RUST_BACKTRACE=1`.

The decisive CPU call graph for the 24-pocket model is:

```text
TextSystem::resolve_font
  TextSystem::font_id
    anyhow::format_err / Error::msg
      Backtrace::capture
        stack unwinding
```

About 89% of sampled CPU time was in the backtrace-capture path. This has a concrete cause: the UI requests `font_family("sans-serif")`, but the pinned cross-platform backend matches literal font-family names. Its `load_family` does not map that generic name to a system family. `TextSystem::font_id` caches the failed lookup, then `clone_font_id_result` reconstructs an `anyhow` error on every cache hit. With backtraces enabled, each reconstruction captures another stack trace before falling back to a working font. More sketch/browser labels amplify the cost.

**Controlled font experiment:** changing only the root font family to the installed `Noto Sans` in the isolated instrumented copy, with backtraces still enabled, produced 151–161 FPS idle and 164–165 FPS during orbit for the 24-pocket model across two runs. A third run through the saved helper measured 107 FPS idle and 59 FPS orbit, with a 1375 × 678 viewport and nearly 17 ms orbit intervals. Desktop pacing/placement/focus was not fixed across these runs, so the repeatable conclusion is the eliminated font-error path, not a guaranteed 165 FPS target. This is a diagnostic proof of the font path's importance, not an applied production fix. A portable fix should select/bundle a resolvable font or teach the backend to resolve generic families and avoid constructing fresh errors on cached failures. Hardcoding a locally installed font is not a portable application solution. This experiment changes a font family as well as avoiding the error path; layout metrics may differ, so its frame rate is an indicative improvement rather than a universal target.

Earlier screenshot-driven development/Xvfb measurements were affected by window-system pacing, transient counter samples, and unreliable dialog automation. They are excluded from the final FPS table; direct measurements supersede them.

**Model rebuilds**

The fixtures are an 80 × 50 × 10 mm plate, followed by 1, 8, or 24 separate face-attached rectangular pocket cuts. Each pocket has its own sketch and feature dependency. The timings include document validation, parameter evaluation, sketch solves, OCCT construction and Boolean operations, inspection, face mapping, and tessellation. They exclude UI delivery and mesh upload. These CAD fixtures cover planar extrusion/cut chains; curved surfaces, fillets, and large parametric assemblies remain unmeasured.

| Design | Faces | Triangles | Median full rebuild | Median parameter evaluation + sketch solves |
| --- | ---: | ---: | ---: | ---: |
| Plate | 6 | 12 | 8.0 ms | 0.19 ms |
| One pocket | 11 | 28 | 18.5 ms | 0.35 ms |
| Eight pockets | 46 | 140 | 137 ms | 1.12 ms |
| Twenty-four pockets | 126 | 396 | 731 ms | 3.73 ms |

Earlier runs measured roughly 9 / 19 / 144 / 778–803 ms, respectively. The growth is consistent despite background scheduling and clock variation. The 24-pocket rebuild's final-run upper sample was 782 ms. Eight samples per design support indicative comparisons, not reliable extreme-tail estimates.

The native kernel dominates these cases. A separate `perf` run sampled approximately 68% of CPU work inside OCCT shared libraries, 17% inside libc, and 9% inside the benchmark executable. This is a mixed-workload profile rather than a per-stage accounting: libc allocations can originate in OCCT, and the executable also contains Rust dependencies. The leading native symbols were `TopExp_Explorer::Next`, `TopoDS_Iterator::Next`, topology location operations, and volume integration. Only about 0.7% of sampled CPU work was in `libTKMesh`; finer tessellation is not the first issue to address for these planar models.

**Constrained sketches**

Independent rectangles use four points and seven constraints each. The settled solve includes rank/DOF and redundancy diagnostics. The edited solve changes the first driving width from 5 to 6 mm while retaining the original input points, requiring numerical convergence as well as diagnostics.

| Rectangles | Points | Constraints | Median settled solve | Median edited solve |
| --- | ---: | ---: | ---: | ---: |
| 1 | 4 | 7 | 0.11 ms | 0.24 ms |
| 10 | 40 | 70 | 2.55 ms | 38.8 ms |
| 24 | 96 | 168 | 18.0 ms | 33.4 ms |
| 32 | 128 | 224 | 33.9 ms | 73.6 ms |

These are five-sample measurements. Edited-solve timings have scheduling/iteration variation and are not monotonic in every row. The maximum case reaches the active sketch's 128-point limit. The solver uses dense Jacobians, augmented QR during iterations, SVD for rank/DOF, and dense redundancy diagnostics. Constraint decomposition and sparse/blocked solving are likely more valuable than simply adding threads. A 74 ms solve limits updated geometry to roughly 14 updates/sec even when camera navigation remains smooth on the background-worker architecture.

**Production renderer**

The offscreen benchmark uses `ViewportRenderer`, the same camera, shaders, mesh uploads, 4× MSAA color pass, single-sample face-ID pass, and presentation encoding as the desktop viewport. It orbits the camera and waits for device completion. Completed-frame latency is CPU wall time from submission through the GPU fence, not a pure GPU timestamp. Throughput permits batches of three frames. Neither measurement includes GPUI layout/composition, the window system, presentation, or display refresh.

The stress fixtures are synthetic grids of 100 and 10,000 cubes, totaling 1,200 and 120,000 triangles. They test rendering complexity without confounding it with OCCT rebuild cost. They are not parametric assembly benchmarks and do not contain 10,000 separate draw calls.

| Scene, grid enabled | Completed-frame median at 1600 × 1200 | Offscreen throughput |
| --- | ---: | ---: |
| Plate | 0.206 ms | 6,564 frames/sec |
| One pocket | 0.189 ms | 6,550 frames/sec |
| Eight pockets | 0.193 ms | 6,356 frames/sec |
| Twenty-four pockets | 0.189 ms | 6,481 frames/sec |
| 100 cubes / 1,200 triangles | 0.216 ms | 5,854 frames/sec |
| 10,000 cubes / 120,000 triangles | 0.228 ms | 5,130 frames/sec |

The 120,000-triangle stress case took 0.052 ms median CPU submission time and 0.412 ms p95 completed-frame latency at 1600 × 1200 with the grid. Uploading its mesh took about 2.1 ms. These numbers show substantial rendering headroom on this GPU; they must not be presented as application/display FPS.

**Bottlenecks and recommended order**

1. Fix the resolved UI font and the cached-failure error path described above. This is the largest measured interactive-frame bottleneck. Turning library backtraces off is a useful temporary workaround, but the font experiment shows that preventing the repeated lookup/error path is substantially better.
2. Cache unchanged sketch solutions and native feature shapes, then rebuild only affected descendants. `evaluation/cache.rs` and `evaluation/invalidation.rs` are scaffolds; the current `solid::evaluate` walks every sketch and reconstructs the whole native model for each request. A roughly 0.75-second rebuild is the main obstacle to responsive editing of the pocket-chain fixtures.
3. Make expensive inspection lazy or separately cached. Native evaluation computes validity, volume, area, curvature/draft samples, face membership, and interference on ordinary regeneration. Repeated topology walks and properties computations deserve stage-level profiling and reuse. The current samples do not isolate inspection's exact percentage.
4. Address dense solver scaling and repeated UUID lookup/allocation in constraint rows. Preserve useful rank/DOF diagnostics, but avoid recomputing their entire decomposition on every interactive update where possible.
5. Stop unconditional idle redraws. `WorkspaceView::render` schedules another notification with `window.on_next_frame` every frame. Invalidate on camera, document, selection, resize, and worker-result changes; retain active scheduling while dragging or waiting for a pick/result. This reduces idle CPU/GPU use and repeated UI layout.
6. Refresh picking only when camera, geometry, target size, or the pick request invalidates it. The renderer currently submits the face-ID pass every frame. Selection-color changes alone should not need a new ID image. Resolution and grid rendering affect frame cost, but the renderer is not the limiting subsystem for the tested hardware and scenes.

Native cancellation is another editing-latency risk: the worker replaces pending requests and rejects stale results, but cancellation does not interrupt the long `evaluate_complete_model` FFI call. A new edit may wait for an obsolete native rebuild before its own rebuild starts.

**Reproduce**

```sh
nix-shell --run 'cargo run --release --features gpu,solver,kernel --example performance_review'
nix-shell --run 'python3 examples/desktop_performance_review.py'
```

The example writes the four `.con` fixtures to `/tmp/confusion-performance/`. Each timing pair is `(median, p95)` using the documented small sample counts; with five/eight samples, the reported p95 is effectively the maximum. Render cases warm for 200 ms, collect 120 completed-frame samples, and measure 2,400 throughput frames. The raw renderer/model output is in `performance-review-results.txt`; direct desktop results are in `performance-review-desktop.json`; the isolated font experiment is in `performance-review-font-check.json`. The Python helper copies the application into a temporary directory, injects startup loading/frame measurement, builds it, and runs the eight desktop cases. `--font-family 'Noto Sans'` repeats the font experiment if that font is installed. It requires a graphical desktop and sends no input events. An inspectable CPU-profile excerpt is in `performance-review-cpu-profile.txt`; full CPU profiles and the original instrumented snapshot are in `/tmp/confusion-performance/` for this session.

Validation: the release benchmark evaluated all generated designs and rendered all six scenes at both resolutions with and without the grid. The Rust example compiled and its formatting check passed. The saved Python helper was also executed successfully for the 24-pocket font experiment; its JSON, fixture verification, and automatic shutdown passed. The direct desktop runs verified all four expected mesh/feature counts and reported no application errors. This review adds reproducible benchmark helpers and documentation; it does not implement the proposed optimizations.

**Optimization follow-up**

The workspace now selects an installed sans family once at startup instead of requesting the unsupported generic `sans-serif` family every frame. The GPU image reuse, optional picking pass, and MSAA store changes were reverted after visible flickering was reported. The original rendering and compositor submission path is restored. A sustained 120 FPS at 2560 × 1440 has not been validated. The renderer benchmark now includes that physical resolution. The desktop helper accepts `--viewport-size WIDTH HEIGHT` in logical pixels and records physical dimensions; at 125% display scaling, use `--viewport-size 2048 1152` for 2560 × 1440.
